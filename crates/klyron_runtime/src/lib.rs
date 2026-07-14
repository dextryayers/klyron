//! klyron_runtime — V8 isolate pool, snapshot caching, concurrent execution

pub use klyron_core::*;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

// ── RuntimeMetrics ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeMetrics {
    pub scripts_executed: u64,
    pub total_execution_time_ms: u64,
    pub active_isolates: usize,
    pub pool_size: usize,
    pub snapshot_cache_hits: u64,
    pub snapshot_cache_misses: u64,
}

// ── Send-safe Runtime wrapper ─────────────────────────────────────────────

pub struct SendRuntime(Runtime);

unsafe impl Send for SendRuntime {}

impl SendRuntime {
    #[inline]
    pub fn new(extensions: Vec<deno_core::Extension>, enable_typescript: bool, async_: bool) -> Result<Self> {
        let rt = Runtime::builder()
            .enable_typescript(enable_typescript)
            .async_(async_)
            .extensions(extensions)
            .build()?;
        Ok(Self(rt))
    }

    #[inline]
    pub fn execute_script(&mut self, name: &str, source: &str) -> Result<String> {
        self.0.execute_script(name, source)
    }
}

// ── V8 Isolate Pool ───────────────────────────────────────────────────────

static ISOLATE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct IsolatePool {
    pool: Mutex<Vec<SendRuntime>>,
    metrics: Arc<Mutex<RuntimeMetrics>>,
    max_size: usize,
}

impl IsolatePool {
    pub fn new(size: usize) -> Self {
        Self {
            pool: Mutex::new(Vec::with_capacity(size)),
            metrics: Arc::new(Mutex::new(RuntimeMetrics {
                pool_size: size,
                ..Default::default()
            })),
            max_size: size,
        }
    }

    pub fn warmup<F>(&self, factory: F, enable_typescript: bool, async_: bool) -> Result<()>
    where
        F: Fn() -> Vec<deno_core::Extension>,
    {
        let mut pool = self.pool.lock();
        while pool.len() < self.max_size {
            let rt = SendRuntime::new(factory(), enable_typescript, async_)?;
            pool.push(rt);
            ISOLATE_COUNTER.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }

    pub fn acquire(&self) -> RuntimeHandle {
        let mut pool = self.pool.lock();
        let rt = pool.pop();
        {
            let mut m = self.metrics.lock();
            m.active_isolates = pool.len();
        }
        RuntimeHandle {
            runtime: rt,
            start_time: Instant::now(),
            metrics: self.metrics.clone(),
        }
    }

    pub fn release(&self, rt: SendRuntime) {
        let mut pool = self.pool.lock();
        if pool.len() < self.max_size {
            pool.push(rt);
        }
    }

    #[inline]
    pub fn metrics(&self) -> RuntimeMetrics {
        self.metrics.lock().clone()
    }
}

pub struct RuntimeHandle {
    runtime: Option<SendRuntime>,
    start_time: Instant,
    metrics: Arc<Mutex<RuntimeMetrics>>,
}

impl RuntimeHandle {
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.runtime.is_some()
    }

    #[inline]
    pub fn execute_script(&mut self, name: &str, source: &str) -> Result<String> {
        let rt = self.runtime.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Runtime not available"))?;
        rt.execute_script(name, source)
    }

    pub fn execute_with_timeout(&mut self, name: &str, source: &str, timeout: Duration) -> Result<String> {
        let name_owned = name.to_string();
        let source_owned = source.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        let _handle = std::thread::spawn(move || {
            let mut local_rt = SendRuntime::new(vec![], true, false)
                .expect("Failed to create timeout runtime");
            let result = local_rt.execute_script(&name_owned, &source_owned);
            let _ = tx.send(result);
        });
        match rx.recv_timeout(timeout) {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(e)) => Err(e),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                anyhow::bail!("Execution timed out after {timeout:?}")
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                anyhow::bail!("Execution thread panicked")
            }
        }
    }
}

impl Drop for RuntimeHandle {
    fn drop(&mut self) {
        let elapsed = self.start_time.elapsed();
        drop(self.runtime.take());
        let mut m = self.metrics.lock();
        m.scripts_executed += 1;
        m.total_execution_time_ms += elapsed.as_millis() as u64;
    }
}

// ── Startup Snapshot Caching ──────────────────────────────────────────────

pub struct StartupSnapshotCache {
    store: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl StartupSnapshotCache {
    #[inline]
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    #[inline]
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let store = self.store.lock();
        let result = store.get(key).cloned();
        if result.is_some() {
            self.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
        }
        result
    }

    #[inline]
    pub fn set(&self, key: String, snapshot: Vec<u8>) {
        let mut store = self.store.lock();
        store.insert(key, snapshot);
    }

    pub fn save_to_disk(&self, path: &std::path::Path) -> Result<()> {
        let store = self.store.lock();
        let json = serde_json::to_string(&*store)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_disk(&self, path: &std::path::Path) -> Result<()> {
        let json = std::fs::read_to_string(path)?;
        let store: HashMap<String, Vec<u8>> = serde_json::from_str(&json)?;
        *self.store.lock() = store;
        Ok(())
    }

    #[inline]
    pub fn stats(&self) -> (u64, u64) {
        (self.hits.load(Ordering::Relaxed), self.misses.load(Ordering::Relaxed))
    }
}

// ── RuntimePool (concurrent script execution) ────────────────────────────

pub struct RuntimePool {
    isolates: IsolatePool,
    snapshot_cache: StartupSnapshotCache,
    metrics: Arc<Mutex<RuntimeMetrics>>,
}

impl RuntimePool {
    pub fn new(pool_size: usize) -> Self {
        Self {
            isolates: IsolatePool::new(pool_size),
            snapshot_cache: StartupSnapshotCache::new(),
            metrics: Arc::new(Mutex::new(RuntimeMetrics::default())),
        }
    }

    pub fn execute(&self, name: &str, source: &str) -> Result<String> {
        let mut handle = self.isolates.acquire();
        let result = handle.execute_script(name, source);
        let mut m = self.metrics.lock();
        m.scripts_executed += 1;
        result
    }

    pub fn execute_with_timeout(&mut self, name: &str, source: &str, timeout: Duration) -> Result<String> {
        let mut handle = self.isolates.acquire();
        let start = Instant::now();
        let result = handle.execute_with_timeout(name, source, timeout);
        let mut m = self.metrics.lock();
        m.scripts_executed += 1;
        m.total_execution_time_ms += start.elapsed().as_millis() as u64;
        result
    }

    #[inline]
    pub fn metrics(&self) -> RuntimeMetrics {
        let base = self.metrics.lock().clone();
        let (hits, misses) = self.snapshot_cache.stats();
        RuntimeMetrics {
            snapshot_cache_hits: hits,
            snapshot_cache_misses: misses,
            ..base
        }
    }

    #[inline]
    pub fn snapshot_cache(&self) -> &StartupSnapshotCache {
        &self.snapshot_cache
    }

    #[inline]
    pub fn warmup<F>(&self, factory: F, enable_typescript: bool, async_: bool) -> Result<()>
    where
        F: Fn() -> Vec<deno_core::Extension>,
    {
        self.isolates.warmup(factory, enable_typescript, async_)
    }
}

// ── Inline hot-path helpers ───────────────────────────────────────────────

#[inline]
pub fn create_runtime(extensions: Vec<deno_core::Extension>, enable_typescript: bool, async_: bool) -> Result<Runtime> {
    Runtime::builder()
        .enable_typescript(enable_typescript)
        .async_(async_)
        .extensions(extensions)
        .build()
}

#[inline]
pub fn execute_script(runtime: &mut Runtime, name: &str, source: &str) -> Result<String> {
    runtime.execute_script(name, source)
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_runtime_basic() {
        let rt = create_runtime(vec![], true, false);
        assert!(rt.is_ok());
    }

    #[test]
    fn test_isolate_pool_metrics() {
        let pool = IsolatePool::new(4);
        assert_eq!(pool.metrics().pool_size, 4);
    }

    #[test]
    fn test_snapshot_cache() {
        let cache = StartupSnapshotCache::new();
        cache.set("test".into(), vec![1, 2, 3]);
        assert_eq!(cache.get("test"), Some(vec![1, 2, 3]));
        assert_eq!(cache.get("nonexistent"), None);
        let (hits, misses) = cache.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
    }
}
