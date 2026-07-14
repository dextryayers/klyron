use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct CacheEntry {
    value: String,
    expires_at: Option<Instant>,
    tags: Vec<String>,
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
        let store = self.store.lock().unwrap();
        if let Some(entry) = store.get(key) {
            if let Some(expires) = entry.expires_at {
                if Instant::now() > expires {
                    return None;
                }
            }
            return Some(entry.value.clone());
        }
        None
    }

    pub fn set(&self, key: &str, value: &str) {
        let mut store = self.store.lock().unwrap();
        store.insert(key.to_string(), CacheEntry {
            value: value.to_string(),
            expires_at: Some(Instant::now() + self.default_ttl),
            tags: Vec::new(),
        });
    }

    pub fn set_with_ttl(&self, key: &str, value: &str, ttl: Duration) {
        let mut store = self.store.lock().unwrap();
        store.insert(key.to_string(), CacheEntry {
            value: value.to_string(),
            expires_at: Some(Instant::now() + ttl),
            tags: Vec::new(),
        });
    }

    pub fn set_with_tags(&self, key: &str, value: &str, tags: &[&str]) {
        let mut store = self.store.lock().unwrap();
        store.insert(key.to_string(), CacheEntry {
            value: value.to_string(),
            expires_at: Some(Instant::now() + self.default_ttl),
            tags: tags.iter().map(|t| t.to_string()).collect(),
        });
    }

    pub fn delete(&self, key: &str) -> bool {
        let mut store = self.store.lock().unwrap();
        store.remove(key).is_some()
    }

    pub fn clear(&self) {
        let mut store = self.store.lock().unwrap();
        store.clear();
    }

    pub fn has(&self, key: &str) -> bool {
        let store = self.store.lock().unwrap();
        if let Some(entry) = store.get(key) {
            if let Some(expires) = entry.expires_at {
                if Instant::now() > expires {
                    return false;
                }
            }
            return true;
        }
        false
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

    pub fn size(&self) -> usize {
        let store = self.store.lock().unwrap();
        store.len()
    }

    pub fn keys(&self) -> Vec<String> {
        let store = self.store.lock().unwrap();
        store.keys().cloned().collect()
    }
}

impl Default for CacheManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
