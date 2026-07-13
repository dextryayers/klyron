//! PhpProcessEngine — lightweight PHP execution via subprocess.
//!
//! Spawns the system `php` CLI for every invocation.  Variables are kept
//! in an in-process `SharedState` HashMap and injected into the PHP runtime
//! before each `execute_code` / `execute_file` call.
//!
//! This is the simplest backend and requires no WASM binary or C extension.

use crate::{build_injection_prefix, php_bin, run_php_file, run_php_r, PhpConfig, PhpEngine, PhpResult, SharedState};

/// Subprocess-based PHP engine.
///
/// Every `PhpEngine` method delegates to the system `php` CLI.  No
/// WASM or native extension needed.
pub struct PhpProcessEngine {
  config: PhpConfig,
  state: SharedState,
}

impl PhpProcessEngine {
  pub fn new(config: PhpConfig) -> Self {
    Self { config, state: SharedState::new() }
  }
}

impl PhpEngine for PhpProcessEngine {
  fn execute_file(&self, path: &str, args: &[String]) -> Result<PhpResult, String> {
    let injection = build_injection_prefix(&self.state);

    if injection.is_empty() {
      let (stdout, stderr, exit_code) = run_php_file(php_bin(&self.config), path, args, &self.config)?;
      return Ok(PhpResult { stdout, stderr, exit_code, output_vars: Default::default() });
    }

    let wrapper = format!("{}{}", injection, format!("include '{}';", path.replace('\'', "\\'")));
    let tmp = std::env::temp_dir().join(format!("klyron_proc_{}.php", std::process::id()));
    std::fs::write(&tmp, &wrapper).map_err(|e| format!("write wrapper: {e}"))?;

    let (stdout, stderr, exit_code) = run_php_file(php_bin(&self.config), tmp.to_str().unwrap(), args, &self.config)?;
    let _ = std::fs::remove_file(&tmp);
    Ok(PhpResult { stdout, stderr, exit_code, output_vars: Default::default() })
  }

  fn execute_code(&self, code: &str) -> Result<PhpResult, String> {
    let injection = build_injection_prefix(&self.state);
    let full = format!("{}{}", injection, code);
    let (stdout, stderr, exit_code) = run_php_r(php_bin(&self.config), &full, &self.config)?;
    Ok(PhpResult { stdout, stderr, exit_code, output_vars: Default::default() })
  }

  fn call_function(&self, name: &str, args: &[serde_json::Value]) -> Result<serde_json::Value, String> {
    let args_json = serde_json::to_string(args).map_err(|e| format!("serialize args: {e}"))?;
    let escaped_name = name.replace('\'', "\\'");
    let escaped_json = args_json.replace('\'', "\\'");
    let code = format!(
      "echo json_encode(call_user_func_array('{}', json_decode('{}', true) ?: []));",
      escaped_name, escaped_json
    );
    let (stdout, stderr, exit_code) = run_php_r(php_bin(&self.config), &code, &self.config)?;
    if exit_code != 0 {
      return Err(format!("call_function '{}' failed: {}", name, stderr.trim()));
    }
    serde_json::from_str(stdout.trim()).map_err(|e| format!("parse result: {e}"))
  }

  fn evaluate(&self, expr: &str) -> Result<serde_json::Value, String> {
    let escaped = expr.replace('\'', "\\'");
    let code = format!("echo json_encode(eval('return {};'));", escaped);
    let (stdout, stderr, exit_code) = run_php_r(php_bin(&self.config), &code, &self.config)?;
    if exit_code != 0 {
      return Err(format!("evaluate failed: {}", stderr.trim()));
    }
    serde_json::from_str(stdout.trim()).map_err(|e| format!("parse result: {e}"))
  }

  fn set_variable(&self, name: &str, value: serde_json::Value) -> Result<(), String> {
    self.state.set(name, value);
    Ok(())
  }

  fn get_variable(&self, name: &str) -> Result<serde_json::Value, String> {
    self.state.get(name).ok_or_else(|| format!("variable '{}' not set", name))
  }
}
