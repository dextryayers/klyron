use std::process::Command;
use std::time::Instant;

fn bench_cli_startup() {
    let start = Instant::now();
    let output = Command::new("cargo")
        .args(["run", "--manifest-path", "Cargo.toml", "--", "--version"])
        .output()
        .expect("Failed to run klyron");
    let elapsed = start.elapsed();
    let status = if output.status.success() { "ok" } else { "fail" };
    let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
    println!("  CLI startup: {elapsed:?} ({status}) version={ver}");
}

fn bench_cli_help() {
    let start = Instant::now();
    let output = Command::new("cargo")
        .args(["run", "--manifest-path", "Cargo.toml", "--", "--help"])
        .output()
        .expect("Failed to run klyron");
    let elapsed = start.elapsed();
    println!("  CLI help: {elapsed:?} ({} lines)", String::from_utf8_lossy(&output.stdout).lines().count());
}

fn bench_cli_install() {
    let tmp_dir = std::env::temp_dir().join(format!("klyron_bench_install_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&tmp_dir);
    std::fs::write(tmp_dir.join("package.json"), r#"{"name":"test","dependencies":{"left-pad":"1.3.0"}}"#)
        .expect("Write package.json failed");

    let start = Instant::now();
    let output = Command::new("cargo")
        .args(["run", "--manifest-path", "Cargo.toml", "--", "install"])
        .current_dir(&tmp_dir)
        .output()
        .expect("Failed to run klyron install");
    let elapsed = start.elapsed();
    println!("  CLI install (left-pad): {elapsed:?} (status: {})", output.status);
    let _ = std::fs::remove_dir_all(&tmp_dir);
}

fn bench_cli_eval() {
    let start = Instant::now();
    let output = Command::new("cargo")
        .args(["run", "--manifest-path", "Cargo.toml", "--", "eval", "console.log('hello')"])
        .output()
        .expect("Failed to run klyron eval");
    let elapsed = start.elapsed();
    let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
    println!("  CLI eval: {elapsed:?} (output={out:?})");
}

fn main() {
    println!("\n=== CLI Benchmarks ===");

    println!("\n-- Startup --");
    bench_cli_startup();

    println!("\n-- Help --");
    bench_cli_help();

    println!("\n-- Eval --");
    bench_cli_eval();

    println!("\n-- Install (requires network) --");
    // bench_cli_install(); // Disabled by default - needs network
    println!("  Skipped (requires network). Uncomment to run.");
}
