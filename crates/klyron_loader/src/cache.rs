use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

static RESOLVE_CACHE: Lazy<Mutex<HashMap<String, Option<PathBuf>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn resolve_cache_get(key: &str) -> Option<PathBuf> {
    if let Ok(cache) = RESOLVE_CACHE.lock() {
        if let Some(Some(path)) = cache.get(key) {
            return Some(path.clone());
        }
    }
    None
}

pub fn resolve_cache_insert(key: &str, path: PathBuf) {
    if let Ok(mut cache) = RESOLVE_CACHE.lock() {
        cache.insert(key.to_string(), Some(path));
    }
}

pub fn clear_resolve_cache() {
    if let Ok(mut cache) = RESOLVE_CACHE.lock() {
        cache.clear();
    }
}

pub fn get_resolve_cache_size() -> usize {
    RESOLVE_CACHE.lock().map(|c| c.len()).unwrap_or(0)
}

pub fn remove_from_cache(key: &str) -> bool {
    if let Ok(mut cache) = RESOLVE_CACHE.lock() {
        cache.remove(key).is_some()
    } else {
        false
    }
}

pub struct CacheStats {
    pub entries: usize,
    pub keys: Vec<String>,
}

pub fn get_cache_stats() -> CacheStats {
    if let Ok(cache) = RESOLVE_CACHE.lock() {
        CacheStats {
            entries: cache.len(),
            keys: cache.keys().cloned().collect(),
        }
    } else {
        CacheStats {
            entries: 0,
            keys: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_and_get() {
        clear_resolve_cache();
        resolve_cache_insert("test-key", PathBuf::from("/tmp/test.js"));
        let result = resolve_cache_get("test-key");
        assert_eq!(result, Some(PathBuf::from("/tmp/test.js")));
    }

    #[test]
    fn test_cache_get_missing() {
        clear_resolve_cache();
        let result = resolve_cache_get("nonexistent");
        assert_eq!(result, None);
    }

    #[test]
    fn test_clear_cache() {
        resolve_cache_insert("a", PathBuf::from("/a.js"));
        resolve_cache_insert("b", PathBuf::from("/b.js"));
        assert!(get_resolve_cache_size() > 0);
        clear_resolve_cache();
        assert_eq!(get_resolve_cache_size(), 0);
    }

    #[test]
    fn test_remove_from_cache() {
        clear_resolve_cache();
        resolve_cache_insert("test-key", PathBuf::from("/tmp/test.js"));
        assert!(remove_from_cache("test-key"));
        assert!(!remove_from_cache("nonexistent"));
    }

    #[test]
    fn test_cache_stats() {
        clear_resolve_cache();
        resolve_cache_insert("x", PathBuf::from("/x.js"));
        let stats = get_cache_stats();
        assert!(stats.entries >= 1);
    }
}
