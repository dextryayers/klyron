/// Integration tests for klyron CLI commands.

use std::process::Command;
use std::path::Path;

fn klyron_bin() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_klyron"))
}

#[test]
fn test_cli_version() {
    let output = std::process::Command::new(klyron_bin())
        .arg("-V")
        .output()
        .expect("Failed to run klyron -V");
    assert!(output.status.success());
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("klyron"));
}

#[test]
fn test_cli_help() {
    let output = std::process::Command::new(klyron_bin())
        .arg("--help")
        .output()
        .expect("Failed to run klyron --help");
    assert!(output.status.success());
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("Usage") || out.contains("Commands"));
}

#[test]
fn test_cli_eval_js() {
    let output = std::process::Command::new(klyron_bin())
        .args(["eval", "console.log('hello')"])
        .output()
        .expect("Failed to run klyron eval");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_cli_eval_math() {
    let output = std::process::Command::new(klyron_bin())
        .args(["--json", "eval", "1 + 1"])
        .output()
        .expect("Failed to run klyron eval --json");
    assert!(output.status.success());
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("ok") || out.contains("2"));
}

#[test]
fn test_cli_version_json() {
    let output = std::process::Command::new(klyron_bin())
        .args(["--json", "info"])
        .output()
        .expect("Failed to run klyron --json info");
    assert!(output.status.success());
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("version"));
}
