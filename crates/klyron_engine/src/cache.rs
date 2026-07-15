use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use lru::LruCache;
use serde::{Deserialize, Serialize};

const DEFAULT_CACHE_DIR: &str = ".klyron/cache/engine";
const DEFAULT_MAX_MEMORY_ENTRIES: usize = 10000;
const DEFAULT_MAX_MEMORY_SIZE_MB: u64 = 256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_size_mb: u64,
    pub ttl_secs: u64,
    pub disk_path: PathBuf,
    pub compression: bool,
    pub max_memory_entries: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size_mb: 512,
            ttl_secs: 3600,
            disk_path: get_default_cache_dir(),
            compression: true,
            max_memory_entries: DEFAULT_MAX_MEMORY_ENTRIES,
        }
    }
}

impl CacheConfig {
    pub fn with_ttl(mut self, ttl_secs: u64) -> Self {
        self.ttl_secs = ttl_secs;
        self
    }

    pub fn with_max_size(mut self, max_size_mb: u64) -> Self {
        self.max_size_mb = max_size_mb;
        self
    }

    pub fn with_disk_path(mut self, path: PathBuf) -> Self {
        self.disk_path = path;
        self
    }

    pub fn with_compression(mut self, compression: bool) -> Self {
        self.compression = compression;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub value: Vec<u8>,
    pub created_at: u64,
    pub expires_at: u64,
    pub size_bytes: u64,
    pub content_type: String,
    pub compressed: bool,
}

pub struct MemoryCache {
    cache: LruCache<String, CacheEntry>,
    current_size: u64,
    max_size: u64,
    config: CacheConfig,
}

impl MemoryCache {
    pub fn new(config: CacheConfig) -> Self {
        let max_entries = config.max_memory_entries;
        Self {
            cache: LruCache::new(std::num::NonZeroUsize::new(max_entries).unwrap()),
            current_size: 0,
            max_size: config.max_size_mb * 1024 * 1024,
            config,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<CacheEntry> {
        let now = now_secs();
        if let Some(entry) = self.cache.get(key) {
            if entry.expires_at > 0 && now > entry.expires_at {
                self.cache.pop(key);
                self.current_size = self.current_size.saturating_sub(entry.size_bytes);
                return None;
            }
            return Some(entry.clone());
        }
        None
    }

    pub fn put(&mut self, key: String, value: Vec<u8>, ttl_secs: u64, content_type: String) {
        let now = now_secs();
        let size = value.len() as u64;

        let entry = CacheEntry {
            key: key.clone(),
            value,
            created_at: now,
            expires_at: if ttl_secs > 0 { now + ttl_secs } else { 0 },
            size_bytes: size,
            content_type,
            compressed: self.config.compression,
        };

        while self.current_size + size > self.max_size {
            if let Some((_k, evicted)) = self.cache.pop_lru() {
                self.current_size = self.current_size.saturating_sub(evicted.size_bytes);
            } else {
                break;
            }
        }

        self.current_size += size;
        self.cache.put(key, entry);
    }

    pub fn remove(&mut self, key: &str) {
        if let Some(entry) = self.cache.pop(key) {
            self.current_size = self.current_size.saturating_sub(entry.size_bytes);
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_size = 0;
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn current_size_bytes(&self) -> u64 {
        self.current_size
    }

    pub fn max_size_bytes(&self) -> u64 {
        self.max_size
    }

    pub fn contains(&self, key: &str) -> bool {
        let now = now_secs();
        self.cache.get(key).map_or(false, |e| e.expires_at == 0 || now <= e.expires_at)
    }
}

pub struct DiskCache {
    disk_path: PathBuf,
    config: CacheConfig,
}

impl DiskCache {
    pub fn new(config: CacheConfig) -> Self {
        let path = &config.disk_path;
        std::fs::create_dir_all(path).ok();
        Self {
            disk_path: path.clone(),
            config,
        }
    }

    pub fn get(&self, key: &str) -> Option<CacheEntry> {
        let path = self.disk_path_for(key);
        if !path.exists() { return None; }

        let data = std::fs::read(&path).ok()?;
        let entry: CacheEntry = bincode::deserialize(&data).ok()?;

        let now = now_secs();
        if entry.expires_at > 0 && now > entry.expires_at {
            std::fs::remove_file(&path).ok();
            return None;
        }

        Some(entry)
    }

    pub fn put(&self, key: String, value: Vec<u8>, ttl_secs: u64, content_type: String) {
        let now = now_secs();
        let size = value.len() as u64;

        let entry = CacheEntry {
            key: key.clone(),
            value,
            created_at: now,
            expires_at: if ttl_secs > 0 { now + ttl_secs } else { 0 },
            size_bytes: size,
            content_type,
            compressed: self.config.compression,
        };

        let path = self.disk_path_for(&key);
        if let Ok(data) = bincode::serialize(&entry) {
            std::fs::write(&path, data).ok();
        }
    }

    pub fn remove(&self, key: &str) {
        let path = self.disk_path_for(key);
        std::fs::remove_file(&path).ok();
    }

    pub fn clear(&self) {
        std::fs::remove_dir_all(&self.disk_path).ok();
        std::fs::create_dir_all(&self.disk_path).ok();
    }

    pub fn len(&self) -> usize {
        std::fs::read_dir(&self.disk_path)
            .map(|entries| entries.filter_map(|e| e.ok()).count())
            .unwrap_or(0)
    }

    pub fn current_size_bytes(&self) -> u64 {
        std::fs::read_dir(&self.disk_path).ok()
            .map(|entries| {
                entries.filter_map(|e| e.ok())
                    .filter_map(|e| e.metadata().ok())
                    .map(|m| m.len())
                    .sum()
            })
            .unwrap_or(0)
    }

    fn disk_path_for(&self, key: &str) -> PathBuf {
        let safe_key = key.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
        self.disk_path.join(format!("{}.cache", safe_key))
    }
}

pub struct TwoTierCache {
    memory: Mutex<MemoryCache>,
    disk: DiskCache,
    config: CacheConfig,
}

impl TwoTierCache {
    pub fn new(config: CacheConfig) -> Self {
        let memory = MemoryCache::new(config.clone());
        let disk = DiskCache::new(config.clone());
        Self {
            memory: Mutex::new(memory),
            disk,
            config,
        }
    }

    pub fn get(&self, key: &str) -> Option<CacheEntry> {
        if let Some(entry) = self.memory.lock().ok()?.get(key) {
            return Some(entry);
        }

        if let Some(entry) = self.disk.get(key) {
            let mut memory = self.memory.lock().ok()?;
            memory.put(key.to_string(), entry.value.clone(), self.config.ttl_secs, entry.content_type.clone());
            return Some(entry);
        }

        None
    }

    pub fn put(&self, key: String, value: Vec<u8>, ttl_secs: u64, content_type: String) {
        let ttl = if ttl_secs > 0 { ttl_secs } else { self.config.ttl_secs };
        if let Ok(mut memory) = self.memory.lock() {
            memory.put(key.clone(), value.clone(), ttl, content_type.clone());
        }
        self.disk.put(key, value, ttl, content_type);
    }

    pub fn remove(&self, key: &str) {
        if let Ok(mut memory) = self.memory.lock() {
            memory.remove(key);
        }
        self.disk.remove(key);
    }

    pub fn clear(&self) {
        if let Ok(mut memory) = self.memory.lock() {
            memory.clear();
        }
        self.disk.clear();
    }

    pub fn memory_len(&self) -> usize {
        self.memory.lock().map(|m| m.len()).unwrap_or(0)
    }

    pub fn disk_len(&self) -> usize {
        self.disk.len()
    }

    pub fn total_entries(&self) -> usize {
        self.memory_len() + self.disk_len()
    }

    pub fn memory_size_bytes(&self) -> u64 {
        self.memory.lock().map(|m| m.current_size_bytes()).unwrap_or(0)
    }

    pub fn disk_size_bytes(&self) -> u64 {
        self.disk.current_size_bytes()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.memory.lock().map(|m| m.contains(key)).unwrap_or(false)
            || self.disk.get(key).is_some()
    }
}

fn get_default_cache_dir() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(DEFAULT_CACHE_DIR)
    } else {
        PathBuf::from("/tmp/.klyron/cache/engine")
    }
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}
