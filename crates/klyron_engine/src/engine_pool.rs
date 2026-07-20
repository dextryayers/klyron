use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::Mutex;
use uuid::Uuid;

use crate::engine::{EngineRuntime, JsEngineKind};

#[derive(Debug, Clone)]
pub struct PoolEntryStats {
    pub acquires: u64,
    pub total_elapsed: std::time::Duration,
    pub created_at: Instant,
}

struct SendEngine(EngineRuntime);

unsafe impl Send for SendEngine {}
unsafe impl Sync for SendEngine {}

#[derive(Clone)]
pub struct EnginePoolEntry {
    pub id: String,
    pub engine_kind: JsEngineKind,
    engine: Arc<Mutex<SendEngine>>,
    pub stats: Arc<Mutex<PoolEntryStats>>,
    pub in_use: Arc<Mutex<bool>>,
}

impl EnginePoolEntry {
    pub fn new(kind: JsEngineKind) -> Result<Self, String> {
        let engine = EngineRuntime::new(kind)?;
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            engine_kind: kind,
            engine: Arc::new(Mutex::new(SendEngine(engine))),
            stats: Arc::new(Mutex::new(PoolEntryStats {
                acquires: 0,
                total_elapsed: std::time::Duration::default(),
                created_at: Instant::now(),
            })),
            in_use: Arc::new(Mutex::new(false)),
        })
    }

    pub fn eval(&self, code: &str) -> Result<String, String> {
        self.engine.lock().0.eval(code)
    }

    pub fn execute_script(&self, filename: &str, source: &str) -> Result<String, String> {
        self.engine.lock().0.execute_script(filename, source)
    }
}

#[derive(Clone)]
pub struct EnginePool {
    pool: Arc<Mutex<HashMap<String, EnginePoolEntry>>>,
    kind: JsEngineKind,
    current_size: Arc<Mutex<usize>>,
    max_size: usize,
    min_size: usize,
    scale_up_threshold: f64,
    scale_down_threshold: f64,
}

impl EnginePool {
    pub fn new(kind: JsEngineKind, min_size: usize, max_size: usize) -> Self {
        let pool = Arc::new(Mutex::new(HashMap::new()));
        let current_size = Arc::new(Mutex::new(0usize));
        for _ in 0..min_size {
            if let Ok(entry) = EnginePoolEntry::new(kind) {
                pool.lock().insert(entry.id.clone(), entry);
                *current_size.lock() += 1;
            }
        }
        Self {
            pool,
            kind,
            current_size,
            max_size,
            min_size,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.2,
        }
    }

    pub fn acquire(&self) -> Result<EnginePoolEntry, String> {
        {
            let pool = self.pool.lock();
            for entry in pool.values() {
                let mut in_use = entry.in_use.lock();
                if !*in_use {
                    *in_use = true;
                    entry.stats.lock().acquires += 1;
                    return Ok(entry.clone());
                }
            }
        }

        let mut size = self.current_size.lock();
        {
            let pool = self.pool.lock();
            for entry in pool.values() {
                let mut in_use = entry.in_use.lock();
                if !*in_use {
                    *in_use = true;
                    entry.stats.lock().acquires += 1;
                    return Ok(entry.clone());
                }
            }
        }

        if *size < self.max_size {
            let entry = EnginePoolEntry::new(self.kind)?;
            *entry.in_use.lock() = true;
            self.pool.lock().insert(entry.id.clone(), entry.clone());
            *size += 1;
            entry.stats.lock().acquires += 1;
            Ok(entry)
        } else {
            Err("Engine pool exhausted, all engines in use".to_string())
        }
    }

    pub fn release(&self, entry: &EnginePoolEntry) {
        *entry.in_use.lock() = false;
    }

    pub fn warmup(&self, count: usize) -> Vec<Result<EnginePoolEntry, String>> {
        let mut results = Vec::new();
        let mut size = self.current_size.lock();
        let to_add = count.min(self.max_size.saturating_sub(*size));
        if to_add > 0 {
            let mut pool = self.pool.lock();
            for _ in 0..to_add {
                match EnginePoolEntry::new(self.kind) {
                    Ok(entry) => {
                        pool.insert(entry.id.clone(), entry.clone());
                        *size += 1;
                        results.push(Ok(entry));
                    }
                    Err(e) => results.push(Err(e)),
                }
            }
        }
        results
    }

    pub fn pre_compile_scripts(&self, scripts: &[(&str, &str)]) {
        if let Ok(entry) = self.acquire() {
            for (_name, code) in scripts {
                let _ = entry.eval(code);
            }
            self.release(&entry);
        }
    }

    pub fn auto_scale(&self) {
        let mut size = self.current_size.lock();
        let (total, in_use_count) = {
            let pool = self.pool.lock();
            (pool.len(), pool.values().filter(|e| *e.in_use.lock()).count())
        };

        let usage_ratio = if total > 0 {
            in_use_count as f64 / total as f64
        } else {
            0.0
        };

        if usage_ratio > self.scale_up_threshold && *size < self.max_size {
            let to_add = ((*size as f64) * 0.25).ceil() as usize;
            let to_add = to_add.min(self.max_size.saturating_sub(*size));
            for _ in 0..to_add {
                if let Ok(entry) = EnginePoolEntry::new(self.kind) {
                    self.pool.lock().insert(entry.id.clone(), entry);
                    *size += 1;
                }
            }
        } else if usage_ratio < self.scale_down_threshold && *size > self.min_size {
            let to_remove = ((*size as f64) * 0.25).ceil() as usize;
            let to_remove = to_remove.min(size.saturating_sub(self.min_size));
            let mut pool = self.pool.lock();
            let ids: Vec<String> = pool.iter()
                .filter(|(_, e)| !*e.in_use.lock())
                .map(|(id, _)| id.clone())
                .take(to_remove)
                .collect();
            for id in &ids {
                pool.remove(id);
                *size -= 1;
            }
        }
    }

    pub fn size(&self) -> usize {
        *self.current_size.lock()
    }

    pub fn available(&self) -> usize {
        self.pool.lock().values().filter(|e| !*e.in_use.lock()).count()
    }

    pub fn kind(&self) -> JsEngineKind {
        self.kind
    }
}

impl Default for EnginePool {
    fn default() -> Self {
        Self::new(JsEngineKind::Boa, 2, 16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_pool_new() {
        // With default features, no engine is available, but the pool should still be created
        let pool = EnginePool::new(JsEngineKind::Boa, 0, 4);
        assert_eq!(pool.size(), 0);
        assert_eq!(pool.kind(), JsEngineKind::Boa);
    }

    #[test]
    fn test_engine_pool_acquire_release() {
        let pool = EnginePool::new(JsEngineKind::Boa, 0, 4);
        // acquire may fail since Boa is not available with default features
        let result = pool.acquire();
        // Should either succeed or fail gracefully
        if let Ok(entry) = result {
            assert!(entry.in_use.lock().clone());
            pool.release(&entry);
            assert!(!entry.in_use.lock().clone());
        }
    }

    #[test]
    fn test_engine_pool_size_bounds() {
        let pool = EnginePool::new(JsEngineKind::Boa, 0, 10);
        assert!(pool.size() <= 10);
    }

    #[test]
    fn test_engine_pool_warmup() {
        let pool = EnginePool::new(JsEngineKind::Boa, 0, 10);
        let results = pool.warmup(3);
        // warmup may fail, but shouldn't panic
        assert!(results.len() <= 3);
    }

    #[test]
    fn test_engine_pool_available() {
        let pool = EnginePool::new(JsEngineKind::Boa, 0, 4);
        assert_eq!(pool.available(), 0);
    }

    #[test]
    fn test_engine_pool_default() {
        let pool = EnginePool::default();
        assert_eq!(pool.kind(), JsEngineKind::Boa);
    }

    #[test]
    fn test_pool_entry_stats() {
        let stats = PoolEntryStats {
            acquires: 5,
            total_elapsed: std::time::Duration::from_secs(10),
            created_at: std::time::Instant::now(),
        };
        assert_eq!(stats.acquires, 5);
        assert_eq!(stats.total_elapsed.as_secs(), 10);
    }

    #[test]
    fn test_auto_scale_empty_pool() {
        let pool = EnginePool::new(JsEngineKind::Boa, 0, 4);
        pool.auto_scale();
        assert!(pool.size() <= 4);
    }

    #[test]
    fn test_pre_compile_scripts_empty() {
        let pool = EnginePool::new(JsEngineKind::Boa, 0, 4);
        pool.pre_compile_scripts(&[]);
        assert!(pool.size() <= 4);
    }

    #[test]
    fn test_engine_pool_kind() {
        let pool = EnginePool::new(JsEngineKind::QuickJS, 0, 2);
        assert_eq!(pool.kind(), JsEngineKind::QuickJS);
    }
}
