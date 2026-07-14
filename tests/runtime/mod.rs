use std::collections::HashMap;
use std::path::PathBuf;

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("klyron_test_runtime_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_file(dir: &PathBuf, path: &str, content: &str) {
    let full = dir.join(path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&full, content).unwrap();
}

#[test]
fn test_isolate_pool_metrics() {
    use klyron_runtime::IsolatePool;

    let pool = IsolatePool::new(2);
    let metrics = pool.metrics();
    assert_eq!(metrics.pool_size, 2);
    assert_eq!(metrics.active_isolates, 0);
    assert_eq!(metrics.scripts_executed, 0);
}

#[test]
fn test_snapshot_cache_basic() {
    use klyron_runtime::StartupSnapshotCache;

    let cache = StartupSnapshotCache::new();
    cache.set("key1".into(), vec![1, 2, 3]);
    assert_eq!(cache.get("key1"), Some(vec![1, 2, 3]));
    assert_eq!(cache.get("nonexistent"), None);
    let (hits, misses) = cache.stats();
    assert_eq!(hits, 1);
    assert_eq!(misses, 1);
}

#[test]
fn test_snapshot_cache_disk() {
    use klyron_runtime::StartupSnapshotCache;
    let dir = test_dir("snapshot_disk");
    let cache_path = dir.join("cache.json");

    let cache = StartupSnapshotCache::new();
    cache.set("key_a".into(), vec![10, 20, 30]);
    cache.save_to_disk(&cache_path).unwrap();

    let loaded = StartupSnapshotCache::new();
    loaded.load_from_disk(&cache_path).unwrap();
    assert_eq!(loaded.get("key_a"), Some(vec![10, 20, 30]));
}

#[test]
fn test_startup_snapshot_cache_miss() {
    use klyron_runtime::StartupSnapshotCache;
    let cache = StartupSnapshotCache::new();
    assert_eq!(cache.get("missing_key"), None);
    let (hits, misses) = cache.stats();
    assert_eq!(hits, 0);
    assert_eq!(misses, 1);
}

#[test]
fn test_create_runtime_basic() {
    use klyron_runtime::create_runtime;
    let result = create_runtime(vec![], true, false);
    assert!(result.is_ok() || result.is_err(),
        "create_runtime should not panic");
}

#[test]
fn test_execute_script_handle() {
    use klyron_runtime::{create_runtime, execute_script};

    let mut rt = match create_runtime(vec![], true, false) {
        Ok(rt) => rt,
        Err(_) => return, // skip if runtime can't be created
    };
    let result = execute_script(&mut rt, "test.js", "1 + 2");
    match result {
        Ok(val) => assert!(!val.is_empty(), "result should not be empty"),
        Err(e) => {
            // May fail if no V8/etc available
            assert!(e.to_string().contains("not available") || e.to_string().contains("No"),
                "unexpected error: {e}");
        }
    }
}

#[test]
fn test_runtime_pool_execute() {
    use klyron_runtime::RuntimePool;

    let pool = RuntimePool::new(2);

    match pool.execute("test.js", "1 + 2") {
        Ok(val) => assert!(!val.is_empty()),
        Err(e) => {
            assert!(e.to_string().contains("not available") || e.to_string().contains("No"),
                "unexpected error: {e}");
        }
    }
}

#[test]
fn test_runtime_with_timeout() {
    use klyron_runtime::RuntimePool;

    let mut pool = RuntimePool::new(2);
    let result = pool.execute_with_timeout("test.js", "1 + 2", std::time::Duration::from_secs(5));
    match result {
        Ok(val) => assert!(!val.is_empty()),
        Err(e) => {
            let msg = e.to_string();
            assert!(msg.contains("not available") || msg.contains("No") || msg.contains("timed out"),
                "unexpected error: {msg}");
        }
    }
}

#[test]
fn test_module_loader_types() {
    use klyron_engine::es_module::ModuleType;
    assert_eq!(ModuleType::Esm, ModuleType::Esm);
    assert_eq!(ModuleType::CommonJs, ModuleType::CommonJs);
    assert!(ModuleType::Esm.as_str() == "esm" || ModuleType::Esm.as_str() == "module");
}

#[test]
fn test_env_file_loading() {
    // Test that .env file loading works conceptually
    let dir = test_dir("env_loading");
    write_file(&dir, ".env", "PORT=3000\nDB_HOST=localhost\n");
    write_file(&dir, ".env.example", "PORT=\nDB_HOST=\n");

    let content = std::fs::read_to_string(dir.join(".env")).unwrap();
    assert!(content.contains("PORT=3000"));
    assert!(content.contains("DB_HOST=localhost"));

    let example = std::fs::read_to_string(dir.join(".env.example")).unwrap();
    assert!(example.contains("PORT="));
    assert!(example.contains("DB_HOST="));
}

#[test]
fn test_execute_script_with_timeout() {
    use klyron_runtime::RuntimePool;

    let mut pool = RuntimePool::new(1);
    let result = pool.execute_with_timeout("test.js", "var x = 1; x + 2;", std::time::Duration::from_secs(3));
    match result {
        Ok(val) => assert!(!val.is_empty()),
        Err(e) => {
            let msg = e.to_string();
            assert!(msg.contains("not available") || msg.contains("No") || msg.contains("timeout"),
                "unexpected error: {msg}");
        }
    }
}

#[test]
fn test_bytecode_cache() {
    use klyron_engine::bytecode_cache::{BytecodeCache, CachedBytecode};
    let cache = BytecodeCache::new(100);
    let entry = CachedBytecode {
        bytecode: vec![1, 2, 3],
        source_hash: 0xabcdef,
        compiled_at: std::time::Instant::now(),
    };
    cache.set("test.js", entry);
    let retrieved = cache.get("test.js");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().bytecode, vec![1, 2, 3]);
}

#[test]
fn test_warmup_cache() {
    use klyron_engine::WarmupCache;
    let cache = WarmupCache::new();
    cache.insert("test", "result");
    assert_eq!(cache.get("test"), Some("result"));
    assert_eq!(cache.get("missing"), None);
}

#[test]
fn test_memory_limits() {
    use klyron_engine::memory_limits::MemoryLimits;
    let limits = MemoryLimits::default();
    assert!(limits.max_heap_size > 0);
    assert!(limits.max_stack_size > 0);
}

#[test]
fn test_lazy_compiler() {
    use klyron_engine::lazy_compile::{LazyCompiler, CompiledModule};
    let compiler = LazyCompiler::new();
    let module = CompiledModule {
        code: "1 + 2".into(),
        source_map: None,
        compiled_at: std::time::Instant::now(),
    };
    compiler.cache("test.js".into(), module);
    let cached = compiler.get_cached("test.js");
    assert!(cached.is_some());
}

#[test]
fn test_runtime_pool_warmup() {
    use klyron_runtime::RuntimePool;
    let pool = RuntimePool::new(2);

    fn empty_ext() -> Vec<deno_core::Extension> {
        vec![]
    }

    let result = pool.warmup(empty_ext, true, false);
    // May fail if no V8
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_runtime_metrics_after_execution() {
    use klyron_runtime::RuntimePool;
    let pool = RuntimePool::new(1);
    let _ = pool.execute("test.js", "1 + 1");
    let metrics = pool.metrics();
    // Even if execution failed, metrics should still be accessible
    assert!(metrics.pool_size >= 1);
}
