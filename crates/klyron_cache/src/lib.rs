use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct CacheEntry {
    value: String,
    expires_at: Option<Instant>,
    tags: Vec<String>,
    hit_count: u64,
    created_at: Instant,
}

pub struct CacheManager {
    store: Mutex<HashMap<String, CacheEntry>>,
    default_ttl: Duration,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
            default_ttl: Duration::from_secs(300),
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let mut store = self.store.lock().unwrap();
        if let Some(entry) = store.get_mut(key) {
            if let Some(expires) = entry.expires_at {
                if Instant::now() > expires {
                    store.remove(key);
                    return None;
                }
            }
            entry.hit_count += 1;
            return Some(entry.value.clone());
        }
        None
    }

    pub fn get_or_set<F>(&self, key: &str, f: F) -> String
    where
        F: FnOnce() -> String,
    {
        if let Some(value) = self.get(key) {
            return value;
        }
        let value = f();
        self.set(key, &value);
        value
    }

    pub fn set(&self, key: &str, value: &str) {
        let mut store = self.store.lock().unwrap();
        store.insert(key.to_string(), CacheEntry {
            value: value.to_string(),
            expires_at: Some(Instant::now() + self.default_ttl),
            tags: Vec::new(),
            hit_count: 0,
            created_at: Instant::now(),
        });
    }

    pub fn set_with_ttl(&self, key: &str, value: &str, ttl: Duration) {
        let mut store = self.store.lock().unwrap();
        store.insert(key.to_string(), CacheEntry {
            value: value.to_string(),
            expires_at: Some(Instant::now() + ttl),
            tags: Vec::new(),
            hit_count: 0,
            created_at: Instant::now(),
        });
    }

    pub fn set_with_tags(&self, key: &str, value: &str, tags: &[&str]) {
        let mut store = self.store.lock().unwrap();
        store.insert(key.to_string(), CacheEntry {
            value: value.to_string(),
            expires_at: Some(Instant::now() + self.default_ttl),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            hit_count: 0,
            created_at: Instant::now(),
        });
    }

    pub fn set_forever(&self, key: &str, value: &str) {
        let mut store = self.store.lock().unwrap();
        store.insert(key.to_string(), CacheEntry {
            value: value.to_string(),
            expires_at: None,
            tags: Vec::new(),
            hit_count: 0,
            created_at: Instant::now(),
        });
    }

    pub fn delete(&self, key: &str) -> bool {
        let mut store = self.store.lock().unwrap();
        store.remove(key).is_some()
    }

    pub fn delete_pattern(&self, pattern: &str) -> usize {
        let mut store = self.store.lock().unwrap();
        let before = store.len();
        store.retain(|k, _| !k.contains(pattern));
        before - store.len()
    }

    pub fn clear(&self) {
        let mut store = self.store.lock().unwrap();
        store.clear();
    }

    pub fn has(&self, key: &str) -> bool {
        let mut store = self.store.lock().unwrap();
        if let Some(entry) = store.get_mut(key) {
            if let Some(expires) = entry.expires_at {
                if Instant::now() > expires {
                    store.remove(key);
                    return false;
                }
            }
            return true;
        }
        false
    }

    pub fn increment(&self, key: &str, amount: i64) -> Option<i64> {
        let mut store = self.store.lock().unwrap();
        let now = Instant::now();
        if let Some(entry) = store.get_mut(key) {
            if let Some(expires) = entry.expires_at {
                if now > expires {
                    store.remove(key);
                    return None;
                }
            }
            if let Ok(val) = entry.value.parse::<i64>() {
                let new_val = val + amount;
                entry.value = new_val.to_string();
                entry.hit_count += 1;
                return Some(new_val);
            }
        }
        None
    }

    pub fn get_hit_count(&self, key: &str) -> Option<u64> {
        let store = self.store.lock().unwrap();
        store.get(key).map(|e| e.hit_count)
    }

    pub fn get_ttl(&self, key: &str) -> Option<Duration> {
        let store = self.store.lock().unwrap();
        store.get(key).and_then(|e| {
            e.expires_at.map(|exp| {
                exp.checked_duration_since(Instant::now()).unwrap_or(Duration::ZERO)
            })
        })
    }

    pub fn touch(&self, key: &str) -> bool {
        let mut store = self.store.lock().unwrap();
        if let Some(entry) = store.get_mut(key) {
            entry.expires_at = Some(Instant::now() + self.default_ttl);
            true
        } else {
            false
        }
    }

    pub fn evict_expired(&self) -> usize {
        let mut store = self.store.lock().unwrap();
        let now = Instant::now();
        let before = store.len();
        store.retain(|_, entry| {
            entry.expires_at.map_or(true, |exp| now <= exp)
        });
        before - store.len()
    }

    pub fn evict_by_tag(&self, tag: &str) -> usize {
        let mut store = self.store.lock().unwrap();
        let before = store.len();
        store.retain(|_, entry| !entry.tags.contains(&tag.to_string()));
        before - store.len()
    }

    pub fn evict_by_pattern(&self, pattern: &str) -> usize {
        self.delete_pattern(pattern)
    }

    pub fn size(&self) -> usize {
        let store = self.store.lock().unwrap();
        store.len()
    }

    pub fn keys(&self) -> Vec<String> {
        let store = self.store.lock().unwrap();
        store.keys().cloned().collect()
    }

    pub fn keys_by_tag(&self, tag: &str) -> Vec<String> {
        let store = self.store.lock().unwrap();
        store.iter()
            .filter(|(_, entry)| entry.tags.contains(&tag.to_string()))
            .map(|(k, _)| k.clone())
            .collect()
    }

    pub fn stats(&self) -> CacheStats {
        let store = self.store.lock().unwrap();
        let total_hits: u64 = store.values().map(|e| e.hit_count).sum();
        let expired_count = store.values().filter(|e| {
            e.expires_at.map_or(false, |exp| Instant::now() > exp)
        }).count();
        let oldest = store.values().map(|e| e.created_at).min();
        CacheStats {
            entries: store.len(),
            total_hits,
            expired_count,
            oldest_entry: oldest,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub total_hits: u64,
    pub expired_count: usize,
    pub oldest_entry: Option<Instant>,
}

impl Default for CacheManager {
    fn default() -> Self { Self::new() }
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
    fn test_cache_delete() {
        let cache = CacheManager::new();
        cache.set("key1", "value1");
        assert!(cache.delete("key1"));
        assert_eq!(cache.get("key1"), None);
    }

    #[test]
    fn test_cache_has() {
        let cache = CacheManager::new();
        cache.set("key1", "value1");
        assert!(cache.has("key1"));
        assert!(!cache.has("key2"));
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
    fn test_cache_increment() {
        let cache = CacheManager::new();
        cache.set("counter", "10");
        assert_eq!(cache.increment("counter", 5), Some(15));
        assert_eq!(cache.get("counter"), Some("15".to_string()));
    }

    #[test]
    fn test_cache_get_or_set() {
        let cache = CacheManager::new();
        let val = cache.get_or_set("computed", || "expensive_value".to_string());
        assert_eq!(val, "expensive_value");
        assert_eq!(cache.get("computed"), Some("expensive_value".to_string()));
    }

    #[test]
    fn test_cache_tags() {
        let cache = CacheManager::new();
        cache.set_with_tags("user:1", "Alice", &["user", "admin"]);
        cache.set_with_tags("user:2", "Bob", &["user"]);
        assert_eq!(cache.keys_by_tag("user").len(), 2);
        assert_eq!(cache.keys_by_tag("admin").len(), 1);
        cache.evict_by_tag("admin");
        assert!(!cache.has("user:1"));
        assert!(cache.has("user:2"));
    }

    #[test]
    fn test_cache_delete_pattern() {
        let cache = CacheManager::new();
        cache.set("session:abc", "data1");
        cache.set("session:def", "data2");
        cache.set("user:123", "data3");
        let deleted = cache.delete_pattern("session:");
        assert_eq!(deleted, 2);
        assert!(!cache.has("session:abc"));
        assert!(cache.has("user:123"));
    }

    #[test]
    fn test_cache_stats() {
        let cache = CacheManager::new();
        cache.set("a", "1");
        cache.set("b", "2");
        cache.get("a");
        cache.get("a");
        cache.get("b");
        let stats = cache.stats();
        assert_eq!(stats.entries, 2);
        assert!(stats.total_hits >= 3);
    }
}
