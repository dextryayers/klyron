use std::collections::HashMap;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

#[derive(Debug, Clone)]
struct CacheEntry {
    value: Vec<u8>,
    expires_at: Option<Instant>,
    #[allow(dead_code)]
    tags: Vec<String>,
    hit_count: u64,
    #[allow(dead_code)]
    created_at: Instant,
}

impl CacheEntry {
    #[inline]
    fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |exp| exp <= Instant::now())
    }
}

#[derive(Debug, Clone)]
pub struct CacheEntryInfo {
    pub value: Vec<u8>,
    pub expires_at: Option<Instant>,
    pub tags: Vec<String>,
    pub hit_count: u64,
    pub created_at: Instant,
}

const DEFAULT_SHARDS: usize = 16;

struct Shard {
    store: HashMap<String, CacheEntry>,
    lfu_counts: HashMap<String, u64>,
}

pub struct ConcurrentCache {
    shards: Vec<Mutex<Shard>>,
    shard_mask: usize,
    default_ttl: Duration,
    max_size: usize,
}

impl ConcurrentCache {
    pub fn new() -> Self {
        Self::with_shards(DEFAULT_SHARDS)
    }

    pub fn with_shards(count: usize) -> Self {
        let count = count.next_power_of_two();
        let shards = (0..count).map(|_| {
            Mutex::new(Shard {
                store: HashMap::new(),
                lfu_counts: HashMap::new(),
            })
        }).collect();
        Self {
            shards,
            shard_mask: count - 1,
            default_ttl: Duration::from_secs(300),
            max_size: 10_000,
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    pub fn with_max_size(mut self, max: usize) -> Self {
        self.max_size = max;
        self
    }

    #[inline]
    fn shard_idx(&self, key: &str) -> usize {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize & self.shard_mask
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let idx = self.shard_idx(key);
        let mut shard = self.shards[idx].lock();
        let is_expired = {
            let entry = shard.store.get(key);
            entry.map_or(false, |e| e.is_expired())
        };
        if is_expired {
            shard.store.remove(key);
            shard.lfu_counts.remove(key);
            return None;
        }
        if let Some(entry) = shard.store.get_mut(key) {
            entry.hit_count += 1;
            let value = entry.value.clone();
            *shard.lfu_counts.entry(key.to_string()).or_insert(0) += 1;
            return Some(value);
        }
        None
    }

    pub fn get_or_insert_with<F>(&self, key: &str, f: F) -> Vec<u8>
    where
        F: FnOnce() -> Vec<u8>,
    {
        if let Some(value) = self.get(key) {
            return value;
        }
        let value = f();
        self.set(key, &value);
        value
    }

    pub fn set(&self, key: &str, value: &[u8]) {
        let idx = self.shard_idx(key);
        let mut shard = self.shards[idx].lock();

        if shard.store.len() >= self.max_size {
            self.evict_lfu(&mut shard);
        }

        shard.store.insert(key.to_string(), CacheEntry {
            value: value.to_vec(),
            expires_at: Some(Instant::now() + self.default_ttl),
            tags: Vec::new(),
            hit_count: 0,
            created_at: Instant::now(),
        });
    }

    pub fn set_many(&self, items: Vec<(&str, &[u8])>) {
        for (key, value) in items {
            self.set(key, value);
        }
    }

    pub fn set_with_ttl(&self, key: &str, value: &[u8], ttl: Duration) {
        let idx = self.shard_idx(key);
        let mut shard = self.shards[idx].lock();

        if shard.store.len() >= self.max_size {
            self.evict_lfu(&mut shard);
        }

        shard.store.insert(key.to_string(), CacheEntry {
            value: value.to_vec(),
            expires_at: Some(Instant::now() + ttl),
            tags: Vec::new(),
            hit_count: 0,
            created_at: Instant::now(),
        });
    }

    pub fn set_with_tags(&self, key: &str, value: &[u8], tags: &[&str]) {
        let idx = self.shard_idx(key);
        let mut shard = self.shards[idx].lock();

        if shard.store.len() >= self.max_size {
            self.evict_lfu(&mut shard);
        }

        shard.store.insert(key.to_string(), CacheEntry {
            value: value.to_vec(),
            expires_at: Some(Instant::now() + self.default_ttl),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            hit_count: 0,
            created_at: Instant::now(),
        });
    }

    fn evict_lfu(&self, shard: &mut Shard) {
        let min_key = shard.lfu_counts.iter()
            .min_by_key(|(_, count)| **count)
            .map(|(k, _)| k.clone());
        if let Some(key) = min_key {
            shard.store.remove(&key);
            shard.lfu_counts.remove(&key);
        }
    }

    pub fn delete(&self, key: &str) -> bool {
        let idx = self.shard_idx(key);
        let mut shard = self.shards[idx].lock();
        let existed = shard.store.remove(key).is_some();
        shard.lfu_counts.remove(key);
        existed
    }

    pub fn scan(&self, pattern: &str) -> Vec<String> {
        let mut results = Vec::new();
        for shard in &self.shards {
            let guard = shard.lock();
            for key in guard.store.keys() {
                if key.contains(pattern) {
                    results.push(key.clone());
                }
            }
        }
        results
    }

    pub fn clear(&self) {
        for shard in &self.shards {
            let mut guard = shard.lock();
            guard.store.clear();
            guard.lfu_counts.clear();
        }
    }

    pub fn has(&self, key: &str) -> bool {
        let idx = self.shard_idx(key);
        let mut shard = self.shards[idx].lock();
        if let Some(entry) = shard.store.get_mut(key) {
            if entry.is_expired() {
                shard.store.remove(key);
                shard.lfu_counts.remove(key);
                return false;
            }
            return true;
        }
        false
    }

    pub fn expires_at(&self, key: &str) -> Option<Instant> {
        let idx = self.shard_idx(key);
        let shard = self.shards[idx].lock();
        shard.store.get(key).and_then(|e| e.expires_at)
    }

    pub fn evict_expired(&self) -> usize {
        let mut total = 0;
        for shard in &self.shards {
            let mut guard = shard.lock();
            let now = Instant::now();
            let expired: Vec<String> = guard.store.iter()
                .filter(|(_, e)| e.expires_at.map_or(false, |exp| exp <= now))
                .map(|(k, _)| k.clone())
                .collect();
            total += expired.len();
            for key in &expired {
                guard.store.remove(key);
                guard.lfu_counts.remove(key);
            }
        }
        total
    }

    pub fn size(&self) -> usize {
        self.shards.iter().map(|s| s.lock().store.len()).sum()
    }
}

pub struct CacheManager {
    concurrent: ConcurrentCache,
}

impl CacheManager {
    #[inline]
    pub fn new() -> Self {
        Self {
            concurrent: ConcurrentCache::new().with_max_size(10_000),
        }
    }

    #[inline]
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.concurrent = ConcurrentCache::new().with_ttl(ttl).with_max_size(10_000);
        self
    }

    #[inline]
    pub fn get(&self, key: &str) -> Option<String> {
        self.concurrent.get(key).and_then(|v| String::from_utf8(v).ok())
    }

    #[inline]
    pub fn get_or_insert_with<F>(&self, key: &str, f: F) -> String
    where
        F: FnOnce() -> String,
    {
        let v = self.concurrent.get_or_insert_with(key, || f().into_bytes());
        String::from_utf8(v).unwrap()
    }

    #[inline]
    pub fn set(&self, key: &str, value: &str) {
        self.concurrent.set(key, value.as_bytes());
    }

    #[inline]
    pub fn set_many(&self, items: Vec<(&str, &str)>) {
        let items: Vec<(&str, &[u8])> = items.iter().map(|(k, v)| (*k, v.as_bytes())).collect();
        self.concurrent.set_many(items);
    }

    #[inline]
    pub fn set_with_ttl(&self, key: &str, value: &str, ttl: Duration) {
        self.concurrent.set_with_ttl(key, value.as_bytes(), ttl);
    }

    #[inline]
    pub fn set_with_tags(&self, key: &str, value: &str, tags: &[&str]) {
        self.concurrent.set_with_tags(key, value.as_bytes(), tags);
    }

    #[inline]
    pub fn set_forever(&self, key: &str, value: &str) {
        self.concurrent.set_with_ttl(key, value.as_bytes(), Duration::from_secs(u64::MAX));
    }

    #[inline]
    pub fn delete(&self, key: &str) -> bool {
        self.concurrent.delete(key)
    }

    #[inline]
    pub fn scan(&self, pattern: &str) -> Vec<String> {
        self.concurrent.scan(pattern)
    }

    #[inline]
    pub fn clear(&self) {
        self.concurrent.clear();
    }

    #[inline]
    pub fn has(&self, key: &str) -> bool {
        self.concurrent.has(key)
    }

    #[inline]
    pub fn expires_at(&self, key: &str) -> Option<Instant> {
        self.concurrent.expires_at(key)
    }

    #[inline]
    pub fn evict_expired(&self) -> usize {
        self.concurrent.evict_expired()
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.concurrent.size()
    }

    #[inline]
    pub fn keys(&self) -> Vec<String> {
        self.concurrent.scan("")
    }
}

impl Default for CacheManager {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_set_get() {
        let cache = CacheManager::new();
        cache.set("key1", "value1");
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
    }

    #[test]
    fn test_cache_set_many() {
        let cache = CacheManager::new();
        cache.set_many(vec![("a", "1"), ("b", "2")]);
        assert_eq!(cache.get("a"), Some("1".to_string()));
        assert_eq!(cache.get("b"), Some("2".to_string()));
    }

    #[test]
    fn test_cache_delete() {
        let cache = CacheManager::new();
        cache.set("key1", "value1");
        assert!(cache.delete("key1"));
        assert_eq!(cache.get("key1"), None);
    }

    #[test]
    fn test_cache_scan() {
        let cache = CacheManager::new();
        cache.set("session:abc", "data1");
        cache.set("session:def", "data2");
        cache.set("user:123", "data3");
        let matches = cache.scan("session:");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_cache_get_or_insert_with() {
        let cache = CacheManager::new();
        let val = cache.get_or_insert_with("computed", || "expensive_value".to_string());
        assert_eq!(val, "expensive_value");
        let cached = cache.get_or_insert_with("computed", || "new_value".to_string());
        assert_eq!(cached, "expensive_value");
    }

    #[test]
    fn test_cache_clear() {
        let cache = CacheManager::new();
        cache.set("key1", "value1");
        cache.set("key2", "value2");
        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_cache_ttl() {
        let cache = CacheManager::new().with_ttl(Duration::from_millis(10));
        cache.set("key", "val");
        assert!(cache.has("key"));
        thread::sleep(Duration::from_millis(20));
        assert!(!cache.has("key"));
    }

    #[test]
    fn test_cache_expires_at() {
        let cache = CacheManager::new().with_ttl(Duration::from_secs(300));
        cache.set("key", "val");
        let exp = cache.expires_at("key");
        assert!(exp.is_some());
        assert!(exp.unwrap() > Instant::now());
    }
}
