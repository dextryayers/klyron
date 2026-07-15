use std::time::Instant;
use std::path::PathBuf;

fn bench_node_fs_write(count: usize) {
    let tmp = std::env::temp_dir().join(format!("klyron_fs_bench_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).expect("mkdir failed");

    let start = Instant::now();
    for i in 0..count {
        let path = tmp.join(format!("file-{i}.txt"));
        std::fs::write(&path, format!("content-{i}")).expect("write failed");
    }
    let elapsed = start.elapsed();
    let avg = elapsed / count as u32;
    println!("  node:fs write {count} files: {elapsed:?} ({avg:?} avg)");

    let _ = std::fs::remove_dir_all(&tmp);
}

fn bench_node_fs_read(count: usize) {
    let tmp = std::env::temp_dir().join(format!("klyron_fs_bench_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).expect("mkdir failed");
    for i in 0..count {
        std::fs::write(tmp.join(format!("file-{i}.txt")), format!("content-{i}")).expect("write failed");
    }

    let start = Instant::now();
    for i in 0..count {
        let _content = std::fs::read_to_string(tmp.join(format!("file-{i}.txt"))).expect("read failed");
    }
    let elapsed = start.elapsed();
    let avg = elapsed / count as u32;
    println!("  node:fs read {count} files: {elapsed:?} ({avg:?} avg)");

    let _ = std::fs::remove_dir_all(&tmp);
}

fn bench_node_crypto_hash(sizes: &[usize]) {
    use sha2::{Sha256, Digest};
    for &size in sizes {
        let data = vec![0xABu8; size];
        let start = Instant::now();
        let iterations = 10000u64;
        for _ in 0..iterations {
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let _hash = hasher.finalize();
        }
        let elapsed = start.elapsed();
        let avg = elapsed / iterations as u32;
        let throughput = (size as f64 * iterations as f64 / elapsed.as_secs_f64()) / (1024.0 * 1024.0);
        println!("  sha256 {size:>6}B x{iterations}: {elapsed:?} ({avg:?} avg, {throughput:.1} MB/s)");
    }
}

fn bench_node_path_operations(count: usize) {
    use std::path::Path;

    let paths: Vec<&str> = (0..count).map(|i| {
        if i % 3 == 0 { format!("/usr/local/lib/node_modules/pkg-{i}/index.js") }
        else if i % 3 == 1 { format!("./src/components/Component{i}.tsx") }
        else { format!("../node_modules/.pnpm/pkg@{i}/node_modules/pkg/index.mjs") }
    }).collect();

    let start = Instant::now();
    for p in &paths {
        let path = Path::new(p);
        let _parent = path.parent();
        let _file_name = path.file_name();
        let _extension = path.extension();
        let _ = path.join("subdir").join("file.txt");
    }
    let elapsed = start.elapsed();
    let avg = elapsed / count as u32;
    println!("  path ops ({count}): {elapsed:?} ({avg:?} avg)");
}

fn bench_http_server_requests() {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
    let port = listener.local_addr().unwrap().port();
    let handle = std::thread::spawn(move || {
        for stream in listener.incoming().take(100) {
            if let Ok(mut stream) = stream {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello";
                let _ = stream.write_all(response);
            }
        }
    });

    std::thread::sleep(std::time::Duration::from_millis(50));

    let count = 100u32;
    let start = Instant::now();
    for _ in 0..count {
        if let Ok(mut stream) = TcpStream::connect(format!("127.0.0.1:{port}")) {
            let _ = stream.write_all(b"GET / HTTP/1.0\r\nHost: localhost\r\nConnection: close\r\n\r\n");
            let mut buf = Vec::new();
            let _ = stream.read_to_end(&mut buf);
        }
    }
    let elapsed = start.elapsed();
    let avg = elapsed / count;
    let rps = (count as f64 / elapsed.as_secs_f64()) as u64;
    println!("  HTTP server ({count} requests): {elapsed:?} ({avg:?} avg, {rps} req/s)");

    handle.join().ok();
}

fn bench_json_parse_stringify(count: usize) {
    let objects: Vec<serde_json::Value> = (0..count).map(|i| {
        serde_json::json!({
            "id": i,
            "name": format!("object-{i}"),
            "active": i % 2 == 0,
            "tags": (0..5).map(|t| format!("tag-{t}")).collect::<Vec<_>>(),
            "metadata": {
                "created": "2024-01-01T00:00:00Z",
                "priority": i % 3,
                "score": i as f64 * 1.5,
            }
        })
    }).collect();

    let start = Instant::now();
    for obj in &objects {
        let json = serde_json::to_string(obj).expect("serialize failed");
        let _parsed: serde_json::Value = serde_json::from_str(&json).expect("parse failed");
    }
    let elapsed = start.elapsed();
    let avg = elapsed / count as u32;
    let ops = (count as f64 / elapsed.as_secs_f64()) as u64;
    println!("  JSON parse+stringify ({count}): {elapsed:?} ({avg:?} avg, {ops} ops/s)");
}

fn bench_buffer_concat() {
    let chunks: Vec<Vec<u8>> = (0..1000).map(|i| vec![i as u8; 64]).collect();

    let start = Instant::now();
    let iterations = 10000u64;
    for _ in 0..iterations {
        let mut total_len = 0usize;
        for c in &chunks { total_len += c.len(); }
        let mut buf = Vec::with_capacity(total_len);
        for c in &chunks { buf.extend_from_slice(c); }
    }
    let elapsed = start.elapsed();
    let avg = elapsed / iterations as u32;
    let throughput = (chunks.len() * 64 * iterations as usize) as f64 / elapsed.as_secs_f64() / (1024.0 * 1024.0);
    println!("  buffer concat ({iterations} iterations): {elapsed:?} ({avg:?} avg, {throughput:.1} MB/s)");
}

fn main() {
    println!("\n=== Polyfill Benchmarks ===");

    println!("\n-- node:fs --");
    bench_node_fs_write(1000);
    bench_node_fs_read(1000);

    println!("\n-- node:crypto --");
    bench_node_crypto_hash(&[64, 1024, 65536, 1048576]);

    println!("\n-- path --");
    bench_node_path_operations(10000);

    println!("\n-- HTTP --");
    bench_http_server_requests();

    println!("\n-- JSON --");
    bench_json_parse_stringify(10000);

    println!("\n-- Buffer --");
    bench_buffer_concat();
}
