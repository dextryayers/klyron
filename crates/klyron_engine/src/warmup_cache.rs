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
