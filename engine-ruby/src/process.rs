use std::process::Command;

use crate::{RubyConfig, RubyEngine, RubyResult, SharedState};

pub struct RubyProcessEngine {
  config: RubyConfig,
  state: SharedState,
}

impl RubyProcessEngine {
  pub fn new(config: RubyConfig) -> Self {
    Self {
      config,
      state: SharedState::new(),
    }
  }

  fn ruby_bin(&self) -> &str {
    self.config.ruby_path.as_deref().unwrap_or("ruby")
  }

  fn build_cmd(&self) -> Command {
    let mut cmd = Command::new(self.ruby_bin());
    for path in &self.config.load_paths {
      cmd.arg("-I").arg(path);
    }
    for (key, val) in &self.config.env_vars {
      cmd.env(key, val);
    }
    cmd
  }

  fn run_ruby(&self, code: &str) -> Result<RubyResult, String> {
    let output = self.build_cmd()
      .arg("-e")
      .arg(code)
      .output()
      .map_err(|e| format!("Failed to execute Ruby: {e}"))?;

    Ok(RubyResult {
      stdout: String::from_utf8_lossy(&output.stdout).to_string(),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      output_vars: to_string_map(self.state.drain()),
    })
  }
}

fn to_string_map(input: std::collections::HashMap<String, serde_json::Value>) -> std::collections::HashMap<String, String> {
  input.into_iter().map(|(k, v)| (k, serde_json::to_string(&v).unwrap_or_default())).collect()
}

impl RubyEngine for RubyProcessEngine {
  fn execute_code(&self, code: &str) -> Result<RubyResult, String> {
    self.run_ruby(code)
  }

  fn execute_file(&self, path: &str, args: &[String]) -> Result<RubyResult, String> {
    let mut cmd = self.build_cmd();
    cmd.arg(path);
    for a in args {
      cmd.arg(a);
    }
    let output = cmd.output()
      .map_err(|e| format!("Failed to execute Ruby file: {e}"))?;

    Ok(RubyResult {
      stdout: String::from_utf8_lossy(&output.stdout).to_string(),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      output_vars: to_string_map(self.state.drain()),
    })
  }

  fn evaluate(&self, expr: &str) -> Result<serde_json::Value, String> {
    let code = format!(
      r#"require 'json'; begin; result = ({}); puts JSON.generate({{"ok" => result}}); rescue JSON::GeneratorError; puts JSON.generate({{"ok" => result.to_s}}); rescue => e; puts JSON.generate({{"error" => e.message}}); end"#,
      expr,
    );
    let result = self.run_ruby(&code)?;

    if result.exit_code != 0 && result.stdout.trim().is_empty() {
      return Err(format!("Ruby evaluation failed (exit {}): {}", result.exit_code, result.stderr));
    }
    if result.stdout.trim().is_empty() {
      return Err(format!("Ruby evaluation produced no output. stderr: {}", result.stderr));
    }

    let parsed: serde_json::Value = serde_json::from_str(result.stdout.trim())
      .map_err(|e| format!("Failed to parse Ruby result: {e}\nstdout: {}", result.stdout))?;

    if let Some(err) = parsed.get("error").and_then(|v| v.as_str()) {
      return Err(format!("Ruby evaluation error: {err}"));
    }

    parsed.get("ok").cloned().ok_or_else(|| "Ruby evaluation returned no value".to_string())
  }

  fn call_method(&self, receiver: Option<&str>, method: &str, args: &[serde_json::Value]) -> Result<serde_json::Value, String> {
    let args_json = serde_json::to_string(args)
      .map_err(|e| format!("Failed to serialize args: {e}"))?;
    let receiver_expr = receiver.unwrap_or("self");

    let code = format!(
      r#"require 'json'; begin; args = JSON.parse(ENV.fetch('__RUBY_ARGS', '[]')); result = ({}).{}(*args); puts JSON.generate({{"ok" => result}}); rescue JSON::GeneratorError; puts JSON.generate({{"ok" => result.to_s}}); rescue => e; puts JSON.generate({{"error" => e.message}}); end"#,
      receiver_expr, method,
    );

    let output = self.build_cmd()
      .arg("-e")
      .arg(&code)
      .env("__RUBY_ARGS", &args_json)
      .output()
      .map_err(|e| format!("Failed to call Ruby method: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if !output.status.success() && stdout.trim().is_empty() {
      return Err(format!("Ruby method call failed (exit {}): {}", output.status.code().unwrap_or(-1), String::from_utf8_lossy(&output.stderr)));
    }

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
      .map_err(|e| format!("Failed to parse result: {e}\nstdout: {stdout}"))?;

    if let Some(err) = parsed.get("error").and_then(|v| v.as_str()) {
      return Err(format!("Ruby method call error: {err}"));
    }

    parsed.get("ok").cloned().ok_or_else(|| "Ruby method returned no value".to_string())
  }

  fn set_variable(&self, name: &str, value: serde_json::Value) -> Result<(), String> {
    self.state.set(name, value);
    Ok(())
  }

  fn get_variable(&self, name: &str) -> Result<serde_json::Value, String> {
    self.state.get(name).ok_or_else(|| format!("Variable '{name}' not set"))
  }
}
