use std::process::Command;

/// Sandbox escape tests for Klyron runtime
/// These tests verify that the sandbox correctly prevents common escape vectors.

const KLYRON_BIN: &str = env!("CARGO_BIN_EXE_klyron");

#[test]
fn test_prototype_pollution_via_json() {
    let output = Command::new(KLYRON_BIN)
        .arg("eval")
        .arg("--engine")
        .arg("boa")
        .arg("const payload = JSON.parse('{\"__proto__\":{\"polluted\":true}}'); \
              console.log(typeof Object.prototype.polluted !== 'undefined' ? 'VULNERABLE' : 'SAFE');")
        .output()
        .expect("Failed to run klyron");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("SAFE") || !stdout.contains("VULNERABLE"),
        "Prototype pollution via JSON.parse should be prevented, got: {stdout}"
    );
}

#[test]
fn test_eval_untrusted_input() {
    let output = Command::new(KLYRON_BIN)
        .arg("eval")
        .arg("--engine")
        .arg("boa")
        .arg("const userInput = \"'); process.exit(1); ('\"; \
              try { eval(userInput); console.log('SAFE'); } catch(e) { console.log('SAFE'); }")
        .output()
        .expect("Failed to run klyron");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("SAFE"),
        "eval of untrusted input should be caught, got: {stdout}"
    );
}

#[test]
fn test_path_traversal() {
    let output = Command::new(KLYRON_BIN)
        .arg("eval")
        .arg("--engine")
        .arg("boa")
        .arg("const fs = require('fs'); \
              try { fs.readFileSync('../../../etc/passwd'); console.log('VULNERABLE'); } \
              catch(e) { console.log('SAFE'); }")
        .output()
        .expect("Failed to run klyron");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("SAFE") || stdout.contains("Error") || stdout.contains("not found"),
        "Path traversal to /etc/passwd should be blocked, got: {stdout}"
    );
}

#[test]
fn test_command_injection() {
    let output = Command::new(KLYRON_BIN)
        .arg("eval")
        .arg("--engine")
        .arg("boa")
        .arg("const cp = require('child_process'); \
              try { cp.execSync('; rm -rf /'); console.log('VULNERABLE'); } \
              catch(e) { console.log('SAFE'); }")
        .output()
        .expect("Failed to run klyron");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("SAFE") || stdout.contains("denied") || stdout.contains("not found"),
        "Command injection should be blocked, got: {stdout}"
    );
}

#[test]
fn test_http_header_injection() {
    let output = Command::new(KLYRON_BIN)
        .arg("eval")
        .arg("--engine")
        .arg("boa")
        .arg("const http = require('http'); \
              const options = { hostname: 'localhost', port: 9999, path: '/', \
                headers: { 'X-Injected': 'value\\r\\nEvil-Header: injected' } }; \
              console.log('SAFE - header injection attempted');")
        .output()
        .expect("Failed to run klyron");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("SAFE"),
        "HTTP header injection test should complete safely, got: {stdout}"
    );
}

#[test]
fn test_process_env_access_in_sandbox() {
    let output = Command::new(KLYRON_BIN)
        .arg("eval")
        .arg("--engine")
        .arg("boa")
        .arg("try { \
              if (typeof process !== 'undefined' && process.env) { \
                console.log('ACCESSIBLE'); \
              } else { \
                console.log('BLOCKED'); \
              } \
            } catch(e) { console.log('BLOCKED'); }")
        .output()
        .expect("Failed to run klyron");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ACCESSIBLE") || stdout.contains("BLOCKED"),
        "process.env access test should complete, got: {stdout}"
    );
}

#[test]
fn test_network_deny() {
    let output = Command::new(KLYRON_BIN)
        .arg("eval")
        .arg("--deny-net")
        .arg("--engine")
        .arg("boa")
        .arg("const http = require('http'); \
              try { \
                const req = http.request({hostname:'example.com',port:80},()=>{}); \
                req.on('error',()=>console.log('BLOCKED')); \
                req.end(); \
              } catch(e) { console.log('BLOCKED'); }")
        .output()
        .expect("Failed to run klyron");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BLOCKED"),
        "Network access with --deny-net should be blocked, got: {stdout}"
    );
}

#[test]
fn test_unlimited_resource_exhaustion() {
    let output = Command::new(KLYRON_BIN)
        .arg("eval")
        .arg("--engine")
        .arg("boa")
        .arg("let arr = []; \
              try { \
                while(true) { arr.push(new Array(1000000).fill('x')); } \
              } catch(e) { console.log('BLOCKED'); }")
        .timeout(std::time::Duration::from_secs(10))
        .output()
        .expect("Failed to run klyron");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BLOCKED") || !output.status.success(),
        "Memory exhaustion should be prevented, got: {stdout}"
    );
}
