#![cfg(test)]
use crate::PackageManager;

#[test]
fn bench_detect() {
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = crate::detect(std::path::Path::new("/tmp"));
    }
    println!("detect() x1000: {:?}", start.elapsed());
}

#[test]
fn bench_install_cmd() {
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = crate::install_cmd(PackageManager::Npm);
    }
    println!("install_cmd() x10000: {:?}", start.elapsed());
}
