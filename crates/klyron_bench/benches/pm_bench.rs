use std::time::Instant;
use klyron_pm::lockfile::{KlyronLockfile, LockfilePackage};
use std::collections::HashMap;

fn generate_lockfile(count: usize) -> KlyronLockfile {
    let mut lock = KlyronLockfile::new();
    for i in 0..count {
        let name = format!("pkg-{i}");
        let version = format!("1.{}.{}", i / 100, i % 100);
        let mut deps = HashMap::new();
        if i > 0 {
            deps.insert(format!("pkg-{}", i - 1), format!("1.{}", (i - 1) / 100));
        }
        lock.add_package(&name, &version, LockfilePackage {
            name: name.clone(),
            version: version.clone(),
            resolved: format!("https://registry.npmjs.org/{name}/-/{name}-{version}.tgz"),
            integrity: format!("sha512-{}", "a".repeat(64)),
            dependencies: deps,
            optional_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            bin: None,
            has_node_modules: false,
            install_time_ms: (i as u64) % 1000,
        });
    }
    lock
}

fn bench_lockfile_generate(count: usize, label: &str) {
    let start = Instant::now();
    let lock = generate_lockfile(count);
    let gen_time = start.elapsed();

    let start = Instant::now();
    let bytes = lock.to_bytes().expect("Serialize failed");
    let serialize_time = start.elapsed();

    let start = Instant::now();
    let _ = KlyronLockfile::from_bytes(&bytes).expect("Deserialize failed");
    let deserialize_time = start.elapsed();

    println!(
        "  {label:<22} gen={gen_time:?} serialize={serialize_time:?} deserialize={deserialize_time:?} size={}B",
        bytes.len()
    );
}

fn bench_lockfile_verify() {
    let lock = generate_lockfile(100);
    let bytes = lock.to_bytes().expect("Serialize failed");

    let start = Instant::now();
    let decoded = KlyronLockfile::from_bytes(&bytes).expect("Deserialize failed");
    let verify_time = start.elapsed();

    let ok = decoded.packages().len() == 100;
    println!("  Lockfile verify: {verify_time:?} (valid: {ok})");
}

fn bench_lockfile_migrate_npm() {
    let npm_lock = serde_json::json!({
        "name": "test-project",
        "lockfileVersion": 3,
        "packages": {
            "node_modules/express": {
                "version": "4.18.2",
                "resolved": "https://registry.npmjs.org/express/-/express-4.18.2.tgz",
                "integrity": "sha512-1yDvK8mduT0jKDKjE3T4p9L0TBiKjBYE5kff5i5dXLjQl5sQf9HjKQ8G0nF0ATYd6pGg8QvBSl3WGWfGCzMvUlDA==",
                "dependencies": { "accepts": "1.3.8" }
            },
            "node_modules/accepts": {
                "version": "1.3.8",
                "resolved": "https://registry.npmjs.org/accepts/-/accepts-1.3.8.tgz",
                "integrity": "sha512-PYAthTa2m2VKxuvSD3DPC/Gy+U+sOA1LAuT8mkmRuvw+NACSaeT4Hlz6L5QqE6g==",
                "dependencies": { "mime-types": "2.1.35" }
            }
        }
    });

    let start = Instant::now();
    let _lock = KlyronLockfile::from_json(&serde_json::to_string(&npm_lock).unwrap()).expect("Migration failed");
    let migrate_time = start.elapsed();
    println!("  Lockfile migrate npm: {migrate_time:?}");
}

fn bench_lockfile_parse_binary() {
    let lock = generate_lockfile(1000);
    let bytes = lock.to_bytes().expect("Serialize failed");

    let start = Instant::now();
    for _ in 0..100 {
        let _ = KlyronLockfile::from_bytes(&bytes).expect("Deserialize failed");
    }
    let elapsed = start.elapsed();
    let avg = elapsed / 100u32;
    println!("  Lockfile parse binary (1000 pkgs): {avg:?} avg (100 iterations)");
}

fn bench_dependency_resolution() {
    let count = 500;
    let mut lock = generate_lockfile(count);
    let start = Instant::now();
    let resolved = lock.resolve_dependencies().expect("Resolution failed");
    let elapsed = start.elapsed();
    println!("  Dependency resolution ({count} pkgs): {elapsed:?} ({resolved} edges)");
}

fn main() {
    println!("\n=== Lockfile Benchmarks ===");

    println!("\n-- Generate --");
    bench_lockfile_generate(100, "100 packages");
    bench_lockfile_generate(1000, "1000 packages");
    bench_lockfile_generate(10000, "10000 packages");

    println!("\n-- Verify --");
    bench_lockfile_verify();

    println!("\n-- Migrate npm --");
    bench_lockfile_migrate_npm();

    println!("\n-- Parse Binary --");
    bench_lockfile_parse_binary();

    println!("\n-- Resolution --");
    bench_dependency_resolution();
}
