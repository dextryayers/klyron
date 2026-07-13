//! Python-WASM engine — runs CPython via subprocess and wasmtime fallback.
//!
//! Primary mode: spawns `python3` CLI as a subprocess.
//! Future mode: instantiate CPython compiled to WASM via wasmtime (init()).
//!
//! Communication: stdin/stdout with JSON protocol for data exchange.

use std::sync::Mutex;
use std::time::Duration;

use crate::{PythonConfig, PythonEngine, PythonResult, SharedState};

/// Placeholder for wasmtime Instance (available when "wasm" feature is enabled)
#[cfg(feature = "wasm")]
type WasmInstanceInner = Option<wasmtime::Instance>;
/// Placeholder when wasm feature is disabled
#[cfg(not(feature = "wasm"))]
type WasmInstanceInner = Option<()>;

/// Python engine backed by subprocess + optional WASM runtime
pub struct PythonWasmEngine {
  config: PythonConfig,
  state: SharedState,
  _instance: Mutex<WasmInstanceInner>,
}

impl PythonWasmEngine {
  pub fn new(config: PythonConfig) -> Self {
    Self {
      _instance: Mutex::new(None),
      state: SharedState::new(),
      config,
    }
  }

  /// Initialize the WASM runtime for future use.
  /// Requires "wasm" feature and a `python.wasm` binary.
  #[cfg(feature = "wasm")]
  pub fn init(&self) -> Result<(), String> {
    use wasmtime_wasi::preview1;

    let wasm_path = self.config.python_path.as_deref().unwrap_or("/usr/lib/klyron/python.wasm");
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::from_file(&engine, wasm_path)
      .map_err(|e| format!("Failed to load Python-WASM module: {e}"))?;

    let mut linker: wasmtime::Linker<preview1::WasiP1Ctx> = wasmtime::Linker::new(&engine);
    preview1::add_to_linker_sync(&mut linker, |t| t)
      .map_err(|e| format!("WASI linker setup: {e}"))?;

    let wasi_ctx = wasmtime_wasi::WasiCtxBuilder::new()
      .inherit_stdio()
      .build_p1();

    let mut store = wasmtime::Store::new(&engine, wasi_ctx);
    let instance = linker.instantiate(&mut store, &module)
      .map_err(|e| format!("Python-WASM instance: {e}"))?;

    let mut guard = self._instance.lock().map_err(|e| format!("lock: {e}"))?;
    *guard = Some(instance);
    Ok(())
  }

  /// Initialize — no-op when wasm feature is disabled
  #[cfg(not(feature = "wasm"))]
  pub fn init(&self) -> Result<(), String> {
    Err("WASM feature not enabled; build with --features wasm to use WASM runtime".to_string())
  }

  /// Resolve the python3 binary path
  fn python_bin(&self) -> &str {
    self.config.python_path.as_deref().unwrap_or("python3")
  }

  /// Spawn a python subprocess with the given args and return captured output
  fn run_python(
    &self,
    args: &[&str],
    stdin: Option<&str>,
    timeout_secs: u64,
  ) -> Result<(String, String, i32), String> {
    let mut cmd = std::process::Command::new(self.python_bin());
    cmd.args(args);
    if let Ok(cwd) = std::env::current_dir() {
      cmd.current_dir(cwd);
    }

    if stdin.is_some() {
      cmd.stdin(std::process::Stdio::piped());
    }

    let mut child = cmd
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .spawn()
      .map_err(|e| format!("Failed to spawn python3: {e}"))?;

    use std::io::Write;
    if let Some(input) = stdin {
      if let Some(mut stdin_handle) = child.stdin.take() {
        stdin_handle.write_all(input.as_bytes()).map_err(|e| format!("stdin write: {e}"))?;
      }
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let _handle = std::thread::spawn(move || {
      let result = child.wait_with_output();
      tx.send(result).ok();
    });

    match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
      Ok(Ok(output)) => {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Ok((stdout, stderr, output.status.code().unwrap_or(-1)))
      }
      Ok(Err(e)) => Err(format!("Python process error: {e}")),
      Err(_) => Err(format!(
        "Python process timed out after {} seconds",
        timeout_secs
      )),
    }
  }
}

impl PythonEngine for PythonWasmEngine {
  fn execute_file(&self, path: &str, args: &[String]) -> Result<PythonResult, String> {
    let mut cmd_args = vec![path];
    for a in args {
      cmd_args.push(a.as_str());
    }
    let (stdout, stderr, exit_code) = self.run_python(&cmd_args, None, self.config.timeout_secs)?;
    Ok(PythonResult {
      stdout,
      stderr,
      exit_code,
      output_vars: std::collections::HashMap::new(),
    })
  }

  fn execute_code(&self, code: &str) -> Result<PythonResult, String> {
    let (stdout, stderr, exit_code) = self.run_python(&["-c", code], None, self.config.timeout_secs)?;
    Ok(PythonResult {
      stdout,
      stderr,
      exit_code,
      output_vars: std::collections::HashMap::new(),
    })
  }

  fn call_function(&self, name: &str, args: &[serde_json::Value]) -> Result<serde_json::Value, String> {
    let args_json = serde_json::to_string(args).map_err(|e| format!("serialize args: {e}"))?;
    let script = format!(
      r#"import json, sys
try:
    result = {name}(*json.loads('{args_json}'))
    print(json.dumps(result))
except Exception as e:
    sys.stderr.write(str(e))
    sys.exit(1)
"#,
      name = name,
      args_json = args_json.replace('\'', "\\'"),
    );
    let (stdout, stderr, exit_code) = self.run_python(&["-c", &script], None, self.config.timeout_secs)?;
    if exit_code != 0 {
      return Err(format!("Python call_function '{name}' failed: {stderr}"));
    }
    serde_json::from_str(stdout.trim()).map_err(|e| format!("parse result: {e}"))
  }

  fn evaluate(&self, expr: &str) -> Result<serde_json::Value, String> {
    let script = format!("import json; print(json.dumps({expr}))");
    let (stdout, stderr, exit_code) = self.run_python(&["-c", &script], None, self.config.timeout_secs)?;
    if exit_code != 0 {
      return Err(format!("Python eval error: {stderr}"));
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
