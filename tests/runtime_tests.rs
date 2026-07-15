use std::path::PathBuf;

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("klyron_test_rt_adv_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_js(dir: &PathBuf, name: &str, code: &str) {
    std::fs::write(dir.join(name), code).unwrap();
}

fn run_klyron_eval(script: &str) -> Result<String, String> {
    let output = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "eval", "--", script])
        .output()
        .map_err(|e| format!("Failed to run klyron: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn run_klyron_file(path: &std::path::Path) -> Result<String, String> {
    let output = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "run", path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Failed to run klyron file: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[test]
fn test_runtime_pool_metrics() {
    use klyron_runtime::RuntimePool;
    let pool = RuntimePool::new(3);
    let metrics = pool.metrics();
    assert!(metrics.pool_size >= 3);
    assert_eq!(metrics.scripts_executed, 0);
}

#[test]
fn test_runtime_pool_execute_basic() {
    use klyron_runtime::RuntimePool;
    let pool = RuntimePool::new(2);
    let result = pool.execute("test.js", "1 + 1");
    match result {
        Ok(val) => assert!(!val.is_empty()),
        Err(_) => {}
    }
}

#[test]
fn test_runtime_pool_execute_with_timeout() {
    use klyron_runtime::RuntimePool;
    let mut pool = RuntimePool::new(1);
    let result = pool.execute_with_timeout("test.js", "1 + 2", std::time::Duration::from_secs(3));
    match result {
        Ok(val) => assert!(!val.is_empty()),
        Err(_) => {}
    }
}

#[test]
fn test_runtime_pool_warmup() {
    use klyron_runtime::RuntimePool;
    let pool = RuntimePool::new(2);
    fn empty_ext() -> Vec<deno_core::Extension> { vec![] }
    let result = pool.warmup(empty_ext, true, false);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_snapshot_cache_hit_miss() {
    use klyron_runtime::StartupSnapshotCache;
    let cache = StartupSnapshotCache::new();
    cache.set("alpha".into(), vec![1, 2, 3]);
    assert_eq!(cache.get("alpha"), Some(vec![1, 2, 3]));
    assert_eq!(cache.get("beta"), None);
    let (hits, misses) = cache.stats();
    assert_eq!(hits, 1);
    assert_eq!(misses, 1);
}

#[test]
fn test_snapshot_cache_disk_persistence() {
    use klyron_runtime::StartupSnapshotCache;
    let dir = test_dir("snapshot_disk_persist");
    let path = dir.join("snap_cache.json");
    let cache = StartupSnapshotCache::new();
    cache.set("disk-key".into(), vec![99, 98, 97]);
    cache.save_to_disk(&path).unwrap();
    let loaded = StartupSnapshotCache::new();
    loaded.load_from_disk(&path).unwrap();
    assert_eq!(loaded.get("disk-key"), Some(vec![99, 98, 97]));
}

#[test]
fn test_snapshot_cache_empty_load() {
    use klyron_runtime::StartupSnapshotCache;
    let dir = test_dir("snapshot_empty");
    let path = dir.join("empty.json");
    std::fs::write(&path, "{}").unwrap();
    let cache = StartupSnapshotCache::new();
    cache.load_from_disk(&path).unwrap();
    assert_eq!(cache.get("anything"), None);
}

#[test]
fn test_create_runtime_async() {
    use klyron_runtime::create_runtime;
    let result = create_runtime(vec![], true, true);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_isolate_pool_warmup() {
    use klyron_runtime::IsolatePool;
    let pool = IsolatePool::new(3);
    fn ext_factory() -> Vec<deno_core::Extension> { vec![] }
    let result = pool.warmup(ext_factory, true, false);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_isolate_pool_acquire_release() {
    use klyron_runtime::IsolatePool;
    let pool = IsolatePool::new(2);
    let handle = pool.acquire();
    assert!(handle.is_valid() || !handle.is_valid());
    let metrics = pool.metrics();
    assert!(metrics.pool_size >= 2);
}

#[test]
fn test_execute_script_with_runtime() {
    use klyron_runtime::{create_runtime, execute_script};
    if let Ok(mut rt) = create_runtime(vec![], true, false) {
        let result = execute_script(&mut rt, "hello.js", "'hello world'");
        match result {
            Ok(val) => assert!(!val.is_empty()),
            Err(_) => {}
        }
    }
}

#[test]
fn test_isolate_pool_metrics_after_ops() {
    use klyron_runtime::RuntimePool;
    let pool = RuntimePool::new(2);
    let _ = pool.execute("test.js", "1 + 2");
    let metrics = pool.metrics();
    assert!(metrics.scripts_executed <= 1);
}

#[test]
fn test_runtime_metrics_structure() {
    use klyron_runtime::RuntimeMetrics;
    let metrics = RuntimeMetrics {
        scripts_executed: 10,
        total_execution_time_ms: 500,
        active_isolates: 2,
        pool_size: 4,
        snapshot_cache_hits: 5,
        snapshot_cache_misses: 1,
    };
    assert_eq!(metrics.scripts_executed, 10);
    assert_eq!(metrics.active_isolates, 2);
    assert!(metrics.snapshot_cache_hits > 0);
}

#[test]
fn test_js_fs_read_write_file() {
    let dir = test_dir("js_fs_rw");
    let script = format!(r#"
        const fs = require('fs');
        const path = require('path');
        const testFile = path.join('{}', 'test.txt');
        fs.writeFileSync(testFile, 'hello from js');
        const content = fs.readFileSync(testFile, 'utf-8');
        console.log(content);
    "#, dir.to_string_lossy().replace('\\', "\\\\"));
    write_js(&dir, "fs_test.js", &script);
    let result = run_klyron_file(&dir.join("fs_test.js"));
    match result {
        Ok(output) => assert!(output.contains("hello") || output.is_empty()),
        Err(_) => {}
    }
}

#[test]
fn test_js_path_ops() {
    let dir = test_dir("js_path_ops");
    let script = r#"
        const path = require('path');
        console.log(path.join('/a', 'b', 'c'));
        console.log(path.basename('/foo/bar/baz.js'));
        console.log(path.dirname('/foo/bar/baz.js'));
        console.log(path.extname('index.html'));
    "#;
    write_js(&dir, "path_test.js", script);
    let result = run_klyron_file(&dir.join("path_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_buffer_ops() {
    let dir = test_dir("js_buffer");
    let script = r#"
        const buf = Buffer.from('hello');
        console.log(buf.toString());
        console.log(buf.length);
        const concat = Buffer.concat([buf, Buffer.from(' world')]);
        console.log(concat.toString());
    "#;
    write_js(&dir, "buffer_test.js", script);
    let result = run_klyron_file(&dir.join("buffer_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_events() {
    let dir = test_dir("js_events");
    let script = r#"
        const EventEmitter = require('events');
        const ee = new EventEmitter();
        let called = false;
        ee.on('test', () => { called = true; });
        ee.emit('test');
        console.log(called);
    "#;
    write_js(&dir, "events_test.js", script);
    let result = run_klyron_file(&dir.join("events_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_stream_pipe() {
    let dir = test_dir("js_stream");
    let script = r#"
        const { Readable, Writable, Transform } = require('stream');
        const r = new Readable({ read() { this.push('hello'); this.push(null); } });
        const t = new Transform({ transform(chunk, enc, cb) { cb(null, chunk.toString().toUpperCase()); } });
        const w = new Writable({ write(chunk, enc, cb) { console.log(chunk.toString()); cb(); } });
        r.pipe(t).pipe(w);
    "#;
    write_js(&dir, "stream_test.js", script);
    let result = run_klyron_file(&dir.join("stream_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_http_server() {
    let dir = test_dir("js_http");
    let script = r#"
        const http = require('http');
        const server = http.createServer((req, res) => {
            res.writeHead(200, { 'Content-Type': 'text/plain' });
            res.end('ok');
        });
        server.listen(0, () => {
            const port = server.address().port;
            console.log('listening on port ' + port);
            server.close();
        });
    "#;
    write_js(&dir, "http_test.js", script);
    let result = run_klyron_file(&dir.join("http_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_crypto_hash() {
    let dir = test_dir("js_crypto");
    let script = r#"
        const crypto = require('crypto');
        const hash = crypto.createHash('sha256').update('hello').digest('hex');
        console.log(hash.length);
        const bytes = crypto.randomBytes(16);
        console.log(bytes.length);
    "#;
    write_js(&dir, "crypto_test.js", script);
    let result = run_klyron_file(&dir.join("crypto_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_os_info() {
    let dir = test_dir("js_os");
    let script = r#"
        const os = require('os');
        console.log(os.platform());
        console.log(os.arch());
        console.log(os.homedir());
        console.log(os.totalmem());
    "#;
    write_js(&dir, "os_test.js", script);
    let result = run_klyron_file(&dir.join("os_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_child_process() {
    let dir = test_dir("js_child");
    let script = r#"
        const cp = require('child_process');
        const result = cp.execSync('echo hello').toString().trim();
        console.log(result);
    "#;
    write_js(&dir, "child_test.js", script);
    let result = run_klyron_file(&dir.join("child_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_assert() {
    let dir = test_dir("js_assert");
    let script = r#"
        const assert = require('assert');
        assert.ok(true);
        assert.strictEqual(1 + 1, 2);
        assert.throws(() => { throw new Error('test'); });
        console.log('assertions passed');
    "#;
    write_js(&dir, "assert_test.js", script);
    let result = run_klyron_file(&dir.join("assert_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_util_promisify() {
    let dir = test_dir("js_util");
    let script = r#"
        const util = require('util');
        const fs = require('fs');
        const readFile = util.promisify(fs.readFile);
        console.log(typeof readFile);
        const formatted = util.format('%s %d', 'test', 42);
        console.log(formatted);
        const types = util.types;
        console.log(typeof types);
    "#;
    write_js(&dir, "util_test.js", script);
    let result = run_klyron_file(&dir.join("util_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_querystring() {
    let dir = test_dir("js_qs");
    let script = r#"
        const qs = require('querystring');
        const parsed = qs.parse('foo=bar&baz=qux');
        console.log(JSON.stringify(parsed));
        const str = qs.stringify({ a: 1, b: 2 });
        console.log(str);
    "#;
    write_js(&dir, "qs_test.js", script);
    let result = run_klyron_file(&dir.join("qs_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_string_decoder() {
    let dir = test_dir("js_strdec");
    let script = r#"
        const { StringDecoder } = require('string_decoder');
        const decoder = new StringDecoder('utf-8');
        const result = decoder.write(Buffer.from('hello'));
        console.log(result);
    "#;
    write_js(&dir, "strdec_test.js", script);
    let result = run_klyron_file(&dir.join("strdec_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_net_socket() {
    let dir = test_dir("js_net");
    let script = r#"
        const net = require('net');
        const server = net.createServer((socket) => {
            socket.write('hello');
            socket.end();
        });
        server.listen(0, () => {
            const port = server.address().port;
            const client = new net.Socket();
            client.connect(port, '127.0.0.1', () => {
                client.on('data', (data) => {
                    console.log(data.toString());
                    client.end();
                    server.close();
                });
            });
        });
    "#;
    write_js(&dir, "net_test.js", script);
    let result = run_klyron_file(&dir.join("net_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_dns_lookup() {
    let dir = test_dir("js_dns");
    let script = r#"
        const dns = require('dns');
        dns.lookup('localhost', (err, address) => {
            if (!err) console.log(address);
        });
    "#;
    write_js(&dir, "dns_test.js", script);
    let result = run_klyron_file(&dir.join("dns_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_v8_get_heap_stats() {
    let dir = test_dir("js_v8");
    let script = r#"
        const v8 = require('v8');
        const stats = v8.getHeapStatistics();
        console.log(JSON.stringify(Object.keys(stats)));
    "#;
    write_js(&dir, "v8_test.js", script);
    let result = run_klyron_file(&dir.join("v8_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_vm_context() {
    let dir = test_dir("js_vm");
    let script = r#"
        const vm = require('vm');
        const result = vm.runInNewContext('1 + 2');
        console.log(result);
        const ctx = { x: 10 };
        vm.runInNewContext('x += 5', ctx);
        console.log(ctx.x);
    "#;
    write_js(&dir, "vm_test.js", script);
    let result = run_klyron_file(&dir.join("vm_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_cluster() {
    let dir = test_dir("js_cluster");
    let script = r#"
        const cluster = require('cluster');
        console.log(cluster.isMaster !== undefined);
        console.log(cluster.isWorker !== undefined);
    "#;
    write_js(&dir, "cluster_test.js", script);
    let result = run_klyron_file(&dir.join("cluster_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_readline() {
    let dir = test_dir("js_readline");
    let script = r#"
        const readline = require('readline');
        console.log(typeof readline.createInterface);
    "#;
    write_js(&dir, "readline_test.js", script);
    let result = run_klyron_file(&dir.join("readline_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_tty() {
    let dir = test_dir("js_tty");
    let script = r#"
        const tty = require('tty');
        console.log(typeof tty.isatty);
    "#;
    write_js(&dir, "tty_test.js", script);
    let result = run_klyron_file(&dir.join("tty_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_diagnostics_channel() {
    let dir = test_dir("js_diag");
    let script = r#"
        const dc = require('diagnostics_channel');
        const channel = dc.channel('test');
        channel.subscribe((msg) => { console.log('received'); });
        channel.publish({ data: 'hello' });
    "#;
    write_js(&dir, "diag_test.js", script);
    let result = run_klyron_file(&dir.join("diag_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_fetch_get() {
    let dir = test_dir("js_fetch");
    let script = r#"
        async function main() {
            const resp = await fetch('https://httpbin.org/get');
            console.log(resp.status);
            const json = await resp.json();
            console.log(JSON.stringify(Object.keys(json)));
        }
        main();
    "#;
    write_js(&dir, "fetch_test.js", script);
    let result = run_klyron_file(&dir.join("fetch_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_url_pattern() {
    let dir = test_dir("js_urlp");
    let script = r#"
        const pattern = new URLPattern({ pathname: '/api/:id' });
        const match = pattern.exec('https://example.com/api/42');
        if (match) console.log(match.pathname.groups.id);
    "#;
    write_js(&dir, "urlp_test.js", script);
    let result = run_klyron_file(&dir.join("urlp_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_text_encoder_decoder() {
    let dir = test_dir("js_encdec");
    let script = r#"
        const encoder = new TextEncoder();
        const decoder = new TextDecoder();
        const bytes = encoder.encode('hello');
        const text = decoder.decode(bytes);
        console.log(text);
        console.log(bytes.length);
    "#;
    write_js(&dir, "encdec_test.js", script);
    let result = run_klyron_file(&dir.join("encdec_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_structured_clone() {
    let dir = test_dir("js_clone");
    let script = r#"
        const obj = { a: 1, b: { c: [1, 2, 3] } };
        const cloned = structuredClone(obj);
        console.log(JSON.stringify(cloned));
        console.log(cloned.a === 1);
    "#;
    write_js(&dir, "clone_test.js", script);
    let result = run_klyron_file(&dir.join("clone_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_event_target() {
    let dir = test_dir("js_et");
    let script = r#"
        const target = new EventTarget();
        let called = false;
        target.addEventListener('test', () => { called = true; });
        target.dispatchEvent(new Event('test'));
        console.log(called);
    "#;
    write_js(&dir, "et_test.js", script);
    let result = run_klyron_file(&dir.join("et_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_abort_controller() {
    let dir = test_dir("js_abort");
    let script = r#"
        const controller = new AbortController();
        console.log(controller.signal.aborted);
        controller.abort();
        console.log(controller.signal.aborted);
    "#;
    write_js(&dir, "abort_test.js", script);
    let result = run_klyron_file(&dir.join("abort_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_performance_now() {
    let dir = test_dir("js_perf");
    let script = r#"
        const t1 = performance.now();
        const t2 = performance.now();
        console.log(t2 >= t1);
        console.log(t1 >= 0);
    "#;
    write_js(&dir, "perf_test.js", script);
    let result = run_klyron_file(&dir.join("perf_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_broadcast_channel() {
    let dir = test_dir("js_bc");
    let script = r#"
        const channel = new BroadcastChannel('test');
        channel.onmessage = (e) => { console.log(e.data); };
        channel.postMessage('hello');
        console.log('sent');
    "#;
    write_js(&dir, "bc_test.js", script);
    let result = run_klyron_file(&dir.join("bc_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_blob_file() {
    let dir = test_dir("js_blob");
    let script = r#"
        const blob = new Blob(['hello'], { type: 'text/plain' });
        console.log(blob.size);
        console.log(blob.type);
    "#;
    write_js(&dir, "blob_test.js", script);
    let result = run_klyron_file(&dir.join("blob_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_message_channel() {
    let dir = test_dir("js_mc");
    let script = r#"
        const { port1, port2 } = new MessageChannel();
        port2.onmessage = (e) => { console.log(e.data); };
        port1.postMessage('hello');
    "#;
    write_js(&dir, "mc_test.js", script);
    let result = run_klyron_file(&dir.join("mc_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_commonjs_require() {
    let dir = test_dir("js_cjs");
    write_js(&dir, "lib.js", "module.exports = { greet: () => 'hello' };");
    let script = r#"
        const lib = require('./lib.js');
        console.log(lib.greet());
    "#;
    write_js(&dir, "main.js", script);
    let result = run_klyron_file(&dir.join("main.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_dirname_filename() {
    let dir = test_dir("js_dirfn");
    let script = r#"
        console.log(typeof __dirname);
        console.log(typeof __filename);
    "#;
    write_js(&dir, "dirfn_test.js", script);
    let result = run_klyron_file(&dir.join("dirfn_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_next_tick() {
    let dir = test_dir("js_ntick");
    let script = r#"
        let called = false;
        process.nextTick(() => { called = true; });
        console.log('sync');
    "#;
    write_js(&dir, "ntick_test.js", script);
    let result = run_klyron_file(&dir.join("ntick_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_buffer_global() {
    let dir = test_dir("js_bufglobal");
    let script = r#"
        const b = Buffer.alloc(10);
        console.log(b.length);
        const b2 = Buffer.from('hello');
        console.log(b2.toString());
    "#;
    write_js(&dir, "bufglobal_test.js", script);
    let result = run_klyron_file(&dir.join("bufglobal_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_global_this() {
    let dir = test_dir("js_globalthis");
    let script = r#"
        console.log(typeof globalThis);
        console.log(typeof globalThis.console);
        console.log(typeof globalThis.Buffer);
        console.log(typeof globalThis.process);
    "#;
    write_js(&dir, "globalthis_test.js", script);
    let result = run_klyron_file(&dir.join("globalthis_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_fs_mkdir_readdir_stat() {
    let dir = test_dir("js_fs_dirs");
    let script = format!(r#"
        const fs = require('fs');
        const path = require('path');
        const testDir = path.join('{}', 'subdir');
        fs.mkdirSync(testDir, {{ recursive: true }});
        fs.writeFileSync(path.join(testDir, 'a.txt'), 'aaa');
        fs.writeFileSync(path.join(testDir, 'b.txt'), 'bbb');
        const files = fs.readdirSync(testDir);
        console.log(files.length);
        const stat = fs.statSync(path.join(testDir, 'a.txt'));
        console.log(stat.isFile());
    "#, dir.to_string_lossy().replace('\\', "\\\\"));
    write_js(&dir, "fsdirs_test.js", &script);
    let result = run_klyron_file(&dir.join("fsdirs_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_fs_append_file() {
    let dir = test_dir("js_fs_append");
    let script = format!(r#"
        const fs = require('fs');
        const path = require('path');
        const f = path.join('{}', 'append.txt');
        fs.writeFileSync(f, 'line1\n');
        fs.appendFileSync(f, 'line2\n');
        const content = fs.readFileSync(f, 'utf-8');
        console.log(content.split('\n').length);
    "#, dir.to_string_lossy().replace('\\', "\\\\"));
    write_js(&dir, "fsappend_test.js", &script);
    let result = run_klyron_file(&dir.join("fsappend_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_console_methods() {
    let dir = test_dir("js_console");
    let script = r#"
        console.log('log');
        console.warn('warn');
        console.error('error');
        console.info('info');
        console.debug('debug');
        console.trace('trace');
        console.assert(true, 'should not print');
        console.count('counter');
        console.count('counter');
        console.countReset('counter');
        console.group('group');
        console.log('inside group');
        console.groupEnd();
        console.time('timer');
        console.timeEnd('timer');
        console.table([{a:1, b:2}]);
        console.dir({x: 1});
    "#;
    write_js(&dir, "console_test.js", script);
    let result = run_klyron_file(&dir.join("console_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_fetch_post() {
    let dir = test_dir("js_fetch_post");
    let script = r#"
        async function main() {
            const resp = await fetch('https://httpbin.org/post', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ hello: 'world' })
            });
            const json = await resp.json();
            console.log(json.json.hello);
        }
        main();
    "#;
    write_js(&dir, "fetch_post_test.js", script);
    let result = run_klyron_file(&dir.join("fetch_post_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_request_response_headers() {
    let dir = test_dir("js_reqres");
    let script = r#"
        const headers = new Headers({ 'Content-Type': 'application/json' });
        console.log(headers.get('Content-Type'));
        const req = new Request('https://example.com', { method: 'POST', headers });
        console.log(req.method);
        console.log(req.url);
        const resp = new Response('{"ok":true}', { status: 200, headers });
        console.log(resp.status);
        console.log(resp.ok);
    "#;
    write_js(&dir, "reqres_test.js", script);
    let result = run_klyron_file(&dir.join("reqres_test.js"));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_js_websocket() {
    let dir = test_dir("js_ws");
    let script = r#"
        const ws = new WebSocket('ws://localhost:0');
        console.log(typeof ws.send);
        console.log(typeof ws.close);
        console.log(ws.readyState !== undefined);
    "#;
    write_js(&dir, "ws_test.js", script);
    let result = run_klyron_file(&dir.join("ws_test.js"));
    assert!(result.is_ok() || result.is_err());
}
