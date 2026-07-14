use std::path::{Path, PathBuf};
use std::sync::Mutex;

use lru::LruCache;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::engine::JsEngineKind;

const CACHE_DIR: &str = ".klyron/cache/bytecode";
const MAX_CACHE_ENTRIES: usize = 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedBytecode {
    pub engine_kind: JsEngineKind,
    pub path: String,
    pub content_hash: String,
    pub bytecode: Vec<u8>,
    pub compiled_at: u64,
    pub version: u32,
}

pub struct BytecodeCache {
    cache_dir: PathBuf,
    lru: Mutex<LruCache<String, CachedBytecode>>,
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
                if entry.content_hash == hash && entry.engine_kind == engine_kind {
                    return Ok(entry.bytecode.clone());
                }
            }
        }

        let disk_path = self.disk_path(&key);
        if disk_path.exists() {
            if let Ok(cached) = load_from_disk(&disk_path) {
                if cached.content_hash == hash && cached.engine_kind == engine_kind {
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

        let entry = CachedBytecode {
            engine_kind,
            path: path.to_string(),
            content_hash: hash,
            bytecode: bytecode.clone(),
            compiled_at: now,
            version: 1,
        };

        {
            let mut lru = self.lru.lock().map_err(|e| e.to_string())?;
            lru.put(key.clone(), entry.clone());
        }

        if let Ok(data) = bincode::serialize(&entry) {
            std::fs::write(&disk_path, data).ok();
        }

        Ok(bytecode)
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
        std::fs::remove_dir_all(&self.cache_dir).ok();
        std::fs::create_dir_all(&self.cache_dir).ok();
    }

    pub fn len(&self) -> usize {
        self.lru.lock().map(|l| l.len()).unwrap_or(0)
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
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

fn cache_key(path: &str, hash: &str, kind: JsEngineKind) -> String {
    format!("{}:{}:{}", path, hash, kind.name())
}

fn load_from_disk(path: &Path) -> Result<CachedBytecode, String> {
    let data = std::fs::read(path).map_err(|e| format!("Failed to read cache: {}", e))?;
    bincode::deserialize(&data).map_err(|e| format!("Failed to deserialize cache: {}", e))
}
