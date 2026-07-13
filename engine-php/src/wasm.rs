//! PHP-WASM bridge — runs PHP 8.x compiled to WebAssembly via wasmtime.
//!
//! Uses WASIp1 (Preview 1) for core wasm module support.
//!
//! Design:
//! - Uses wasmtime to instantiate a PHP WASM module
//! - PHP's C code (Zend engine) is compiled to WASM via Emscripten
//! - WASI provides POSIX-like I/O inside the WASM sandbox
//! - Communication: stdin/stdout JSON protocol between JS ↔ PHP
//!
//! Current status: **skeleton** — requires a compiled php.wasm binary.
//! The actual PHP-WASM compilation is done externally:
//!   https://github.com/oraoto/pib
//!   https://github.com/seanmorris/php-wasm

use std::sync::Mutex;

use crate::{PhpConfig, PhpEngine, PhpResult, SharedState};

/// PHP-WASM engine backed by wasmtime runtime
pub struct PhpWasmEngine {
  config: PhpConfig,
  state: SharedState,
  _instance: Mutex<Option<wasmtime::Instance>>,
}

impl PhpWasmEngine {
  pub fn new(config: PhpConfig) -> Self {
    Self {
      _instance: Mutex::new(None),
      state: SharedState::new(),
      config,
    }
  }

  /// Initialize the WASM runtime and instantiate the PHP module.
  /// Requires `php.wasm` at the configured path or bundled with Klyron.
  pub fn init(&self) -> Result<(), String> {
    use wasmtime_wasi::preview1;

    let php_wasm_path = self.config.php_path.as_deref().unwrap_or("/usr/lib/klyron/php.wasm");
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::from_file(&engine, php_wasm_path)
      .map_err(|e| format!("Failed to load PHP-WASM module: {e}"))?;

    let mut linker: wasmtime::Linker<preview1::WasiP1Ctx> = wasmtime::Linker::new(&engine);
    preview1::add_to_linker_sync(&mut linker, |t| t)
      .map_err(|e| format!("WASI linker setup: {e}"))?;

    let wasi_ctx = wasmtime_wasi::WasiCtxBuilder::new()
      .inherit_stdio()
      .build_p1();

    let mut store = wasmtime::Store::new(&engine, wasi_ctx);
    let instance = linker.instantiate(&mut store, &module)
      .map_err(|e| format!("PHP-WASM instance: {e}"))?;

    let mut guard = self._instance.lock().map_err(|e| format!("lock: {e}"))?;
    *guard = Some(instance);
    Ok(())
  }
}

impl PhpEngine for PhpWasmEngine {
  fn execute_file(&self, _path: &str, _args: &[String]) -> Result<PhpResult, String> {
    Err("PHP-WASM execute_file: not yet implemented (requires php.wasm)".to_string())
  }

  fn execute_code(&self, _code: &str) -> Result<PhpResult, String> {
    Err("PHP-WASM execute_code: not yet implemented (requires php.wasm)".to_string())
  }

  fn call_function(&self, _name: &str, _args: &[serde_json::Value]) -> Result<serde_json::Value, String> {
    Err("PHP-WASM call_function: not yet implemented".to_string())
  }

  fn evaluate(&self, _expr: &str) -> Result<serde_json::Value, String> {
    Err("PHP-WASM evaluate: not yet implemented".to_string())
  }

  fn set_variable(&self, _name: &str, _value: serde_json::Value) -> Result<(), String> {
    Err("PHP-WASM set_variable: not yet implemented".to_string())
  }

  fn get_variable(&self, _name: &str) -> Result<serde_json::Value, String> {
    Err("PHP-WASM get_variable: not yet implemented".to_string())
  }
}
