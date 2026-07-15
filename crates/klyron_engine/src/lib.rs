pub mod process;
pub mod traits;
pub mod engine;
pub mod warmup_cache;
pub mod memory_limits;
pub mod sandbox;
pub mod fallback;
pub mod snapshot;
pub mod es_module;
pub mod sourcemap;
pub mod engine_pool;
pub mod bytecode_cache;
pub mod lazy_compile;
pub mod pre_warm;
pub mod profiler;
pub mod cache;
pub mod streaming;
pub mod parallel;

pub use process::{EngineProcess, EngineInput, EngineOutput, FileEntry, find_engine_path};
pub use traits::EngineTrait;
pub use engine::{JsEngineKind, EngineRuntime, JsEngine, JsValue, JsError, BenchResult, benchmark_all_engines, detect_best_engine};
pub use warmup_cache::WarmupCache;
pub use memory_limits::MemoryLimits;
pub use sandbox::{SandboxPool, SandboxContext};
pub use fallback::{FallbackChain, FallbackStrategy};
pub use snapshot::{EngineSnapshot, WarmupSnapshot};
pub use es_module::{ESModuleLoader, ESModule, ModuleType, ModuleLoader};
pub use sourcemap::SourceMap;
pub use engine_pool::{EnginePool, EnginePoolEntry, PoolEntryStats};
pub use bytecode_cache::{BytecodeCache, CachedBytecode};
pub use lazy_compile::{LazyCompiler, CompiledModule};
pub use pre_warm::{EnginePreWarmer, default_pre_warm_scripts};
pub use profiler::{JitProfiler, ProfilingStats, IndividualProfile};
pub use cache::{TwoTierCache, MemoryCache, DiskCache, CacheConfig, CacheEntry};

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use std::time::Instant;

use once_cell::sync::Lazy;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct EngineProfile {
    pub engine_kind: JsEngineKind,
    pub ops_per_sec: f64,
    pub memory_usage_bytes: u64,
    pub gc_count: u64,
    pub warmup_complete: bool,
    pub avg_eval_time_ns: f64,
}

pub static HOT_PATH_CACHE: Lazy<Mutex<HashMap<String, u64>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub static ENGINE_SELECTOR: Lazy<RwLock<AutoSwitcher>> = Lazy::new(|| {
    RwLock::new(AutoSwitcher::new())
});

pub fn record_hot_path(name: &str) {
    if let Ok(mut cache) = HOT_PATH_CACHE.lock() {
        *cache.entry(name.to_string()).or_insert(0) += 1;
    }
}

pub fn get_hot_paths(threshold: u64) -> Vec<String> {
    HOT_PATH_CACHE.lock()
        .ok()
        .map(|cache| {
            cache.iter()
                .filter(|&(_, count)| *count >= threshold)
                .map(|(name, _)| name.clone())
                .collect()
        })
        .unwrap_or_default()
}

pub fn profile_engine(kind: JsEngineKind, iterations: u64) -> Result<EngineProfile, String> {
    let engine = EngineRuntime::new(kind)?;
    let test_code = "1 + 2 + 3";
    let complex_code = r#"
        function fib(n) { return n < 2 ? n : fib(n-1) + fib(n-2); }
        fib(20);
    "#;

    let start = Instant::now();
    for _ in 0..iterations {
        engine.eval(test_code)?;
    }
    let elapsed = start.elapsed();
    let ops_per_sec = if elapsed.as_secs_f64() > 0.0 {
        iterations as f64 / elapsed.as_secs_f64()
    } else {
        0.0
    };

    let bench_start = Instant::now();
    engine.eval(complex_code)?;
    let bench_elapsed = bench_start.elapsed();

    Ok(EngineProfile {
        engine_kind: kind,
        ops_per_sec,
        memory_usage_bytes: 0,
        gc_count: 0,
        warmup_complete: true,
        avg_eval_time_ns: bench_elapsed.as_nanos() as f64,
    })
}

pub fn profile_all_engines(iterations: u64) -> Vec<EngineProfile> {
    let mut profiles = Vec::new();
    for kind in JsEngineKind::all() {
        match profile_engine(kind, iterations) {
            Ok(profile) => profiles.push(profile),
            Err(e) => {
                tracing::warn!("Failed to profile {}: {}", kind, e);
            }
        }
    }
    profiles
}

pub struct AutoSwitcher {
    usage_counts: HashMap<JsEngineKind, u64>,
    avg_latencies: HashMap<JsEngineKind, f64>,
    current: JsEngineKind,
    switch_threshold: u64,
    sample_count: u64,
}

impl AutoSwitcher {
    pub fn new() -> Self {
        Self {
            usage_counts: HashMap::new(),
            avg_latencies: HashMap::new(),
            current: JsEngineKind::Boa,
            switch_threshold: 100,
            sample_count: 0,
        }
    }

    pub fn record_latency(&mut self, kind: JsEngineKind, latency_ns: f64) {
        let count = self.usage_counts.entry(kind).or_insert(0);
        *count += 1;
        self.sample_count += 1;

        let avg = self.avg_latencies.entry(kind).or_insert(0.0);
        *avg = (*avg * (*count - 1) as f64 + latency_ns) / *count as f64;
    }

    pub fn select_engine(&mut self, script_size: usize, is_hot_path: bool) -> JsEngineKind {
        if is_hot_path {
            return JsEngineKind::V8;
        }
        if script_size < 1024 {
            return JsEngineKind::QuickJS;
        }
        if self.sample_count > self.switch_threshold {
            let best = self.avg_latencies.iter()
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(kind, _)| *kind)
                .unwrap_or(JsEngineKind::Boa);
            self.current = best;
            return best;
        }
        self.current
    }

    pub fn current_engine(&self) -> JsEngineKind {
        self.current
    }

    pub fn set_current(&mut self, kind: JsEngineKind) {
        self.current = kind;
    }

    pub fn stats(&self) -> HashMap<JsEngineKind, (u64, f64)> {
        self.usage_counts.iter().map(|(k, c)| {
            (*k, (*c, *self.avg_latencies.get(k).unwrap_or(&0.0)))
        }).collect()
    }
}

pub struct BytecodeCacheV2 {
    inner: Arc<TwoTierCache>,
    profiler: Arc<JitProfiler>,
}

impl BytecodeCacheV2 {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            inner: Arc::new(TwoTierCache::new(config)),
            profiler: Arc::new(JitProfiler::new()),
        }
    }

    pub fn get_or_compile(
        &self,
        key: &str,
        compiler: impl FnOnce() -> Result<Vec<u8>, String>,
    ) -> Result<Vec<u8>, String> {
        if let Some(entry) = self.inner.get(key) {
            self.profiler.record_cache_hit();
            return Ok(entry.value);
        }

        self.profiler.record_cache_miss();
        let start = Instant::now();
        let bytecode = compiler()?;
        let compile_time = start.elapsed();

        self.profiler.record_compile(key, compile_time);
        self.inner.put(key.to_string(), bytecode.clone(), 3600, "bytecode".to_string());

        Ok(bytecode)
    }

    pub fn invalidate(&self, key: &str) {
        self.inner.remove(key);
    }

    pub fn clear(&self) {
        self.inner.clear();
    }

    pub fn profiler(&self) -> &JitProfiler {
        &self.profiler
    }

    pub fn stats(&self) -> (u64, u64) {
        let stats = self.profiler.get_stats();
        (stats.cache_hits, stats.cache_misses)
    }
}

pub struct NanoProcessIsolator {
    engine_path: String,
    timeout: std::time::Duration,
}

impl NanoProcessIsolator {
    pub fn new(engine_path: &str) -> Self {
        Self {
            engine_path: engine_path.to_string(),
            timeout: std::time::Duration::from_secs(30),
        }
    }

    pub fn execute(&self, code: &str) -> Result<EngineOutput, String> {
        let mut process = EngineProcess::spawn(&self.engine_path, &["--eval"])
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        let input = EngineInput {
            action: "eval".to_string(),
            code: Some(code.to_string()),
            args: None,
            filename: None,
            project: None,
            files: None,
        };

        process.communicate_with_timeout(&input, self.timeout)
            .map_err(|e| format!("Process execution failed: {}", e))
    }

    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

pub struct StreamingCompiler {
    chunks: Mutex<Vec<String>>,
    compiled: Mutex<bool>,
    engine: Mutex<Option<EngineRuntime>>,
}

impl StreamingCompiler {
    pub fn new(kind: JsEngineKind) -> Result<Self, String> {
        Ok(Self {
            chunks: Mutex::new(Vec::new()),
            compiled: Mutex::new(false),
            engine: Mutex::new(Some(EngineRuntime::new(kind)?)),
        })
    }

    pub fn feed(&self, chunk: &str) {
        let mut chunks = self.chunks.lock().unwrap();
        chunks.push(chunk.to_string());
        *self.compiled.lock().unwrap() = false;
    }

    pub fn compile(&self) -> Result<String, String> {
        let chunks = self.chunks.lock().unwrap();
        let full_code = chunks.join("\n");
        let engine = self.engine.lock().unwrap();
        if let Some(ref eng) = *engine {
            let result = eng.eval(&full_code)?;
            *self.compiled.lock().unwrap() = true;
            Ok(result)
        } else {
            Err("Engine not available".to_string())
        }
    }

    pub fn reset(&self) {
        self.chunks.lock().unwrap().clear();
        *self.compiled.lock().unwrap() = false;
    }

    pub fn is_compiled(&self) -> bool {
        *self.compiled.lock().unwrap()
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.lock().unwrap().len()
    }
}

pub struct MmapFileCache {
    cache: Mutex<HashMap<String, Vec<u8>>>,
    mmap_threshold: usize,
}

impl MmapFileCache {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            mmap_threshold: 1024 * 1024,
        }
    }

    pub fn load_file(&self, path: &str) -> Result<Vec<u8>, String> {
        {
            let cache = self.cache.lock().map_err(|e| e.to_string())?;
            if let Some(data) = cache.get(path) {
                return Ok(data.clone());
            }
        }

        let data = std::fs::read(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;

        if data.len() >= self.mmap_threshold {
            tracing::debug!("mmap-cached {} ({} bytes)", path, data.len());
        }

        let mut cache = self.cache.lock().map_err(|e| e.to_string())?;
        cache.insert(path.to_string(), data.clone());
        Ok(data)
    }

    pub fn remove(&self, path: &str) {
        let mut cache = self.cache.lock().unwrap();
        cache.remove(path);
    }

    pub fn clear(&self) {
        self.cache.lock().unwrap().clear();
    }

    pub fn len(&self) -> usize {
        self.cache.lock().map(|c| c.len()).unwrap_or(0)
    }
}

pub fn pre_warm_common_modules(cache: &BytecodeCacheV2, kind: JsEngineKind) {
    let common_scripts = default_pre_warm_scripts();
    for (name, code) in common_scripts {
        let key = format!("prewarm:{}:{}:{}", kind.name(), name, blake3::hash(code.as_bytes()).to_hex());
        let _ = cache.get_or_compile(&key, || {
            let engine = EngineRuntime::new(kind)?;
            engine.eval(code)?;
            Ok(code.as_bytes().to_vec())
        });
    }
    tracing::info!("Pre-warmed {} common modules for {}", common_scripts.len(), kind);
}
