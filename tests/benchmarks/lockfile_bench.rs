use std::collections::HashMap;
use std::time::Instant;

/// Benchmark lockfile serialization/deserialization for different package sizes
fn bench_lockfile(package_count: usize) -> (u128, u128, usize, usize, usize) {
    use klyron_pm::lockfile::{KlyronLockfile, LockfilePackage};

    let mut lock = KlyronLockfile::new();
    for i in 0..package_count {
        let name = format!("bench-pkg-{i}");
        let version = format!("{}.{}.{}", i / 1000, (i % 1000) / 10, i % 10);
        let mut deps = HashMap::new();
        if i > 0 {
            deps.insert(format!("bench-pkg-{}", i - 1), "1.0.0".to_string());
        }

        lock.add_package(&name, &version, LockfilePackage {
            name,
            version,
            resolved: format!("https://registry.example.com/pkg-{i}.tgz"),
            integrity: format!("sha512-{}", "abcdef0123456789".repeat(4)),
            dependencies: deps,
            optional_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            bin: None,
            has_node_modules: false,
            install_time_ms: (i as u64).saturating_mul(10),
        });
    }

    // Measure serialization
    let start = Instant::now();
    let binary = lock.to_bytes().unwrap();
    let serialize_ns = start.elapsed().as_nanos();

    // Measure deserialization
    let start = Instant::now();
    let _decoded = KlyronLockfile::from_bytes(&binary).unwrap();
    let deserialize_ns = start.elapsed().as_nanos();

    // JSON size for comparison
    let json = lock.to_json_pretty().unwrap();
    let json_size = json.len();
    let binary_size = binary.len();

    // Estimate package-lock.json size
    let npm_json = serde_json::json!({
        "name": "bench",
        "lockfileVersion": 3,
        "packages": (0..package_count).map(|i| {
            let name = format!("node_modules/bench-pkg-{i}");
            let version = format!("{}.{}.{}", i / 1000, (i % 1000) / 10, i % 10);
            (name, serde_json::json!({
                "version": version,
                "resolved": format!("https://registry.example.com/pkg-{i}.tgz"),
                "integrity": format!("sha512-{}", "a".repeat(64)),
            }))
        }).collect::<serde_json::Map<_, _>>(),
    });
    let npm_lock_size = serde_json::to_string_pretty(&npm_json).unwrap().len();

    (serialize_ns, deserialize_ns, binary_size, json_size, npm_lock_size)
}

#[test]
fn test_lockfile_bench_100_packages() {
    let (ser, deser, bin, json, npm) = bench_lockfile(100);
    println!("[bench] 100 packages:");
    println!("  serialize: {ser} ns, deserialize: {deser} ns");
    println!("  binary: {bin} bytes, JSON: {json} bytes, npm: {npm} bytes");
    assert!(bin < json, "binary ({bin}) should be smaller than JSON ({json})");
    assert!(bin < npm, "binary ({bin}) should be smaller than npm lock ({npm})");
    assert!(ser > 0, "serialize time should be > 0");
    assert!(deser > 0, "deserialize time should be > 0");
}

#[test]
fn test_lockfile_bench_1000_packages() {
    let (ser, deser, bin, json, npm) = bench_lockfile(1000);
    println!("[bench] 1000 packages:");
    println!("  serialize: {ser} ns, deserialize: {deser} ns");
    println!("  binary: {bin} bytes, JSON: {json} bytes, npm: {npm} bytes");
    assert!(bin < json, "binary ({bin}) should be smaller than JSON ({json})");
    assert!(bin < npm, "binary ({bin}) should be smaller than npm lock ({npm})");
}

#[test]
fn test_lockfile_bench_10000_packages() {
    let (ser, deser, bin, json, npm) = bench_lockfile(10000);
    println!("[bench] 10000 packages:");
    println!("  serialize: {ser} ns, deserialize: {deser} ns");
    println!("  binary: {bin} bytes, JSON: {json} bytes, npm: {npm} bytes");
    assert!(bin < json, "binary ({bin}) should be smaller than JSON ({json})");
    // For 10k packages, even with overhead the binary ratio should be clear
    let ratio = bin as f64 / json as f64;
    assert!(ratio < 0.8, "binary/JSON ratio should be < 0.8, got {ratio:.2}");
}

#[test]
fn test_lockfile_bench_compare_sizes() {
    let sizes: Vec<(usize, usize, usize)> = [100, 1000, 10000]
        .iter()
        .map(|&n| {
            let (_, _, bin, json, npm) = bench_lockfile(n);
            (bin, json, npm)
        })
        .collect();

    for (i, &count) in [100, 1000, 10000].iter().enumerate() {
        let (bin, json, npm) = sizes[i];
        println!("{count:>5} pkgs | binary: {bin:>8} | JSON: {json:>8} | npm: {npm:>8} | ratio: {:.2}%", bin as f64 / json as f64 * 100.0);
    }

    // Verify scaling is roughly linear (not super-linear)
    if sizes.len() >= 3 {
        let ratio_1k = sizes[1].0 as f64 / sizes[0].0 as f64;
        let ratio_10k = sizes[2].0 as f64 / sizes[1].0 as f64;
        // 1000/100 = 10, 10000/1000 = 10, so ratios should be similar
        assert!((ratio_1k - ratio_10k).abs() < 5.0,
            "scaling should be linear: 1k ratio={ratio_1k:.1}, 10k ratio={ratio_10k:.1}");
    }
}
