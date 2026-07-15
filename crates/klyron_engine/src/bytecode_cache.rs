use std::path::{Path, PathBuf};
use std::sync::Mutex;

use lru::LruCache;
use serde::{Deserialize, Serialize};

use crate::engine::JsEngineKind;

const CACHE_DIR: &str = ".klyron/cache/bytecode";
const MAX_CACHE_ENTRIES: usize = 1000;
const MAX_CACHE_SIZE_BYTES: u64 = 512 * 1024 * 1024;

const ENGINE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedBytecode {
    pub engine_kind: JsEngineKind,
    pub path: String,
    pub content_hash: String,
    pub bytecode: Vec<u8>,
    pub compiled_at: u64,
    pub version: u32,
    pub size_bytes: u64,
}

pub struct BytecodeCache {
    cache_dir: PathBuf,
    lru: Mutex<LruCache<String, CachedBytecode>>,
    current_size: Mutex<u64>,
    max_size_bytes: u64,
}

impl BytecodeCache {
    pub fn new() -> Self {
        let cache_dir = dirs_cache_dir();
        std::fs::create_dir_all(&cache_dir).ok();
        Self {
            cache_dir,
            lru: Mutex::new(LruCache::new(
                std::num::NonZeroUsize::new(MAX_CACHE_ENTRIES).unwrap(),
            )),
            current_size: Mutex::new(0),
            max_size_bytes: MAX_CACHE_SIZE_BYTES,
        }
    }

    pub fn with_max_size(max_size_bytes: u64) -> Self {
        let cache_dir = dirs_cache_dir();
        std::fs::create_dir_all(&cache_dir).ok();
        Self {
            cache_dir,
            lru: Mutex::new(LruCache::new(
                std::num::NonZeroUsize::new(MAX_CACHE_ENTRIES).unwrap(),
            )),
            current_size: Mutex::new(0),
            max_size_bytes,
        }
    }

    pub fn get_or_compile(
        &self,
        path: &str,
        content: &str,
        engine_kind: JsEngineKind,
        compiler: impl FnOnce(&str, &str) -> Result<Vec<u8>, String>,
    ) -> Result<Vec<u8>, String> {
        let hash = hash_content(content);
        let key = cache_key(path, &hash, engine_kind);

        {
            let mut lru = self.lru.lock().map_err(|e| e.to_string())?;
            if let Some(entry) = lru.get(&key) {
                if entry.content_hash == hash
                    && entry.engine_kind == engine_kind
                    && entry.version == ENGINE_VERSION
                {
                    return Ok(entry.bytecode.clone());
                }
            }
        }

        let disk_path = self.disk_path(&key);
        if disk_path.exists() {
            if let Ok(cached) = load_from_disk(&disk_path) {
                if cached.content_hash == hash
                    && cached.engine_kind == engine_kind
                    && cached.version == ENGINE_VERSION
                {
                    let bytecode = cached.bytecode.clone();
                    let mut lru = self.lru.lock().map_err(|e| e.to_string())?;
                    lru.put(key.clone(), cached);
                    return Ok(bytecode);
                }
            }
        }

        let bytecode = compiler(path, content)?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let size_bytes = bytecode.len() as u64;
        let entry = CachedBytecode {
            engine_kind,
            path: path.to_string(),
            content_hash: hash,
            bytecode: bytecode.clone(),
            compiled_at: now,
            version: ENGINE_VERSION,
            size_bytes,
        };

        {
            let mut lru = self.lru.lock().map_err(|e| e.to_string())?;
            lru.put(key.clone(), entry.clone());
        }

        {
            let mut current_size = self.current_size.lock().map_err(|e| e.to_string())?;
            *current_size += size_bytes;
            while *current_size > self.max_size_bytes {
                let evicted = {
                    let mut lru = self.lru.lock().map_err(|e| e.to_string())?;
                    lru.pop_lru()
                };
                match evicted {
                    Some((_k, evicted_entry)) => {
                        *current_size = current_size.saturating_sub(evicted_entry.size_bytes);
                        let evict_path = self.disk_path(&cache_key(
                            &evicted_entry.path,
                            &evicted_entry.content_hash,
                            evicted_entry.engine_kind,
                        ));
                        std::fs::remove_file(&evict_path).ok();
                    }
                    None => break,
                }
            }
        }

        if let Ok(data) = bincode::serialize(&entry) {
            std::fs::write(&disk_path, data).ok();
        }

        Ok(bytecode)
    }

    pub fn hash_content_blake3(content: &str) -> String {
        let hash = blake3::hash(content.as_bytes());
        hash.to_hex().to_string()
    }

    pub fn invalidate(&self, path: &str) {
        let mut lru = self.lru.lock().unwrap();
        let keys: Vec<String> = lru.iter()
            .filter(|(k, _)| k.starts_with(path))
            .map(|(k, _)| k.clone())
            .collect();
        for k in keys {
            lru.pop(&k);
        }
    }

    pub fn clear(&self) {
        let mut lru = self.lru.lock().unwrap();
        lru.clear();
        *self.current_size.lock().unwrap() = 0;
        std::fs::remove_dir_all(&self.cache_dir).ok();
        std::fs::create_dir_all(&self.cache_dir).ok();
    }

    pub fn len(&self) -> usize {
        self.lru.lock().map(|l| l.len()).unwrap_or(0)
    }

    pub fn total_size_bytes(&self) -> u64 {
        *self.current_size.lock().unwrap()
    }

    fn disk_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.bc", key))
    }
}

impl Default for BytecodeCache {
    fn default() -> Self {
        Self::new()
    }
}

fn dirs_cache_dir() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(CACHE_DIR)
    } else {
        PathBuf::from("/tmp/.klyron/cache/bytecode")
    }
}

fn hash_content(content: &str) -> String {
    let hash = blake3::hash(content.as_bytes());
    hash.to_hex().to_string()
}

fn cache_key(path: &str, hash: &str, kind: JsEngineKind) -> String {
    format!("{}:{}:{}:v{}", path, hash, kind.name(), ENGINE_VERSION)
}

fn load_from_disk(path: &Path) -> Result<CachedBytecode, String> {
    let data = std::fs::read(path).map_err(|e| format!("Failed to read cache: {}", e))?;
    bincode::deserialize(&data).map_err(|e| format!("Failed to deserialize cache: {}", e))
}
