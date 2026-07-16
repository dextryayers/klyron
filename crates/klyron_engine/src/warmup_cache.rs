use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct WarmupEntry {
    pub code: String,
    pub key: String,
    pub compiled_size: usize,
}

pub struct WarmupCache {
    cache: Mutex<HashMap<String, WarmupEntry>>,
    max_entries: usize,
}

impl WarmupCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            max_entries,
        }
    }

    pub fn register(&self, key: &str, code: &str) {
        let mut cache = self.cache.lock().unwrap();
        if cache.len() >= self.max_entries {
            cache.clear();
        }
        cache.insert(
            key.to_string(),
            WarmupEntry {
                code: code.to_string(),
                key: key.to_string(),
                compiled_size: code.len(),
            },
        );
    }

    pub fn get(&self, key: &str) -> Option<WarmupEntry> {
        self.cache.lock().unwrap().get(key).cloned()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.cache.lock().unwrap().contains_key(key)
    }

    pub fn precompile_common(&self) {
        let common_snippets = vec![
            ("console_polyfill", "var console = { log: function() {}, error: function() {}, warn: function() {} };"),
            ("json_polyfill", "var JSON = { parse: function(s) { return eval('('+s+')'); }, stringify: function(o) { return String(o); } };"),
            ("timers_polyfill", "var setTimeout = function(f,n) { f(); }; var clearTimeout = function() {};"),
        ];
        for (key, code) in common_snippets {
            self.register(key, code);
        }
    }

    pub fn len(&self) -> usize {
        self.cache.lock().unwrap().len()
    }

    pub fn clear(&self) {
        self.cache.lock().unwrap().clear();
    }
}

impl Default for WarmupCache {
    fn default() -> Self {
        Self::new(64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warmup_cache_new() {
        let cache = WarmupCache::new(10);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_register_and_get() {
        let cache = WarmupCache::new(10);
        cache.register("test_key", "console.log('hello');");
        assert!(cache.contains("test_key"));
        let entry = cache.get("test_key").unwrap();
        assert_eq!(entry.key, "test_key");
        assert_eq!(entry.code, "console.log('hello');");
    }

    #[test]
    fn test_get_nonexistent() {
        let cache = WarmupCache::new(10);
        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn test_max_entries_eviction() {
        let cache = WarmupCache::new(2);
        cache.register("a", "code_a");
        cache.register("b", "code_b");
        cache.register("c", "code_c");
        // Since len >= max_entries, cache is cleared before inserting
        assert_eq!(cache.len(), 1);
        assert!(cache.contains("c"));
    }

    #[test]
    fn test_precompile_common() {
        let cache = WarmupCache::new(10);
        cache.precompile_common();
        assert!(cache.contains("console_polyfill"));
        assert!(cache.contains("json_polyfill"));
        assert!(cache.contains("timers_polyfill"));
    }

    #[test]
    fn test_clear() {
        let cache = WarmupCache::new(10);
        cache.register("k", "code");
        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_warmup_entry_compiled_size() {
        let cache = WarmupCache::new(10);
        cache.register("test", "some code here");
        let entry = cache.get("test").unwrap();
        assert_eq!(entry.compiled_size, "some code here".len());
    }

    #[test]
    fn test_default() {
        let cache = WarmupCache::default();
        assert_eq!(cache.len(), 0);
    }
}
