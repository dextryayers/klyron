use klyron_pm::lockfile::{KlyronLockfile, LockfilePackage};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LockfileBenchResult {
    pub package_count: usize,
    pub serialize_time_ns: u64,
    pub deserialize_time_ns: u64,
    pub binary_size: usize,
    pub json_size: usize,
    pub npm_lock_size: Option<usize>,
}

pub fn generate_test_lockfile(package_count: usize) -> KlyronLockfile {
    let mut lock = KlyronLockfile::new();
    for i in 0..package_count {
        let name = format!("pkg-{i}");
        let version = format!("1.{}.{}", i / 100, i % 100);
        let mut deps = HashMap::new();
        if i > 0 {
            deps.insert(format!("pkg-{}", i - 1), "1.0.0".to_string());
        }
        lock.add_package(
            &name,
            &version,
            LockfilePackage {
                name: name.clone(),
                version: version.clone(),
                resolved: format!(
                    "https://registry.npmjs.org/{name}/-/{name}-{version}.tgz"
                ),
                integrity: format!("sha512-{}", "a".repeat(64)),
                dependencies: deps,
                optional_dependencies: HashMap::new(),
                peer_dependencies: HashMap::new(),
                integrity_hashes: Vec::new(),
                signature: None,
                signer: None,
                bin: None,
                has_node_modules: false,
                install_time_ms: (i as u64) % 1000,
            },
        );
    }
    lock
}

pub fn bench_lockfile_serialization(package_count: usize) -> LockfileBenchResult {
    let lock = generate_test_lockfile(package_count);

    let start = std::time::Instant::now();
    let binary = lock.to_bytes().unwrap();
    let serialize_time = start.elapsed().as_nanos() as u64;

    let start = std::time::Instant::now();
    let _decoded = KlyronLockfile::from_bytes(&binary).unwrap();
    let deserialize_time = start.elapsed().as_nanos() as u64;

    let json = lock.to_json_pretty().unwrap();
    let json_size = json.len();
    let binary_size = binary.len();

    let npm_packages: serde_json::Value = {
        let mut pkgs = serde_json::Map::new();
        for i in 0..package_count {
            let name = format!("pkg-{i}");
            let version = format!("1.{}.{}", i / 100, i % 100);
            pkgs.insert(
                format!("node_modules/{name}"),
                serde_json::json!({
                    "version": version,
                    "resolved": format!("https://registry.npmjs.org/{name}/-/{name}-{version}.tgz"),
                    "integrity": format!("sha512-{}", "a".repeat(64)),
                }),
            );
        }
        serde_json::json!({
            "name": "bench",
            "lockfileVersion": 3,
            "packages": pkgs,
        })
    };
    let npm_lock_size =
        Some(serde_json::to_string_pretty(&npm_packages).unwrap().len());

    LockfileBenchResult {
        package_count,
        serialize_time_ns: serialize_time,
        deserialize_time_ns: deserialize_time,
        binary_size,
        json_size,
        npm_lock_size,
    }
}

pub fn run_all_lockfile_benchmarks() -> Vec<LockfileBenchResult> {
    let mut results = Vec::new();
    for &count in &[100, 1000, 10000] {
        let result = bench_lockfile_serialization(count);
        println!("\n--- Lockfile Bench: {} packages ---", count);
        println!("  Binary serialize: {} ns", result.serialize_time_ns);
        println!("  Binary deserialize: {} ns", result.deserialize_time_ns);
        println!("  Binary size: {} bytes", result.binary_size);
        println!("  JSON size: {} bytes", result.json_size);
        if let Some(npm) = result.npm_lock_size {
            println!("  package-lock.json size (est): {} bytes", npm);
            let ratio = result.binary_size as f64 / npm as f64;
            println!("  Binary/package-lock ratio: {:.2}%", ratio * 100.0);
        }
        results.push(result);
    }
    results
}
