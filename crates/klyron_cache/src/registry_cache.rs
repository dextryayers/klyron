use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tracing::{debug, trace, warn};

const METADATA_TTL: Duration = Duration::from_secs(300);
const LRU_MAX_ENTRIES: usize = 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedMetadata {
    pub data: Vec<u8>,
    pub etag: Option<String>,
    pub cached_at: Instant,
}

impl CachedMetadata {
    pub fn is_fresh(&self) -> bool {
        self.cached_at.elapsed() < METADATA_TTL
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTarball {
    pub data: Vec<u8>,
    pub integrity: Option<String>,
}

struct LruEntry {
    key: String,
    metadata: CachedMetadata,
}

pub struct RegistryCache {
    lru: Mutex<VecDeque<LruEntry>>,
    disk_cache_dir: PathBuf,
}

impl RegistryCache {
    pub fn new(cache_dir: &Path) -> Self {
        let disk_cache_dir = cache_dir.join("registry");
        let _ = std::fs::create_dir_all(&disk_cache_dir);
        Self {
            lru: Mutex::new(VecDeque::with_capacity(LRU_MAX_ENTRIES)),
            disk_cache_dir,
        }
    }

    pub fn get_metadata(&self, key: &str) -> Option<CachedMetadata> {
        {
            let mut lru = self.lru.lock().ok()?;
            if let Some(pos) = lru.iter().position(|e| e.key == key) {
                if let Some(entry) = lru.remove(pos) {
                    if entry.metadata.is_fresh() {
                        trace!("Registry metadata cache HIT: {}", key);
                        lru.push_back(LruEntry {
                            key: entry.key.clone(),
                            metadata: entry.metadata.clone(),
                        });
                        return Some(entry.metadata);
                    } else {
                        debug!("Registry metadata cache expired: {}", key);
                    }
                }
            }
        }
        let disk_key = sanitize_key(key);
        let disk_path = self.disk_cache_dir.join(&disk_key).with_extension("meta.json");
        if disk_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&disk_path) {
                if let Ok(meta) = serde_json::from_str::<CachedMetadata>(&content) {
                    if meta.is_fresh() {
                        trace!("Registry metadata disk cache HIT: {}", key);
                        let mut lru = self.lru.lock().ok()?;
                        lru.push_back(LruEntry {
                            key: key.to_string(),
                            metadata: meta.clone(),
                        });
                        return Some(meta);
                    } else {
                        let _ = std::fs::remove_file(&disk_path);
                    }
                }
            }
        }
        None
    }

    pub fn set_metadata(&self, key: &str, data: &[u8], etag: Option<String>) {
        let meta = CachedMetadata {
            data: data.to_vec(),
            etag,
            cached_at: Instant::now(),
        };
        {
            let mut lru = self.lru.lock().ok();
            if let Some(ref mut lru) = lru {
                if lru.len() >= LRU_MAX_ENTRIES {
                    lru.pop_front();
                }
                lru.push_back(LruEntry {
                    key: key.to_string(),
                    metadata: meta.clone(),
                });
            }
        }
        let disk_key = sanitize_key(key);
        let disk_path = self.disk_cache_dir.join(&disk_key).with_extension("meta.json");
        if let Ok(content) = serde_json::to_string(&meta) {
            let _ = std::fs::write(&disk_path, content);
        }
    }

    pub fn get_tarball(&self, key: &str) -> Option<CachedTarball> {
        let disk_key = sanitize_key(key);
        let disk_path = self.disk_cache_dir.join(&disk_key).with_extension("tgz");
        if disk_path.exists() {
            if let Ok(data) = std::fs::read(&disk_path) {
                trace!("Registry tarball disk cache HIT: {}", key);
                return Some(CachedTarball {
                    data,
                    integrity: None,
                });
            }
        }
        None
    }

    pub fn set_tarball(&self, key: &str, data: &[u8], integrity: Option<String>) {
        let tarball = CachedTarball {
            data: data.to_vec(),
            integrity,
        };
        let disk_key = sanitize_key(key);
        let disk_path = self.disk_cache_dir.join(&disk_key).with_extension("tgz");
        if let Err(e) = std::fs::write(&disk_path, &tarball.data) {
            warn!("Failed to write tarball cache: {}", e);
        }
    }

    pub fn invalidate(&self, key: &str) {
        {
            let mut lru = self.lru.lock().ok();
            if let Some(ref mut lru) = lru {
                lru.retain(|e| e.key != key);
            }
        }
        let disk_key = sanitize_key(key);
        for ext in &["meta.json", "tgz"] {
            let path = self.disk_cache_dir.join(&disk_key).with_extension(ext);
            let _ = std::fs::remove_file(&path);
        }
    }

    pub fn clear(&self) {
        {
            let mut lru = self.lru.lock().ok();
            if let Some(ref mut lru) = lru {
                lru.clear();
            }
        }
        if let Ok(entries) = std::fs::read_dir(&self.disk_cache_dir) {
            for entry in entries.flatten() {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    pub fn size(&self) -> usize {
        let lru = self.lru.lock().ok();
        lru.map(|l| l.len()).unwrap_or(0)
    }
}

fn sanitize_key(key: &str) -> String {
    key.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_metadata_cache() {
        let dir = std::env::temp_dir().join("_klyron_reg_cache_test");
        let _ = std::fs::create_dir_all(&dir);
        let cache = RegistryCache::new(&dir);

        assert!(cache.get_metadata("test-pkg").is_none());
        cache.set_metadata("test-pkg", b"{\"name\":\"test\"}", Some("\"abc123\"".into()));
        let got = cache.get_metadata("test-pkg");
        assert!(got.is_some());
        assert_eq!(got.unwrap().data, b"{\"name\":\"test\"}");

        cache.invalidate("test-pkg");
        assert!(cache.get_metadata("test-pkg").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_tarball_cache() {
        let dir = std::env::temp_dir().join("_klyron_tarball_cache_test");
        let _ = std::fs::create_dir_all(&dir);
        let cache = RegistryCache::new(&dir);

        assert!(cache.get_tarball("test-pkg-1.0.0").is_none());
        cache.set_tarball("test-pkg-1.0.0", b"tarball-data", Some("sha512-abc".into()));
        let got = cache.get_tarball("test-pkg-1.0.0");
        assert!(got.is_some());
        assert_eq!(got.unwrap().data, b"tarball-data");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cache_clear() {
        let dir = std::env::temp_dir().join("_klyron_cache_clear_test");
        let _ = std::fs::create_dir_all(&dir);
        let cache = RegistryCache::new(&dir);
        cache.set_metadata("pkg-a", b"data", None);
        cache.set_tarball("pkg-b-1.0", b"tarball", None);
        cache.clear();
        assert_eq!(cache.size(), 0);
        assert!(cache.get_metadata("pkg-a").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
