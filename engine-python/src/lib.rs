//! Klyron Python Engine
//!
//! Bridges Python (PyO3 native, subprocess, or WASM) into the Klyron polyglot runtime.
//! Supports:
//! - PythonWasmEngine: subprocess-based engine (default, "wasm" feature)
//! - PythonProcessEngine: subprocess engine with JSON exchange & timeout
//! - PythonNativeEngine: PyO3 native embedding (requires "native" feature)
//! - pip integration (PyPI registry, requirements.txt, venv)
//! - Django management commands
//! - FastAPI / Uvicorn serving
//! - Flask development server
//! - JS ↔ Python bridge via shared memory

mod wasm;
mod process;
#[cfg(feature = "native")]
mod native;
mod pip;
mod django;
mod fastapi;
mod flask;

pub use wasm::PythonWasmEngine;
pub use process::PythonProcessEngine;
#[cfg(feature = "native")]
pub use native::PythonNativeEngine;
pub use pip::PipManager;
pub use django::DjangoCli;
pub use fastapi::FastApiServer;
pub use flask::FlaskServer;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Runtime configuration for Python engine
#[derive(Debug, Clone)]
pub struct PythonConfig {
  /// Path to the Python-WASM binary (or python3 binary for native mode)
  pub python_path: Option<String>,
  /// Additional Python paths (site-packages, etc.)
  pub python_paths: Vec<String>,
  /// Environment variables for Python processes
  pub env_vars: HashMap<String, String>,
  /// Virtual environment directory
  pub venv_dir: Option<String>,
  /// Memory limit for Python execution (MB)
  pub memory_limit_mb: u64,
  /// Timeout for Python execution (seconds)
  pub timeout_secs: u64,
}

impl Default for PythonConfig {
  fn default() -> Self {
    Self {
      python_path: None,
      python_paths: vec![],
      env_vars: HashMap::new(),
      venv_dir: None,
      memory_limit_mb: 512,
      timeout_secs: 60,
    }
  }
}

/// Result of a Python script execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PythonResult {
  pub stdout: String,
  pub stderr: String,
  pub exit_code: i32,
  pub output_vars: HashMap<String, String>,
}

/// Interface that all Python engine backends implement
pub trait PythonEngine: Send + Sync {
  /// Execute a Python file
  fn execute_file(&self, path: &str, args: &[String]) -> Result<PythonResult, String>;
  /// Execute raw Python code
  fn execute_code(&self, code: &str) -> Result<PythonResult, String>;
  /// Call a Python function from JS
  fn call_function(&self, name: &str, args: &[serde_json::Value]) -> Result<serde_json::Value, String>;
  /// Evaluate Python expression and return value
  fn evaluate(&self, expr: &str) -> Result<serde_json::Value, String>;
  /// Set Python variable (visible to subsequent Python execution)
  fn set_variable(&self, name: &str, value: serde_json::Value) -> Result<(), String>;
  /// Get Python variable value
  fn get_variable(&self, name: &str) -> Result<serde_json::Value, String>;
}

/// Shared state between JS and Python
#[derive(Debug, Clone, Default)]
pub struct SharedState {
  variables: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl SharedState {
  pub fn new() -> Self {
    Self { variables: Arc::new(RwLock::new(HashMap::new())) }
  }

  pub fn set(&self, name: &str, value: serde_json::Value) {
    if let Ok(mut vars) = self.variables.write() {
      vars.insert(name.to_string(), value);
    }
  }

  pub fn get(&self, name: &str) -> Option<serde_json::Value> {
    self.variables.read().ok().and_then(|vars| vars.get(name).cloned())
  }

  pub fn drain(&self) -> HashMap<String, serde_json::Value> {
    self.variables.write().map(|mut vars| std::mem::take(&mut *vars)).unwrap_or_default()
  }
}
