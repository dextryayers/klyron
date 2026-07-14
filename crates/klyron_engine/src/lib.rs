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

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use once_cell::sync::Lazy;

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
