#![cfg(test)]

#[test]
fn bench_version() {
    let client = crate::client::BundlerClient::new();
    let start = std::time::Instant::now();
    for _ in 0..1000 { let _ = client.version(); }
    println!("version() x1000: {:?}", start.elapsed());
}
