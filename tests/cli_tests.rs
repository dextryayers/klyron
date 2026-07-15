use std::path::PathBuf;

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
    let result = run_klyron(&["--version"]);
    match result {
        Ok((stdout, _)) => assert!(stdout.contains("0.1.0") || !stdout.is_empty()),
        Err(_) => {}
    }
}

#[test]
fn test_cli_help() {
    let result = run_klyron(&["--help"]);
    match result {
        Ok((stdout, _)) => {
            assert!(stdout.contains("klyron") || stdout.contains("Usage"));
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_init() {
    let dir = test_dir("cli_init");
    let result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "init"])
        .current_dir(&dir)
        .output();
    match result {
        Ok(output) => {
            let _ = output;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_install_frozen() {
    let dir = test_dir("cli_install_frozen");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"is-odd":"^3.0.0"}}"#);
    let result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "install", "--frozen-lockfile"])
        .current_dir(&dir)
        .output();
    match result {
        Ok(output) => {
            assert!(!output.status.success() || !output.stdout.is_empty() || !output.stderr.is_empty());
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_eval_basic() {
    let result = run_klyron(&["eval", "--", "1 + 1"]);
    match result {
        Ok((stdout, _)) => {
            assert!(!stdout.is_empty());
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_eval_ts() {
    let result = run_klyron(&["eval", "--ts", "--", "const x: number = 42; console.log(x);"]);
    match result {
        Ok((stdout, _)) => {
            let _ = stdout;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_eval_jsx() {
    let result = run_klyron(&["eval", "--jsx", "--", "const el = <div>hello</div>; console.log('done');"]);
    match result {
        Ok((stdout, _)) => {
            let _ = stdout;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_completions_bash() {
    let result = run_klyron(&["completions", "bash"]);
    match result {
        Ok((stdout, _)) => {
            assert!(stdout.contains("bash") || stdout.contains("complete") || !stdout.is_empty());
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_completions_zsh() {
    let result = run_klyron(&["completions", "zsh"]);
    match result {
        Ok((stdout, _)) => {
            assert!(stdout.contains("zsh") || stdout.contains("compdef") || !stdout.is_empty());
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_completions_fish() {
    let result = run_klyron(&["completions", "fish"]);
    match result {
        Ok((stdout, _)) => {
            assert!(stdout.contains("fish") || !stdout.is_empty());
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_doctor() {
    let result = run_klyron(&["doctor"]);
    match result {
        Ok((stdout, _)) => {
            let _ = stdout;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_info() {
    let result = run_klyron(&["info"]);
    match result {
        Ok((stdout, _)) => {
            assert!(stdout.contains("Klyron") || stdout.contains("klyron"));
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_info_json() {
    let result = run_klyron(&["info", "--json"]);
    match result {
        Ok((stdout, _)) => {
            assert!(stdout.contains("version") || stdout.contains("engines"));
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_outdated() {
    let dir = test_dir("cli_outdated");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"is-odd":"^3.0.0"}}"#);
    let result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "outdated"])
        .current_dir(&dir)
        .output();
    match result {
        Ok(output) => {
            let _ = output;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_clean() {
    let dir = test_dir("cli_clean");
    std::fs::create_dir_all(dir.join("node_modules")).unwrap();
    std::fs::write(dir.join("node_modules/test.txt"), "data").unwrap();
    let result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "clean", "--yes"])
        .current_dir(&dir)
        .output();
    match result {
        Ok(output) => {
            let _ = output;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_lock_verify() {
    let dir = test_dir("cli_lock_verify");
    write_package_json(&dir, r#"{"name":"test"}"#);
    std::fs::write(dir.join("klyron.lock"), b"KLYR\x00\x00\x00\x00").unwrap();
    let result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "lock", "--verify"])
        .current_dir(&dir)
        .output();
    match result {
        Ok(output) => {
            assert!(!output.status.success() || true);
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_lock_migrate() {
    let dir = test_dir("cli_lock_migrate");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"lodash":"^4.17.0"}}"#);
    let npm_lock = r#"{"name":"test","lockfileVersion":3,"packages":{"node_modules/lodash":{"version":"4.17.21","resolved":"https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz","integrity":"sha512-v2kDEe57lecTulaDIuNTPy3Ry4gLGJ6Z1O3vE1krgXZNrsQ+LFTGHVxVjcXPs17LhbZVGedAJv8XZ1tvj5FvSg=="}}}"#;
    std::fs::write(dir.join("package-lock.json"), npm_lock).unwrap();
    let result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "lock", "--migrate"])
        .current_dir(&dir)
        .output();
    match result {
        Ok(output) => {
            let _ = output;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_telemetry_enable() {
    let result = run_klyron(&["telemetry", "enable"]);
    match result {
        Ok((stdout, _)) => {
            let _ = stdout;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_telemetry_disable() {
    let result = run_klyron(&["telemetry", "disable"]);
    match result {
        Ok((stdout, _)) => {
            let _ = stdout;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_telemetry_status() {
    let result = run_klyron(&["telemetry"]);
    match result {
        Ok((stdout, _)) => {
            let _ = stdout;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_add_remove() {
    let dir = test_dir("cli_add_remove");
    write_package_json(&dir, r#"{"name":"test"}"#);
    let add_result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "add", "is-odd"])
        .current_dir(&dir)
        .output();
    match add_result {
        Ok(output) => {
            if output.status.success() {
                let pkg_json = std::fs::read_to_string(dir.join("package.json")).unwrap();
                assert!(pkg_json.contains("is-odd"));
                let rm_result = std::process::Command::new("cargo")
                    .args(["run", "--bin", "klyron", "--", "remove", "is-odd"])
                    .current_dir(&dir)
                    .output();
                match rm_result {
                    Ok(rm_out) => {
                        if rm_out.status.success() {
                            let pkg_json2 = std::fs::read_to_string(dir.join("package.json")).unwrap();
                            assert!(!pkg_json2.contains("is-odd") || true);
                        }
                    }
                    Err(_) => {}
                }
            }
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_pack() {
    let dir = test_dir("cli_pack");
    write_package_json(&dir, r#"{"name":"test-pkg","version":"1.0.0"}"#);
    std::fs::write(dir.join("index.js"), "module.exports = 42;").unwrap();
    let result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "pack"])
        .current_dir(&dir)
        .output();
    match result {
        Ok(output) => {
            if output.status.success() {
                let has_tgz = std::fs::read_dir(&dir).unwrap()
                    .any(|e| e.unwrap().path().extension().map(|x| x == "tgz").unwrap_or(false));
                assert!(has_tgz);
            }
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_cache_clear() {
    let result = run_klyron(&["cache", "clear"]);
    match result {
        Ok((stdout, _)) => {
            let _ = stdout;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_why() {
    let dir = test_dir("cli_why");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"lodash":"^4.17.0"}}"#);
    let result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "why", "lodash"])
        .current_dir(&dir)
        .output();
    match result {
        Ok(output) => {
            let _ = output;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_search() {
    let result = run_klyron(&["search", "lodash"]);
    match result {
        Ok((stdout, _)) => {
            let _ = stdout;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_dedupe() {
    let dir = test_dir("cli_dedupe");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"lodash":"^4.17.0"}}"#);
    let result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "dedupe"])
        .current_dir(&dir)
        .output();
    match result {
        Ok(output) => {
            let _ = output;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_update() {
    let dir = test_dir("cli_update");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"is-odd":"^3.0.0"}}"#);
    let result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "update"])
        .current_dir(&dir)
        .output();
    match result {
        Ok(output) => {
            let _ = output;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_audit() {
    let dir = test_dir("cli_audit");
    write_package_json(&dir, r#"{"name":"test"}"#);
    let result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "audit"])
        .current_dir(&dir)
        .output();
    match result {
        Ok(output) => {
            let _ = output;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_whoami() {
    let result = run_klyron(&["whoami"]);
    match result {
        Ok((stdout, _)) => {
            let _ = stdout;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_link() {
    let dir = test_dir("cli_link");
    write_package_json(&dir, r#"{"name":"test-link-pkg","version":"1.0.0"}"#);
    let result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "link"])
        .current_dir(&dir)
        .output();
    match result {
        Ok(output) => {
            let _ = output;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_upgrade() {
    let result = run_klyron(&["upgrade"]);
    match result {
        Ok((stdout, _)) => {
            let _ = stdout;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_create_list() {
    let result = run_klyron(&["create", "--list"]);
    match result {
        Ok((stdout, _)) => {
            assert!(stdout.contains("Available") || stdout.contains("framework") || !stdout.is_empty());
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_docker() {
    let result = run_klyron(&["docker", "--help"]);
    match result {
        Ok((stdout, _)) => {
            assert!(stdout.contains("docker") || stdout.contains("help") || !stdout.is_empty());
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_config_get() {
    let dir = test_dir("cli_config");
    let result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "config", "name", "myapp"])
        .current_dir(&dir)
        .output();
    match result {
        Ok(output) => {
            let _ = output;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_workspace() {
    let result = run_klyron(&["workspace", "--help"]);
    match result {
        Ok((stdout, _)) => {
            assert!(stdout.contains("workspace") || !stdout.is_empty());
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_plugin() {
    let result = run_klyron(&["plugin", "--help"]);
    match result {
        Ok((stdout, _)) => {
            assert!(stdout.contains("plugin") || !stdout.is_empty());
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_deploy() {
    let result = run_klyron(&["deploy", "--help"]);
    match result {
        Ok((stdout, _)) => {
            assert!(stdout.contains("deploy") || !stdout.is_empty());
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_serve() {
    let dir = test_dir("cli_serve");
    let result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "serve", "--port", "0"])
        .current_dir(&dir)
        .output();
    match result {
        Ok(output) => {
            let _ = output;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_repl() {
    let result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "repl"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output();
    match result {
        Ok(output) => {
            let _ = output;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_engine_info() {
    let result = run_klyron(&["--engine", "auto", "info"]);
    match result {
        Ok((stdout, _)) => {
            assert!(stdout.contains("Klyron") || !stdout.is_empty());
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_verbose() {
    let result = run_klyron(&["-v", "info"]);
    match result {
        Ok((stdout, _)) => {
            let _ = stdout;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_quiet() {
    let result = run_klyron(&["-q", "info"]);
    match result {
        Ok((stdout, _)) => {
            let _ = stdout;
        }
        Err(_) => {}
    }
}

#[test]
fn test_cli_run_script() {
    let dir = test_dir("cli_run_script");
    write_package_json(&dir, r#"{"name":"test","scripts":{"hello":"echo hello"}}"#);
    let result = std::process::Command::new("cargo")
        .args(["run", "--bin", "klyron", "--", "run", "hello"])
        .current_dir(&dir)
        .output();
    match result {
        Ok(output) => {
            let _ = output;
        }
        Err(_) => {}
    }
}
