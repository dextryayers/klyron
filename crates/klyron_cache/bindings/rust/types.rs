use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub total_hits: u64,
    pub expired_count: usize,
    pub oldest_entry: Option<Instant>,
}

pub struct CacheManager {
    store: std::sync::Mutex<std::collections::HashMap<String, String>>,
    default_ttl: Duration,
}
impl CacheManager {
    pub fn new() -> Self { Self { store: std::sync::Mutex::new(std::collections::HashMap::new()), default_ttl: Duration::from_secs(300) } }
    pub fn with_ttl(mut self, ttl: Duration) -> Self { self.default_ttl = ttl; self }
    pub fn get(&self, key: &str) -> Option<String> { self.store.lock().unwrap().get(key).cloned() }
    pub fn get_or_set<F: FnOnce() -> String>(&self, key: &str, f: F) -> String { self.get(key).unwrap_or_else(|| { let v = f(); self.set(key, &v); v }) }
    pub fn set(&self, key: &str, value: &str) { self.store.lock().unwrap().insert(key.to_string(), value.to_string()); }
    pub fn set_with_ttl(&self, key: &str, value: &str, _ttl: Duration) { self.set(key, value); }
    pub fn set_with_tags(&self, key: &str, value: &str, _tags: &[&str]) { self.set(key, value); }
    pub fn set_forever(&self, key: &str, value: &str) { self.set(key, value); }
    pub fn delete(&self, key: &str) -> bool { self.store.lock().unwrap().remove(key).is_some() }
    pub fn delete_pattern(&self, pattern: &str) -> usize { let mut s = self.store.lock().unwrap(); let before = s.len(); s.retain(|k, _| !k.contains(pattern)); before - s.len() }
    pub fn clear(&self) { self.store.lock().unwrap().clear() }
    pub fn has(&self, key: &str) -> bool { self.store.lock().unwrap().contains_key(key) }
    pub fn increment(&self, key: &str, amount: i64) -> Option<i64> { let mut s = self.store.lock().unwrap(); s.get_mut(key).and_then(|v| { let n = v.parse::<i64>().ok()? + amount; *v = n.to_string(); Some(n) }) }
    pub fn size(&self) -> usize { self.store.lock().unwrap().len() }
    pub fn keys(&self) -> Vec<String> { self.store.lock().unwrap().keys().cloned().collect() }
    pub fn stats(&self) -> CacheStats { let s = self.store.lock().unwrap(); CacheStats { entries: s.len(), total_hits: 0, expired_count: 0, oldest_entry: None } }
}
impl Default for CacheManager { fn default() -> Self { Self::new() } }
