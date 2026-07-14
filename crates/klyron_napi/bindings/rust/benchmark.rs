#![cfg(test)]
use crate::NapiLoader;

#[test]
fn bench_napi_loader() {
    let loader = NapiLoader::new();
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = loader.napi_version();
    }
    let elapsed = start.elapsed();
    println!("napi_version() x1000: {:?}", elapsed);
}

#[test]
fn bench_is_napi_module() {
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = NapiLoader::is_napi_module("addon.node");
    }
    let elapsed = start.elapsed();
    println!("is_napi_module() x10000: {:?}", elapsed);
}
