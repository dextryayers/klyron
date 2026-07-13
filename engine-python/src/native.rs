//! Native Python engine via PyO3.
//!
//! Embeds CPython directly into the Klyron process using PyO3.
//! Requires libpython3.x to be installed on the system.
//!
//! Advantages over WASM:
//! - Full CPython compatibility (all native extensions work)
//! - Zero-copy data sharing between Rust and Python
//! - Sync/async integration with Tokio
//! - No WASM overhead
//!
//! Current status: **skeleton** — PyO3 is optional, enabled with `--features native`.

use crate::{PythonConfig, PythonEngine, PythonResult, SharedState};

/// Native Python engine backed by PyO3
pub struct PythonNativeEngine {
  config: PythonConfig,
  state: SharedState,
}

impl PythonNativeEngine {
  pub fn new(config: PythonConfig) -> Self {
    Self { config, state: SharedState::new() }
  }

  /// Acquire the Python GIL and return a Python interpreter handle.
  /// Uses PyO3's `Python::with_gil` for thread-safe access.
  pub fn with_gil<F, R>(&self, f: F) -> Result<R, String>
  where
    F: FnOnce(pyo3::Python<'_>) -> Result<R, String>,
  {
    pyo3::Python::with_gil(|py| {
      // Set up sys.path with configured paths
      if let Err(e) = py.run(&format!(
        "import sys\nsys.path.extend({:?})",
        self.config.python_paths
      ), None, None) {
        tracing::warn!("Failed to set up Python path: {e}");
      }
      f(py)
    })
  }
}

impl PythonEngine for PythonNativeEngine {
  fn execute_file(&self, path: &str, args: &[String]) -> Result<PythonResult, String> {
    self.with_gil(|py| {
      let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {path}: {e}"))?;
      match py.run(&code, None, None) {
        Ok(_) => Ok(PythonResult {
          stdout: String::new(),
          stderr: String::new(),
          exit_code: 0,
          output_vars: std::collections::HashMap::new(),
        }),
        Err(e) => Ok(PythonResult {
          stdout: String::new(),
          stderr: format!("{e}"),
          exit_code: 1,
          output_vars: std::collections::HashMap::new(),
        }),
      }
    })
  }

  fn execute_code(&self, code: &str) -> Result<PythonResult, String> {
    self.with_gil(|py| {
      match py.run(code, None, None) {
        Ok(_) => Ok(PythonResult {
          stdout: String::new(),
          stderr: String::new(),
          exit_code: 0,
          output_vars: std::collections::HashMap::new(),
        }),
        Err(e) => Ok(PythonResult {
          stdout: String::new(),
          stderr: format!("{e}"),
          exit_code: 1,
          output_vars: std::collections::HashMap::new(),
        }),
      }
    })
  }

  fn call_function(&self, name: &str, args: &[serde_json::Value]) -> Result<serde_json::Value, String> {
    self.with_gil(|py| {
      let locals = pyo3::types::PyDict::new(py);
      let args_json = serde_json::to_string(args).map_err(|e| format!("serialize args: {e}"))?;
      py.run(
        &format!("import json, sys\n__klyron_result = None\ntry:\n  __klyron_result = {name}(*json.loads({args_json:?}))\nexcept Exception as e:\n  sys.stderr.write(str(e))\n  __klyron_result = None"),
        None,
        Some(locals),
      ).map_err(|e| format!("Python call error: {e}"))?;

      if let Ok(val) = locals.get_item("__klyron_result") {
        if let Ok(v) = val.and_then(|v| v.extract::<String>()) {
          return Ok(serde_json::Value::String(v));
        }
      }
      Ok(serde_json::Value::Null)
    })
  }

  fn evaluate(&self, expr: &str) -> Result<serde_json::Value, String> {
    self.with_gil(|py| {
      let locals = pyo3::types::PyDict::new(py);
      py.run(
        &format!("__klyron_eval = {expr}"),
        None,
        Some(locals),
      ).map_err(|e| format!("Python eval error: {e}"))?;

      if let Ok(Some(val)) = locals.get_item("__klyron_eval").and_then(|v| v.extract::<String>().ok()) {
        return Ok(serde_json::Value::String(val));
      }
      Ok(serde_json::Value::Null)
    })
  }

  fn set_variable(&self, name: &str, value: serde_json::Value) -> Result<(), String> {
    self.state.set(name, value);
    Ok(())
  }

  fn get_variable(&self, name: &str) -> Result<serde_json::Value, String> {
    self.state.get(name).ok_or_else(|| format!("Variable '{name}' not found"))
  }
}
