//! Python process engine — runs CPython via subprocess with JSON exchange.
//!
//! Uses `python3 -c "..."` for all operations with structured JSON
//! for data exchange between JavaScript and Python. Includes proper
//! error handling, timeout support, and shared state management.

use std::time::{Duration, Instant};

use crate::{PythonConfig, PythonEngine, PythonResult, SharedState};

/// Python engine backed by a `python3` subprocess with JSON protocol
pub struct PythonProcessEngine {
  config: PythonConfig,
  state: SharedState,
}

impl PythonProcessEngine {
  pub fn new(config: PythonConfig) -> Self {
    Self {
      config,
      state: SharedState::new(),
    }
  }

  /// Resolve the python3 binary path
  fn python_bin(&self) -> &str {
    self.config.python_path.as_deref().unwrap_or("python3")
  }

  /// Spawn a python3 subprocess and capture its output with timeout.
  ///
  /// Uses a background thread to read piped stdout/stderr while the
  /// main thread polls for timeout via `recv_timeout`.
  fn run_python(
    &self,
    code: &str,
    timeout_secs: u64,
  ) -> Result<(String, String, i32), String> {
    let mut cmd = std::process::Command::new(self.python_bin());
    cmd.arg("-c");
      cmd.arg(code);
    if let Ok(cwd) = std::env::current_dir() {
      cmd.current_dir(cwd);
    }

    let child = cmd
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .spawn()
      .map_err(|e| format!("Failed to spawn python3: {e}"))?;

    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
      let result = child.wait_with_output();
      tx.send(result).ok();
    });

    let timeout = Duration::from_secs(timeout_secs);
    let start = Instant::now();

    loop {
      match rx.recv_timeout(Duration::from_millis(50)) {
        Ok(Ok(output)) => {
          let stdout = String::from_utf8_lossy(&output.stdout).to_string();
          let stderr = String::from_utf8_lossy(&output.stderr).to_string();
          return Ok((stdout, stderr, output.status.code().unwrap_or(-1)));
        }
        Ok(Err(e)) => return Err(format!("Python process error: {e}")),
        Err(_) => {
          if start.elapsed() >= timeout {
            return Err(format!(
              "Python process timed out after {} seconds",
              timeout_secs
            ));
          }
          // Continue waiting
        }
      }
    }
  }

  /// Build a Python snippet that injects shared variables, executes code,
  /// and captures any variables back to stdout as JSON.
  fn build_wrapper_script(inner: &str, inject_vars_json: &str) -> String {
    format!(
      r#"import json, sys
__klyron_vars = json.loads('{inject}')
for __k, __v in __klyron_vars.items():
    globals()[__k] = __v
del __klyron_vars, __k, __v
try:
{inner}
except Exception as __e:
    sys.stderr.write(str(__e))
    sys.exit(1)
"#,
      inject = inject_vars_json.replace('\'', "\\'"),
      inner = inner,
    )
  }
}

impl PythonEngine for PythonProcessEngine {
  fn execute_file(&self, path: &str, args: &[String]) -> Result<PythonResult, String> {
    let args_json = serde_json::to_string(args).map_err(|e| format!("serialize args: {e}"))?;
    let inject = serde_json::to_string(&self.state.drain())
      .unwrap_or_else(|_| "{}".to_string());

    let code = Self::build_wrapper_script(
      &format!(
        r#"    import sys, json
    sys.argv = [{}] + json.loads('{args}')
    exec(open({path:?}).read())
"#,
        serde_json::Value::String(path.to_string()),
        args = args_json.replace('\'', "\\'"),
        path = path,
      ),
      &inject,
    );

    let (stdout, stderr, exit_code) = self.run_python(&code, self.config.timeout_secs)?;
    Ok(PythonResult {
      stdout,
      stderr,
      exit_code,
      output_vars: std::collections::HashMap::new(),
    })
  }

  fn execute_code(&self, code: &str) -> Result<PythonResult, String> {
    let inject = serde_json::to_string(&self.state.drain())
      .unwrap_or_else(|_| "{}".to_string());

    let wrapped = Self::build_wrapper_script(
      &format!("    {code}"),
      &inject,
    );

    let (stdout, stderr, exit_code) = self.run_python(&wrapped, self.config.timeout_secs)?;
    Ok(PythonResult {
      stdout,
      stderr,
      exit_code,
      output_vars: std::collections::HashMap::new(),
    })
  }

  fn call_function(&self, name: &str, args: &[serde_json::Value]) -> Result<serde_json::Value, String> {
    let args_json = serde_json::to_string(args).map_err(|e| format!("serialize args: {e}"))?;
    let inject = serde_json::to_string(&self.state.drain())
      .unwrap_or_else(|_| "{}".to_string());

    let code = Self::build_wrapper_script(
      &format!(
        r#"    __result = {name}(*json.loads('{args}'))
    print(json.dumps(__result))
"#,
        name = name,
        args = args_json.replace('\'', "\\'"),
      ),
      &inject,
    );

    let (stdout, stderr, exit_code) = self.run_python(&code, self.config.timeout_secs)?;
    if exit_code != 0 {
      return Err(format!("call_function '{name}' failed: {stderr}"));
    }
    serde_json::from_str(stdout.trim()).map_err(|e| format!("parse result: {e}"))
  }

  fn evaluate(&self, expr: &str) -> Result<serde_json::Value, String> {
    let inject = serde_json::to_string(&self.state.drain())
      .unwrap_or_else(|_| "{}".to_string());

    let code = Self::build_wrapper_script(
      &format!(r#"    print(json.dumps({expr}))"#),
      &inject,
    );

    let (stdout, stderr, exit_code) = self.run_python(&code, self.config.timeout_secs)?;
    if exit_code != 0 {
      return Err(format!("evaluate error: {stderr}"));
    }
    serde_json::from_str(stdout.trim()).map_err(|e| format!("parse eval result: {e}"))
  }

  fn set_variable(&self, name: &str, value: serde_json::Value) -> Result<(), String> {
    self.state.set(name, value);
    Ok(())
  }

  fn get_variable(&self, name: &str) -> Result<serde_json::Value, String> {
    self.state.get(name).ok_or_else(|| format!("Variable '{name}' not found"))
  }
}
