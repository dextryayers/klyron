//! Klyron JavaScript Engine Abstraction
//!
//! Provides a unified interface over multiple JavaScript/TypeScript engines:
//! - **Boa** — Rust-native JS engine
//! - **QuickJS** — Lightweight embeddable engine
//! - **JavaScriptCore** (JSC) — Apple's high-performance engine
//! - **V8** — Google's high-performance engine
//!
//! ## Features
//!
//! - `detect_best_engine()` — Auto-detect the optimal engine on the host system
//! - `benchmark_all_engines()` — Run benchmarks to determine fastest engine
//! - Engine pooling, sandboxing, snapshots, warmup caching
//! - Polyglot script execution (JS, Python, Ruby, PHP via shell engines)
//!
//! ## Example
//!
//! ```rust,ignore
//! use klyron_engine::{EngineRuntime, JsEngineKind, detect_best_engine};
//!
//! let engine = EngineRuntime::new(JsEngineKind::QuickJs).unwrap();
//! let result = engine.eval("1 + 2").unwrap();
//! ```

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
pub mod script_classifier;
pub mod polyglot;

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
pub use script_classifier::{ScriptFeatures, ScriptClassifier, ExponentialMovingAverage, predict_engine};

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
    v8_latency: ExponentialMovingAverage,
    boa_latency: ExponentialMovingAverage,
    quickjs_latency: ExponentialMovingAverage,
    jsc_latency: ExponentialMovingAverage,
    script_classifier: ScriptClassifier,
    min_samples: usize,
    #[allow(dead_code)]
    switch_threshold: f64,
    hot_path_threshold: u64,
    hot_path_cache: HashMap<String, u64>,
    sample_count: usize,
}

impl AutoSwitcher {
    pub fn new() -> Self {
        Self {
            v8_latency: ExponentialMovingAverage::new(0.3),
            boa_latency: ExponentialMovingAverage::new(0.3),
            quickjs_latency: ExponentialMovingAverage::new(0.3),
            jsc_latency: ExponentialMovingAverage::new(0.3),
            script_classifier: ScriptClassifier::new(),
            min_samples: 5,
            switch_threshold: 1.5,
            hot_path_threshold: 100u64,
            hot_path_cache: HashMap::new(),
            sample_count: 0,
        }
    }

    pub fn select(&mut self, script: &str) -> JsEngineKind {
        // 1. Check if script is on hot path (same script >= 100x)
        if self.is_hot_path(script) {
            return JsEngineKind::V8;
        }

        // 2. Classify script characteristics
        let features = self.script_classifier.extract(script);

        // 3. Predict best engine using decision tree
        self.predict(&features)
    }

    fn is_hot_path(&mut self, script: &str) -> bool {
        let count = self.hot_path_cache.entry(script.to_string()).or_insert(0);
        *count += 1;
        *count >= self.hot_path_threshold
    }

    fn predict(&self, features: &ScriptFeatures) -> JsEngineKind {
        match features {
            f if f.expected_runtime_ms < 5.0 => JsEngineKind::QuickJS,
            f if f.has_loops && f.expected_runtime_ms > 100.0 => JsEngineKind::V8,
            f if f.has_async && f.expected_runtime_ms < 50.0 => JsEngineKind::Boa,
            f if f.import_count > 50 => JsEngineKind::JSC,
            _ => JsEngineKind::V8,
        }
    }

    pub fn record_latency(&mut self, kind: JsEngineKind, latency_ms: f64) {
        self.sample_count += 1;
        let ema = match kind {
            JsEngineKind::V8 => &mut self.v8_latency,
            JsEngineKind::Boa => &mut self.boa_latency,
            JsEngineKind::QuickJS => &mut self.quickjs_latency,
            JsEngineKind::JSC => &mut self.jsc_latency,
        };
        ema.update(latency_ms);
    }

    pub fn best_engine(&self) -> Option<JsEngineKind> {
        if self.sample_count < self.min_samples {
            return None;
        }

        let latencies = [
            (JsEngineKind::V8, self.v8_latency.value()),
            (JsEngineKind::Boa, self.boa_latency.value()),
            (JsEngineKind::QuickJS, self.quickjs_latency.value()),
            (JsEngineKind::JSC, self.jsc_latency.value()),
        ];

        latencies.iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(kind, _)| *kind)
    }

    pub fn stats(&self) -> HashMap<JsEngineKind, f64> {
        let mut m = HashMap::new();
        m.insert(JsEngineKind::V8, self.v8_latency.value());
        m.insert(JsEngineKind::Boa, self.boa_latency.value());
        m.insert(JsEngineKind::QuickJS, self.quickjs_latency.value());
        m.insert(JsEngineKind::JSC, self.jsc_latency.value());
        m
    }

    pub fn latencies(&self) -> HashMap<JsEngineKind, f64> {
        self.stats()
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
    let script_count = common_scripts.len();
    for (name, code) in &common_scripts {
        let key = format!("prewarm:{}:{}:{}", kind.name(), name, blake3::hash(code.as_bytes()).to_hex());
        let code_owned = code.to_string();
        let _ = cache.get_or_compile(&key, || {
            let engine = EngineRuntime::new(kind)?;
            engine.eval(&code_owned)?;
            Ok(code_owned.as_bytes().to_vec())
        });
    }
    tracing::info!("Pre-warmed {} common modules for {}", script_count, kind);
}
