use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::DnsResult;

struct CacheEntry {
    result: DnsResult,
    expires: Instant,
}

pub struct DnsCache {
    entries: Mutex<HashMap<String, CacheEntry>>,
    default_ttl: Duration,
}

impl DnsCache {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            default_ttl,
        }
    }

    pub fn get(&self, key: &str) -> Option<DnsResult> {
        let entries = self.entries.lock().ok()?;
        if let Some(entry) = entries.get(key) {
            if Instant::now() < entry.expires {
                return Some(entry.result.clone());
            }
        }
        None
    }

    pub fn set(&self, key: String, result: DnsResult) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.insert(
                key,
                CacheEntry {
                    expires: Instant::now() + self.default_ttl,
                    result,
                },
            );
        }
    }

    pub fn set_ttl(&self, key: String, result: DnsResult, ttl: Duration) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.insert(
                key,
                CacheEntry {
                    expires: Instant::now() + ttl,
                    result,
                },
            );
        }
    }

    pub fn remove(&self, key: &str) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.remove(key);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
    }

    pub fn len(&self) -> usize {
        self.entries.lock().map(|e| e.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn prune_expired(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.retain(|_, entry| Instant::now() < entry.expires);
        }
    }
}

impl Default for DnsCache {
    fn default() -> Self {
        Self::new(Duration::from_secs(60))
    }
}
