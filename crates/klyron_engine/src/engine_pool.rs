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
        let pool = self.pool.lock();
        for entry in pool.values() {
            let mut in_use = entry.in_use.lock();
            if !*in_use {
                *in_use = true;
                entry.stats.lock().acquires += 1;
                return Ok(entry.clone());
            }
        }
        drop(pool);

        let mut size = self.current_size.lock();
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
        for _ in 0..to_add {
            match EnginePoolEntry::new(self.kind) {
                Ok(entry) => {
                    self.pool.lock().insert(entry.id.clone(), entry.clone());
                    *size += 1;
                    results.push(Ok(entry));
                }
                Err(e) => results.push(Err(e)),
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
        let pool = self.pool.lock();
        let total = pool.len();
        let in_use_count = pool.values().filter(|e| *e.in_use.lock()).count();
        drop(pool);

        let usage_ratio = if total > 0 {
            in_use_count as f64 / total as f64
        } else {
            0.0
        };

        let mut size = self.current_size.lock();
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
