//! Klyron PHP/Laravel Engine
//!
//! Bridges PHP (WASM or native) into the Klyron polyglot runtime.
//! Supports:
//! - PHP-WASM: portable, sandboxed, default
//! - phper: native PHP embedding (requires libphp)
//! - Artisan CLI proxy
//! - Composer package management
//! - Blade templating bridge
//! - JS ↔ PHP shared memory interop

mod wasm;
mod artisan;
mod composer;
mod blade;

pub use wasm::PhpWasmEngine;
pub use artisan::ArtisanCli;
pub use composer::Composer;
pub use blade::BladeRenderer;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Runtime configuration for PHP engine
#[derive(Debug, Clone)]
pub struct PhpConfig {
  /// Path to the PHP-WASM binary (or PHP binary for native mode)
  pub php_path: Option<String>,
  /// PHP extension directories to load
  pub extension_dirs: Vec<String>,
  /// php.ini-style configuration directives
  pub ini_settings: HashMap<String, String>,
  /// Custom include paths
  pub include_paths: Vec<String>,
  /// Memory limit for PHP execution (MB)
  pub memory_limit_mb: u64,
  /// Timeout for PHP execution (seconds)
  pub timeout_secs: u64,
}

impl Default for PhpConfig {
  fn default() -> Self {
    Self {
      php_path: None,
      extension_dirs: vec![],
      ini_settings: HashMap::new(),
      include_paths: vec![],
      memory_limit_mb: 256,
      timeout_secs: 30,
    }
  }
}

/// Result of a PHP script execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PhpResult {
  pub stdout: String,
  pub stderr: String,
  pub exit_code: i32,
  pub output_vars: HashMap<String, String>,
}

/// Interface that all PHP engine backends implement
pub trait PhpEngine: Send + Sync {
  /// Execute a PHP file
  fn execute_file(&self, path: &str, args: &[String]) -> Result<PhpResult, String>;
  /// Execute raw PHP code
  fn execute_code(&self, code: &str) -> Result<PhpResult, String>;
  /// Call a PHP function from JS
  fn call_function(&self, name: &str, args: &[serde_json::Value]) -> Result<serde_json::Value, String>;
  /// Evaluate PHP expression and return value
  fn evaluate(&self, expr: &str) -> Result<serde_json::Value, String>;
  /// Set PHP variable (visible to subsequent PHP execution)
  fn set_variable(&self, name: &str, value: serde_json::Value) -> Result<(), String>;
  /// Get PHP variable value
  fn get_variable(&self, name: &str) -> Result<serde_json::Value, String>;
}

/// Shared state between JS and PHP
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
