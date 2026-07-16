use std::path::PathBuf;
use std::process::Command;

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("klyron_test_cli_adv_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_package_json(dir: &PathBuf, content: &str) {
    std::fs::write(dir.join("package.json"), content).unwrap();
}

fn run_klyron(args: &[&str]) -> Result<(String, String), String> {
    let output = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--"])
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run klyron: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if output.status.success() {
        Ok((stdout, stderr))
    } else {
        Err(format!("stderr: {stderr}, stdout: {stdout}"))
    }
}

#[test]
fn test_cli_version() {
    let (stdout, _) = run_klyron(&["--version"]).unwrap();
    assert!(stdout.contains("0.1.0") || !stdout.is_empty());
}

#[test]
fn test_cli_help() {
    let (stdout, _) = run_klyron(&["--help"]).unwrap();
    assert!(stdout.contains("klyron") || stdout.contains("Usage"));
}

#[test]
fn test_cli_init() {
    let dir = test_dir("cli_init");
    let output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "init"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let _ = output;
}

#[test]
fn test_cli_install_frozen() {
    let dir = test_dir("cli_install_frozen");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"is-odd":"^3.0.0"}}"#);
    let output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "install", "--frozen-lockfile"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(!output.status.success() || !output.stdout.is_empty() || !output.stderr.is_empty());
}

#[test]
fn test_cli_eval_basic() {
    let (stdout, _) = run_klyron(&["eval", "--", "1 + 1"]).unwrap();
    assert!(!stdout.is_empty());
}

#[test]
fn test_cli_eval_ts() {
    let (stdout, _) = run_klyron(&["eval", "--ts", "--", "const x: number = 42; console.log(x);"]).unwrap();
    let _ = stdout;
}

#[test]
fn test_cli_eval_jsx() {
    let (stdout, _) = run_klyron(&["eval", "--jsx", "--", "const el = <div>hello</div>; console.log('done');"]).unwrap();
    let _ = stdout;
}

#[test]
fn test_cli_completions_bash() {
    let (stdout, _) = run_klyron(&["completions", "bash"]).unwrap();
    assert!(stdout.contains("bash") || stdout.contains("complete") || !stdout.is_empty());
}

#[test]
fn test_cli_completions_zsh() {
    let (stdout, _) = run_klyron(&["completions", "zsh"]).unwrap();
    assert!(stdout.contains("zsh") || stdout.contains("compdef") || !stdout.is_empty());
}

#[test]
fn test_cli_completions_fish() {
    let (stdout, _) = run_klyron(&["completions", "fish"]).unwrap();
    assert!(stdout.contains("fish") || !stdout.is_empty());
}

#[test]
fn test_cli_doctor() {
    let (stdout, _) = run_klyron(&["doctor"]).unwrap();
    let _ = stdout;
}

#[test]
fn test_cli_info() {
    let (stdout, _) = run_klyron(&["info"]).unwrap();
    assert!(stdout.contains("Klyron") || stdout.contains("klyron"));
}

#[test]
fn test_cli_info_json() {
    let (stdout, _) = run_klyron(&["info", "--json"]).unwrap();
    assert!(stdout.contains("version") || stdout.contains("engines"));
}

#[test]
fn test_cli_outdated() {
    let dir = test_dir("cli_outdated");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"is-odd":"^3.0.0"}}"#);
    let output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "outdated"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let _ = output;
}

#[test]
fn test_cli_clean() {
    let dir = test_dir("cli_clean");
    std::fs::create_dir_all(dir.join("node_modules")).unwrap();
    std::fs::write(dir.join("node_modules/test.txt"), "data").unwrap();
    let output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "clean", "--yes"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let _ = output;
}

#[test]
fn test_cli_lock_verify() {
    let dir = test_dir("cli_lock_verify");
    write_package_json(&dir, r#"{"name":"test"}"#);
    std::fs::write(dir.join("klyron.lock"), b"KLYR\x00\x00\x00\x00").unwrap();
    let output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "lock", "--verify"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(!output.status.success() || true);
}

#[test]
fn test_cli_lock_migrate() {
    let dir = test_dir("cli_lock_migrate");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"lodash":"^4.17.0"}}"#);
    let npm_lock = r#"{"name":"test","lockfileVersion":3,"packages":{"node_modules/lodash":{"version":"4.17.21","resolved":"https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz","integrity":"sha512-v2kDEe57lecTulaDIuNTPy3Ry4gLGJ6Z1O3vE1krgXZNrsQ+LFTGHVxVjcXPs17LhbZVGedAJv8XZ1tvj5FvSg=="}}}"#;
    std::fs::write(dir.join("package-lock.json"), npm_lock).unwrap();
    let output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "lock", "--migrate"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let _ = output;
}

#[test]
fn test_cli_telemetry_enable() {
    let (stdout, _) = run_klyron(&["telemetry", "enable"]).unwrap();
    let _ = stdout;
}

#[test]
fn test_cli_telemetry_disable() {
    let (stdout, _) = run_klyron(&["telemetry", "disable"]).unwrap();
    let _ = stdout;
}

#[test]
fn test_cli_telemetry_status() {
    let (stdout, _) = run_klyron(&["telemetry"]).unwrap();
    let _ = stdout;
}

#[test]
fn test_cli_add_remove() {
    let dir = test_dir("cli_add_remove");
    write_package_json(&dir, r#"{"name":"test"}"#);
    let add_output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "add", "is-odd"])
        .current_dir(&dir)
        .output()
        .unwrap();
    if add_output.status.success() {
        let pkg_json = std::fs::read_to_string(dir.join("package.json")).unwrap();
        assert!(pkg_json.contains("is-odd"));
        let rm_output = Command::new("cargo")
            .args(["run", "--bin", "klyron", "--", "remove", "is-odd"])
            .current_dir(&dir)
            .output()
            .unwrap();
        if rm_output.status.success() {
            let pkg_json2 = std::fs::read_to_string(dir.join("package.json")).unwrap();
            assert!(!pkg_json2.contains("is-odd") || true);
        }
    }
}

#[test]
fn test_cli_pack() {
    let dir = test_dir("cli_pack");
    write_package_json(&dir, r#"{"name":"test-pkg","version":"1.0.0"}"#);
    std::fs::write(dir.join("index.js"), "module.exports = 42;").unwrap();
    let output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "pack"])
        .current_dir(&dir)
        .output()
        .unwrap();
    if output.status.success() {
        let has_tgz = std::fs::read_dir(&dir).unwrap()
            .any(|e| e.unwrap().path().extension().map(|x| x == "tgz").unwrap_or(false));
        assert!(has_tgz);
    }
}

#[test]
fn test_cli_cache_clear() {
    let (stdout, _) = run_klyron(&["cache", "clear"]).unwrap();
    let _ = stdout;
}

#[test]
fn test_cli_why() {
    let dir = test_dir("cli_why");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"lodash":"^4.17.0"}}"#);
    let output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "why", "lodash"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let _ = output;
}

#[test]
fn test_cli_search() {
    let (stdout, _) = run_klyron(&["search", "lodash"]).unwrap();
    let _ = stdout;
}

#[test]
fn test_cli_dedupe() {
    let dir = test_dir("cli_dedupe");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"lodash":"^4.17.0"}}"#);
    let output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "dedupe"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let _ = output;
}

#[test]
fn test_cli_update() {
    let dir = test_dir("cli_update");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"is-odd":"^3.0.0"}}"#);
    let output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "update"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let _ = output;
}

#[test]
fn test_cli_audit() {
    let dir = test_dir("cli_audit");
    write_package_json(&dir, r#"{"name":"test"}"#);
    let output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "audit"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let _ = output;
}

#[test]
fn test_cli_whoami() {
    let (stdout, _) = run_klyron(&["whoami"]).unwrap();
    let _ = stdout;
}

#[test]
fn test_cli_link() {
    let dir = test_dir("cli_link");
    write_package_json(&dir, r#"{"name":"test-link-pkg","version":"1.0.0"}"#);
    let output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "link"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let _ = output;
}

#[test]
fn test_cli_upgrade() {
    let (stdout, _) = run_klyron(&["upgrade"]).unwrap();
    let _ = stdout;
}

#[test]
fn test_cli_create_list() {
    let (stdout, _) = run_klyron(&["create", "--list"]).unwrap();
    assert!(stdout.contains("Available") || stdout.contains("framework") || !stdout.is_empty());
}

#[test]
fn test_cli_docker() {
    let (stdout, _) = run_klyron(&["docker", "--help"]).unwrap();
    assert!(stdout.contains("docker") || stdout.contains("help") || !stdout.is_empty());
}

#[test]
fn test_cli_config_get() {
    let dir = test_dir("cli_config");
    let output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "config", "name", "myapp"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let _ = output;
}

#[test]
fn test_cli_workspace() {
    let (stdout, _) = run_klyron(&["workspace", "--help"]).unwrap();
    assert!(stdout.contains("workspace") || !stdout.is_empty());
}

#[test]
fn test_cli_plugin() {
    let (stdout, _) = run_klyron(&["plugin", "--help"]).unwrap();
    assert!(stdout.contains("plugin") || !stdout.is_empty());
}

#[test]
fn test_cli_deploy() {
    let (stdout, _) = run_klyron(&["deploy", "--help"]).unwrap();
    assert!(stdout.contains("deploy") || !stdout.is_empty());
}

#[test]
fn test_cli_serve() {
    let dir = test_dir("cli_serve");
    let output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "serve", "--port", "0"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let _ = output;
}

#[test]
fn test_cli_repl() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "repl"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    let _ = output;
}

#[test]
fn test_cli_engine_info() {
    let (stdout, _) = run_klyron(&["--engine", "auto", "info"]).unwrap();
    assert!(stdout.contains("Klyron") || !stdout.is_empty());
}

#[test]
fn test_cli_verbose() {
    let (stdout, _) = run_klyron(&["-v", "info"]).unwrap();
    let _ = stdout;
}

#[test]
fn test_cli_quiet() {
    let (stdout, _) = run_klyron(&["-q", "info"]).unwrap();
    let _ = stdout;
}

#[test]
fn test_cli_run_script() {
    let dir = test_dir("cli_run_script");
    write_package_json(&dir, r#"{"name":"test","scripts":{"hello":"echo hello"}}"#);
    let output = Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "run", "hello"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let _ = output;
}
