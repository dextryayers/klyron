//! Klyron Ruby Engine
//!
//! Bridges Ruby (WASM or native) into the Klyron polyglot runtime.
//! Supports:
//! - Ruby-WASM: MRI compiled to WebAssembly (portable, sandboxed)
//! - gem integration (RubyGems registry, Gemfile, Bundler)
//! - Rails application server + generators
//! - Sinatra development server
//! - Rack-compatible middleware

mod wasm;
mod process;
mod gem;
mod rails;
mod sinatra;

pub use wasm::RubyWasmEngine;
pub use process::RubyProcessEngine;
pub use gem::GemManager;
pub use rails::RailsCli;
pub use sinatra::SinatraServer;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Runtime configuration for Ruby engine
#[derive(Debug, Clone)]
pub struct RubyConfig {
  /// Path to Ruby binary (or ruby.wasm)
  pub ruby_path: Option<String>,
  /// Additional load paths (equivalent to RUBYLIB / -I)
  pub load_paths: Vec<String>,
  /// Environment variables for Ruby processes
  pub env_vars: HashMap<String, String>,
  /// Bundler configuration — use Gemfile?
  pub use_bundler: bool,
  /// Memory limit for Ruby execution (MB)
  pub memory_limit_mb: u64,
  /// Timeout for Ruby execution (seconds)
  pub timeout_secs: u64,
}

impl Default for RubyConfig {
  fn default() -> Self {
    Self {
      ruby_path: None,
      load_paths: vec![],
      env_vars: HashMap::new(),
      use_bundler: true,
      memory_limit_mb: 512,
      timeout_secs: 60,
    }
  }
}

/// Result of a Ruby script execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RubyResult {
  pub stdout: String,
  pub stderr: String,
  pub exit_code: i32,
  pub output_vars: HashMap<String, String>,
}

/// Interface that all Ruby engine backends implement
pub trait RubyEngine: Send + Sync {
  /// Execute a Ruby file
  fn execute_file(&self, path: &str, args: &[String]) -> Result<RubyResult, String>;
  /// Execute raw Ruby code
  fn execute_code(&self, code: &str) -> Result<RubyResult, String>;
  /// Call a Ruby method from JS
  fn call_method(&self, receiver: Option<&str>, method: &str, args: &[serde_json::Value]) -> Result<serde_json::Value, String>;
  /// Evaluate Ruby expression and return value
  fn evaluate(&self, expr: &str) -> Result<serde_json::Value, String>;
  /// Set Ruby global variable
  fn set_variable(&self, name: &str, value: serde_json::Value) -> Result<(), String>;
  /// Get Ruby global variable value
  fn get_variable(&self, name: &str) -> Result<serde_json::Value, String>;
}

/// Shared state between JS and Ruby
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
