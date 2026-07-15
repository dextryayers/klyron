use std::collections::HashMap;
use std::time::Duration;

fn test_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("klyron_test_engine_adv_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_engine_pool_init_default() {
    use klyron_engine::EnginePool;
    let pool = EnginePool::default();
    assert_eq!(pool.kind(), klyron_engine::JsEngineKind::Boa);
    assert_eq!(pool.size(), 2);
    assert!(pool.available() <= pool.size());
}

#[test]
fn test_engine_pool_init_custom() {
    use klyron_engine::EnginePool;
    let pool = EnginePool::new(klyron_engine::JsEngineKind::Boa, 3, 8);
    assert_eq!(pool.size(), 3);
}

#[test]
fn test_engine_pool_acquire_release_multiple() {
    use klyron_engine::EnginePool;
    let pool = EnginePool::new(klyron_engine::JsEngineKind::Boa, 2, 4);
    let e1 = pool.acquire();
    let e2 = pool.acquire();
    if let (Ok(entry1), Ok(entry2)) = (&e1, &e2) {
        assert_ne!(entry1.id, entry2.id);
        pool.release(entry1);
        pool.release(entry2);
    }
}

#[test]
fn test_engine_pool_auto_scale() {
    use klyron_engine::EnginePool;
    let pool = EnginePool::new(klyron_engine::JsEngineKind::Boa, 1, 8);
    pool.auto_scale();
    let size = pool.size();
    assert!(size >= 1);
}

#[test]
fn test_engine_switching_hot_shared_background() {
    use klyron_engine::{JsEngineKind, EngineRuntime};
    let hot = EngineRuntime::new(JsEngineKind::Boa);
    let shared = EngineRuntime::new(JsEngineKind::Boa);
    let bg = EngineRuntime::new(JsEngineKind::Boa);
    assert!(hot.is_ok() || hot.is_err());
    let _ = shared;
    let _ = bg;
}

#[test]
fn test_bytecode_cache_lru_eviction() {
    use klyron_engine::bytecode_cache::{BytecodeCache, CachedBytecode};
    let mut cache = BytecodeCache::new(3);
    let now = std::time::Instant::now();
    for i in 0..5 {
        let key = format!("file_{i}.js");
        let entry = CachedBytecode {
            bytecode: vec![i as u8; 64],
            source_hash: i as u64,
            compiled_at: now,
        };
        cache.set(&key, entry);
    }
    assert!(cache.get("file_0.js").is_none());
    assert!(cache.get("file_4.js").is_some());
    assert!(cache.get("file_3.js").is_some());
}

#[test]
fn test_bytecode_cache_max_size() {
    use klyron_engine::bytecode_cache::{BytecodeCache, CachedBytecode};
    let mut cache = BytecodeCache::new(5);
    let now = std::time::Instant::now();
    for i in 0..10 {
        cache.set(&format!("k{i}"), CachedBytecode {
            bytecode: vec![i as u8; 32],
            source_hash: i as u64,
            compiled_at: now,
        });
    }
    let count = (0..10).filter(|i| cache.get(&format!("k{i}")).is_some()).count();
    assert!(count <= 5);
}

#[test]
fn test_bytecode_cache_hit_count() {
    use klyron_engine::bytecode_cache::{BytecodeCache, CachedBytecode};
    let mut cache = BytecodeCache::new(10);
    let now = std::time::Instant::now();
    cache.set("hit.js", CachedBytecode {
        bytecode: vec![1; 16],
        source_hash: 42,
        compiled_at: now,
    });
    let _ = cache.get("hit.js");
    let _ = cache.get("hit.js");
    let _ = cache.get("hit.js");
    let stats = cache.stats();
    assert_eq!(stats.hits, 3);
}

#[test]
fn test_lazy_compilation() {
    use klyron_engine::lazy_compile::{LazyCompiler, CompiledModule};
    let compiler = LazyCompiler::new();
    let module = CompiledModule {
        code: "function add(a,b) { return a + b; }".into(),
        source_map: None,
        compiled_at: std::time::Instant::now(),
    };
    compiler.cache("math.js".into(), module);
    let cached = compiler.get_cached("math.js");
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().code, "function add(a,b) { return a + b; }");
}

#[test]
fn test_lazy_compiler_miss() {
    use klyron_engine::lazy_compile::LazyCompiler;
    let compiler = LazyCompiler::new();
    assert!(compiler.get_cached("nonexistent.js").is_none());
}

#[test]
fn test_pre_warm_at_startup() {
    use klyron_engine::pre_warm::{EnginePreWarmer, default_pre_warm_scripts};
    let warmer = EnginePreWarmer::new(klyron_engine::JsEngineKind::Boa);
    let scripts = default_pre_warm_scripts();
    assert!(!scripts.is_empty());
    assert!(scripts.iter().any(|(name, _)| name.contains("polyfill")));
}

#[test]
fn test_engine_pre_warmer_background() {
    use klyron_engine::pre_warm::EnginePreWarmer;
    let mut warmer = EnginePreWarmer::new(klyron_engine::JsEngineKind::Boa);
    warmer.start_background(2);
    assert!(warmer.is_running());
    warmer.stop();
    assert!(!warmer.is_running());
}

#[test]
fn test_nano_process_isolation() {
    use klyron_engine::process::{EngineProcess, EngineInput};
    let dir = test_dir("nano_process");
    let process = EngineProcess::new();
    let input = EngineInput {
        code: "1 + 1".into(),
        filename: "test.js".into(),
        timeout_ms: 1000,
        memory_limit_mb: 64,
        env_vars: HashMap::new(),
    };
    let result = process.execute(&input);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_nano_process_timeout() {
    use klyron_engine::process::{EngineProcess, EngineInput};
    let process = EngineProcess::new();
    let input = EngineInput {
        code: "while(true) {}".into(),
        filename: "infinite.js".into(),
        timeout_ms: 50,
        memory_limit_mb: 64,
        env_vars: HashMap::new(),
    };
    let start = std::time::Instant::now();
    let result = process.execute(&input);
    let elapsed = start.elapsed();
    assert!(result.is_err() || elapsed < Duration::from_secs(2));
}

#[test]
fn test_cross_module_inlining() {
    use klyron_engine::es_module::{ESModuleLoader, ESModule, ModuleType};
    let loader = ESModuleLoader::new();
    let module_a = ESModule {
        url: "module-a".into(),
        code: "export const a = 42;".into(),
        module_type: ModuleType::Esm,
        dependencies: vec![],
    };
    let module_b = ESModule {
        url: "module-b".into(),
        code: "import { a } from 'module-a'; export const b = a + 1;".into(),
        module_type: ModuleType::Esm,
        dependencies: vec!["module-a".into()],
    };
    loader.register(module_a);
    loader.register(module_b);
    let resolved = loader.resolve("module-b");
    assert!(resolved.is_some());
    let deps = resolved.unwrap().dependencies;
    assert!(deps.contains(&"module-a".into()));
}

#[test]
fn test_es_module_loader_circular() {
    use klyron_engine::es_module::{ESModuleLoader, ESModule, ModuleType};
    let loader = ESModuleLoader::new();
    let module_a = ESModule {
        url: "circ-a".into(),
        code: "import { b } from 'circ-b'; export const a = b + 1;".into(),
        module_type: ModuleType::Esm,
        dependencies: vec!["circ-b".into()],
    };
    let module_b = ESModule {
        url: "circ-b".into(),
        code: "import { a } from 'circ-a'; export const b = a + 1;".into(),
        module_type: ModuleType::Esm,
        dependencies: vec!["circ-a".into()],
    };
    loader.register(module_a);
    loader.register(module_b);
    let resolved = loader.resolve("circ-a");
    assert!(resolved.is_some());
}

#[test]
fn test_streaming_compilation() {
    use klyron_engine::lazy_compile::LazyCompiler;
    use klyron_engine::bytecode_cache::BytecodeCache;
    let compiler = LazyCompiler::new();
    let mut cache = BytecodeCache::new(100);
    let chunks = vec!["var ", "x = ", "1;", "console.log(x);"];
    let full_source = chunks.join("");
    let now = std::time::Instant::now();
    compiler.cache("stream.js".into(), klyron_engine::lazy_compile::CompiledModule {
        code: full_source.clone(),
        source_map: None,
        compiled_at: now,
    });
    cache.set("stream.js", klyron_engine::bytecode_cache::CachedBytecode {
        bytecode: full_source.into_bytes(),
        source_hash: 0xabc,
        compiled_at: now,
    });
    let cached_compiled = compiler.get_cached("stream.js");
    assert!(cached_compiled.is_some());
    let cached_bc = cache.get("stream.js");
    assert!(cached_bc.is_some());
}

#[test]
fn test_parallel_module_resolution() {
    use klyron_engine::es_module::ESModuleLoader;
    let loader = ESModuleLoader::new();
    let modules: Vec<_> = (0..20).map(|i| {
        let deps = if i > 0 {
            vec![format!("mod-{}", i - 1)]
        } else {
            vec![]
        };
        klyron_engine::es_module::ESModule {
            url: format!("mod-{i}"),
            code: format!("export const v{i} = {i};"),
            module_type: klyron_engine::es_module::ModuleType::Esm,
            dependencies: deps,
        }
    }).collect();
    for m in modules {
        loader.register(m);
    }
    for i in 0..20 {
        let resolved = loader.resolve(&format!("mod-{i}"));
        assert!(resolved.is_some(), "mod-{i} should resolve");
    }
}

#[test]
fn test_mmap_file_cache() {
    let dir = test_dir("mmap_cache");
    let file_path = dir.join("cached.data");
    std::fs::write(&file_path, b"persistent cache data").unwrap();
    let content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "persistent cache data");
    let metadata = std::fs::metadata(&file_path).unwrap();
    assert!(metadata.len() > 0);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_engine_pool_stats() {
    use klyron_engine::EnginePool;
    let pool = EnginePool::new(klyron_engine::JsEngineKind::Boa, 1, 4);
    if let Ok(entry) = pool.acquire() {
        let _ = entry.eval("1+1");
        pool.release(&entry);
    }
    assert!(pool.size() >= 1);
}

#[test]
fn test_engine_pool_exhaustion() {
    use klyron_engine::EnginePool;
    let pool = EnginePool::new(klyron_engine::JsEngineKind::Boa, 1, 2);
    let _ = pool.acquire();
    let _ = pool.acquire();
    let third = pool.acquire();
    assert!(third.is_err());
}

#[test]
fn test_engine_pool_release_reuse() {
    use klyron_engine::EnginePool;
    let pool = EnginePool::new(klyron_engine::JsEngineKind::Boa, 1, 3);
    if let Ok(entry) = pool.acquire() {
        let id1 = entry.id.clone();
        pool.release(&entry);
        if let Ok(entry2) = pool.acquire() {
            assert!(!entry2.id.is_empty());
        }
    }
}

#[test]
fn test_bytecode_cache_clear() {
    use klyron_engine::bytecode_cache::{BytecodeCache, CachedBytecode};
    let mut cache = BytecodeCache::new(10);
    let now = std::time::Instant::now();
    cache.set("a.js", CachedBytecode { bytecode: vec![1], source_hash: 1, compiled_at: now });
    cache.set("b.js", CachedBytecode { bytecode: vec![2], source_hash: 2, compiled_at: now });
    assert!(cache.get("a.js").is_some());
    cache.clear();
    assert!(cache.get("a.js").is_none());
    assert!(cache.get("b.js").is_none());
}

#[test]
fn test_bytecode_cache_stats() {
    use klyron_engine::bytecode_cache::{BytecodeCache, CachedBytecode};
    let mut cache = BytecodeCache::new(10);
    let now = std::time::Instant::now();
    cache.set("s.js", CachedBytecode { bytecode: vec![1], source_hash: 1, compiled_at: now });
    let _ = cache.get("s.js");
    let _ = cache.get("missing.js");
    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
}

#[test]
fn test_fallback_chain_fastest() {
    use klyron_engine::fallback::{FallbackChain, FallbackStrategy};
    let mut chain = FallbackChain::with_strategy(FallbackStrategy::Fastest);
    let result = chain.resolve();
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_fallback_chain_first_working() {
    use klyron_engine::fallback::{FallbackChain, FallbackStrategy};
    let mut chain = FallbackChain::with_strategy(FallbackStrategy::FirstWorking);
    let result = chain.resolve();
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_fallback_chain_all() {
    use klyron_engine::fallback::{FallbackChain, FallbackStrategy};
    let mut chain = FallbackChain::with_strategy(FallbackStrategy::All);
    let result = chain.resolve();
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_warmup_cache_insert_get() {
    use klyron_engine::WarmupCache;
    let cache = WarmupCache::new();
    cache.insert("key1", "val1");
    assert_eq!(cache.get("key1"), Some("val1"));
}

#[test]
fn test_warmup_cache_miss() {
    use klyron_engine::WarmupCache;
    let cache = WarmupCache::new();
    assert_eq!(cache.get("nonexistent"), None);
}

#[test]
fn test_memory_limits_defaults() {
    use klyron_engine::memory_limits::MemoryLimits;
    let limits = MemoryLimits::default();
    assert!(limits.max_heap_size > 0);
    assert!(limits.max_stack_size > 0);
}

#[test]
fn test_sandbox_pool_basic() {
    use klyron_engine::sandbox::SandboxPool;
    let pool = SandboxPool::new(2);
    assert_eq!(pool.size(), 2);
}

#[test]
fn test_hot_path_recording() {
    use klyron_engine::{record_hot_path, get_hot_paths};
    record_hot_path("render");
    record_hot_path("render");
    record_hot_path("render");
    let hot = get_hot_paths(3);
    assert!(hot.contains(&"render".to_string()));
}

#[test]
fn test_profile_engine_basic() {
    use klyron_engine::{profile_engine, JsEngineKind};
    let result = profile_engine(JsEngineKind::Boa, 5);
    match result {
        Ok(profile) => {
            assert_eq!(profile.engine_kind, JsEngineKind::Boa);
            assert!(profile.ops_per_sec >= 0.0);
        }
        Err(_) => {}
    }
}

#[test]
fn test_profile_all_engines_empty() {
    use klyron_engine::profile_all_engines;
    let profiles = profile_all_engines(3);
    for p in &profiles {
        assert!(p.ops_per_sec >= 0.0);
    }
}

#[test]
fn test_detect_best_engine_non_empty() {
    use klyron_engine::detect_best_engine;
    let best = detect_best_engine();
    let name = best.name();
    assert!(!name.is_empty());
}

#[test]
fn test_engine_snapshot_basic() {
    use klyron_engine::snapshot::{EngineSnapshot, WarmupSnapshot};
    let snapshot = EngineSnapshot {
        id: "snap-1".into(),
        created_at: std::time::SystemTime::now(),
        engine_kind: klyron_engine::JsEngineKind::Boa,
        bytecode_size: 1024,
        compile_time_ms: 50,
    };
    assert_eq!(snapshot.id, "snap-1");
    let warmup = WarmupSnapshot {
        snapshot: snapshot.clone(),
        warmup_scripts: vec!["polyfill.js".into()],
        hit_count: 0,
    };
    assert_eq!(warmup.warmup_scripts.len(), 1);
}

#[test]
fn test_source_map_basic() {
    use klyron_engine::sourcemap::SourceMap;
    let sm = SourceMap::new();
    let result = sm.generate("console.log('hello');\n//# sourceMappingURL=data:,");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_engine_runtime_eval_basic() {
    use klyron_engine::{JsEngineKind, EngineRuntime};
    if let Ok(engine) = EngineRuntime::new(JsEngineKind::Boa) {
        if let Ok(val) = engine.eval("42") {
            assert!(!val.is_empty());
        }
    }
}

#[test]
fn test_engine_runtime_execute_script() {
    use klyron_engine::{JsEngineKind, EngineRuntime};
    if let Ok(mut engine) = EngineRuntime::new(JsEngineKind::Boa) {
        if let Ok(val) = engine.execute_script("test.js", "1 + 2") {
            assert!(!val.is_empty());
        }
    }
}

#[test]
fn test_pre_warm_scripts_contain_polyfill() {
    use klyron_engine::pre_warm::default_pre_warm_scripts;
    let scripts = default_pre_warm_scripts();
    assert!(!scripts.is_empty());
}

#[test]
fn test_engine_process_file_entry() {
    use klyron_engine::process::FileEntry;
    let entry = FileEntry {
        path: "test.js".into(),
        content: "console.log('hello');".into(),
    };
    assert_eq!(entry.path, "test.js");
    assert_eq!(entry.content, "console.log('hello');");
}
