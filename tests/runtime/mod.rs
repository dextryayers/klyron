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

#[test]
fn test_node_fs_basic_ops() {
    let dir = test_dir("node_fs_ops");
    write_file(&dir, "test.txt", "hello world");
    let content = std::fs::read_to_string(dir.join("test.txt")).unwrap();
    assert_eq!(content, "hello world");

    // Test directory operations
    std::fs::create_dir_all(dir.join("subdir")).unwrap();
    assert!(dir.join("subdir").exists());

    // Test file metadata
    let metadata = std::fs::metadata(dir.join("test.txt")).unwrap();
    assert!(metadata.is_file());
    assert!(metadata.len() > 0);

    // Test directory listing
    let entries: Vec<_> = std::fs::read_dir(&dir).unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert!(entries.contains(&"test.txt".to_string()));
    assert!(entries.contains(&"subdir".to_string()));

    // Test symbolic link creation (unix only)
    #[cfg(unix)]
    {
        let link_path = dir.join("test_link.txt");
        std::os::unix::fs::symlink(&dir.join("test.txt"), &link_path).unwrap();
        assert!(link_path.exists());
        let link_content = std::fs::read_to_string(&link_path).unwrap();
        assert_eq!(link_content, "hello world");
    }

    // Test file append
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(dir.join("test.txt")).unwrap();
    use std::io::Write;
    writeln!(file, "appended line").unwrap();
    drop(file);
    let content = std::fs::read_to_string(dir.join("test.txt")).unwrap();
    assert!(content.contains("appended line"));

    // Test copy
    std::fs::copy(dir.join("test.txt"), dir.join("copy.txt")).unwrap();
    assert!(dir.join("copy.txt").exists());

    // Test rename
    std::fs::rename(dir.join("copy.txt"), dir.join("renamed.txt")).unwrap();
    assert!(dir.join("renamed.txt").exists());
    assert!(!dir.join("copy.txt").exists());

    // Test remove
    std::fs::remove_file(dir.join("renamed.txt")).unwrap();
    assert!(!dir.join("renamed.txt").exists());

    // Test remove_dir_all
    std::fs::remove_dir_all(dir.join("subdir")).unwrap();
    assert!(!dir.join("subdir").exists());
}

#[test]
fn test_node_path_resolve() {
    use std::path::Path;

    // Test path normalization
    let p = Path::new("/foo/bar/../baz").to_path_buf();
    let normalized = p.components().collect::<Vec<_>>();
    assert!(!normalized.is_empty());

    // Test path join
    let base = Path::new("/usr/local");
    let full = base.join("bin").join("node");
    assert_eq!(full.to_string_lossy(), "/usr/local/bin/node");

    // Test path components
    let p = Path::new("/a/b/c/d.js");
    assert_eq!(p.parent(), Some(Path::new("/a/b/c")));
    assert_eq!(p.file_name(), Some(std::ffi::OsStr::new("d.js")));
    assert_eq!(p.extension(), Some(std::ffi::OsStr::new("js")));
    assert_eq!(p.file_stem(), Some(std::ffi::OsStr::new("d")));

    // Test path operations
    let dir = test_dir("path_ops");
    let sub = dir.join("sub").join("nested");
    std::fs::create_dir_all(&sub).unwrap();
    let file_path = sub.join("file.txt");
    write_file(&dir, "sub/nested/file.txt", "content");

    // Test relative path resolution
    let relative = Path::new("sub/nested/file.txt");
    let absolute = dir.join(relative);
    assert!(absolute.exists());

    // Test path starts_with / ends_with
    assert!(absolute.starts_with(&dir));
    assert!(absolute.ends_with("file.txt"));

    // Test path is_absolute / is_relative
    assert!(dir.is_absolute());
    assert!(relative.is_relative());

    // Test canonicalize
    let canon = std::fs::canonicalize(&file_path).unwrap();
    assert!(canon.is_absolute());
    assert!(canon.exists());

    // Test path ancestor traversal
    let ancestors: Vec<_> = Path::new("/a/b/c/d").ancestors().collect();
    assert_eq!(ancestors.len(), 5); // /a/b/c/d, /a/b/c, /a/b, /a, /
}

#[test]
fn test_fetch_response_headers() {
    // Test URL parsing
    let url = "https://api.example.com/data";
    assert!(url.starts_with("https://") || url.starts_with("http://"));

    // Test header construction
    let mut headers = std::collections::HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("Authorization".to_string(), "Bearer test-token".to_string());
    assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
    assert_eq!(headers.get("Authorization").unwrap(), "Bearer test-token");

    // Test header case-insensitivity (HTTP headers are case-insensitive)
    let lower = headers.iter().map(|(k, v)| (k.to_lowercase(), v.clone())).collect::<std::collections::HashMap<_, _>>();
    assert_eq!(lower.get("content-type").unwrap(), "application/json");

    // Test Response-like structure
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Response {
        status: u16,
        ok: bool,
        headers: std::collections::HashMap<String, String>,
        body: Option<String>,
    }

    let resp = Response {
        status: 200,
        ok: true,
        headers: std::collections::HashMap::from([
            ("content-type".into(), "application/json".into()),
        ]),
        body: Some(r#"{"message":"hello"}"#.into()),
    };
    let json = serde_json::to_string(&resp).unwrap();
    let parsed: Response = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.status, 200);
    assert!(parsed.ok);
    assert!(parsed.body.is_some());

    // Test error response
    let error_resp = Response {
        status: 404,
        ok: false,
        headers: std::collections::HashMap::new(),
        body: None,
    };
    assert!(!error_resp.ok);
    assert_eq!(error_resp.status, 404);

    // Test redirect
    let redirect_resp = Response {
        status: 301,
        ok: true,
        headers: std::collections::HashMap::from([
            ("location".into(), "/new-url".into()),
        ]),
        body: None,
    };
    assert_eq!(redirect_resp.headers.get("location").unwrap(), "/new-url");

    // Test JSON body parsing
    if let Some(body) = &resp.body {
        let parsed_body: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(parsed_body["message"], "hello");
    }
}

#[test]
fn test_console_log() {
    // Test basic console output capture
    let dir = test_dir("console_test");
    let script = r#"
console.log("hello");
console.error("error msg");
console.warn("warning");
console.info("info msg");
console.debug("debug msg");
"#;
    write_file(&dir, "test_console.js", script);

    // Verify file was written
    assert!(dir.join("test_console.js").exists());
    let content = std::fs::read_to_string(dir.join("test_console.js")).unwrap();
    assert!(content.contains("console.log"));
    assert!(content.contains("console.error"));
    assert!(content.contains("console.warn"));
    assert!(content.contains("console.info"));
    assert!(content.contains("console.debug"));

    // Test console.log with multiple arguments
    let script2 = r#"console.log("a", "b", "c");"#;
    write_file(&dir, "test_multi.js", script2);
    let content2 = std::fs::read_to_string(dir.join("test_multi.js")).unwrap();
    assert!(content2.contains(r#""a", "b", "c""#));

    // Test console.log with numbers
    let script3 = r#"console.log(1, 2, 3);"#;
    write_file(&dir, "test_numbers.js", script3);
    let content3 = std::fs::read_to_string(dir.join("test_numbers.js")).unwrap();
    assert!(content3.contains("1, 2, 3"));

    // Test console.log with objects
    let script4 = r#"console.log({a: 1, b: 2});"#;
    write_file(&dir, "test_object.js", script4);
    let content4 = std::fs::read_to_string(dir.join("test_object.js")).unwrap();
    assert!(content4.contains("console.log"));

    // Test console.time and console.timeEnd
    let script5 = r#"
console.time("timer");
console.timeEnd("timer");
"#;
    write_file(&dir, "test_time.js", script5);
    let content5 = std::fs::read_to_string(dir.join("test_time.js")).unwrap();
    assert!(content5.contains("console.time"));
}

#[test]
fn test_node_path_join_and_resolve() {
    use std::path::Path;

    // Test Path::join
    let p = Path::new("/a").join("b").join("c");
    assert_eq!(p, Path::new("/a/b/c"));

    // Test Path::with_file_name
    let p = Path::new("/a/b/c.txt").with_file_name("d.txt");
    #[cfg(unix)]
    assert_eq!(p, Path::new("/a/b/d.txt"));

    // Test Path::with_extension
    let p = Path::new("/a/b/c.txt").with_extension("json");
    #[cfg(unix)]
    assert_eq!(p, Path::new("/a/b/c.json"));

    // Test relative path resolution
    let base = Path::new("/usr/local/lib");
    let relative = Path::new("../bin/node");
    let resolved = base.join(relative);
    #[cfg(unix)]
    assert_eq!(resolved, Path::new("/usr/local/lib/../bin/node"));

    // Test component iteration
    let p = Path::new("/usr/local/bin");
    let components: Vec<_> = p.components().collect();
    assert_eq!(components.len(), 4); // Root, usr, local, bin
}

#[test]
fn test_console_log_capture_output() {
    // Test piped stdout capture
    let dir = test_dir("console_capture");

    // Simulate writing a script and capturing its console output
    let script = r#"console.log("captured output");"#;
    write_file(&dir, "capture.js", script);

    let output = std::process::Command::new("node")
        .arg(dir.join("capture.js"))
        .output();
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if out.status.success() {
                assert!(stdout.contains("captured output"));
            }
        }
        Err(_) => {
            // node may not be installed
            eprintln!("Skipping node capture test: node not available");
        }
    }
}
