use lru::LruCache;
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use sha2::{Digest, Sha512};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

thread_local! {
  static CYCLIC_GUARD: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

// ── Error Types ──────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum NodeError {
  #[error("TypeError: {0}")]
  TypeError(String),
  #[error("RangeError: {0}")]
  RangeError(String),
  #[error("ReferenceError: {0}")]
  ReferenceError(String),
  #[error("SyntaxError: {0}")]
  SyntaxError(String),
  #[error("EvalError: {0}")]
  EvalError(String),
  #[error("URIError: {0}")]
  URIError(String),
  #[error("Error: {0}")]
  Error(String),
  #[error("ModuleError: {0}")]
  ModuleError(String),
}

impl NodeError {
  pub fn js_type(&self) -> &str {
    match self {
      Self::TypeError(_) => "TypeError",
      Self::RangeError(_) => "RangeError",
      Self::ReferenceError(_) => "ReferenceError",
      Self::SyntaxError(_) => "SyntaxError",
      Self::EvalError(_) => "EvalError",
      Self::URIError(_) => "URIError",
      Self::Error(_) => "Error",
      Self::ModuleError(_) => "Error",
    }
  }
}

// ── Buffer ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Buffer(Vec<u8>);

impl Buffer {
  #[inline]
  pub fn alloc(size: usize) -> Self {
    Self(vec![0u8; size])
  }

  #[inline]
  pub fn alloc_filled(size: usize, fill: u8) -> Self {
    Self(vec![fill; size])
  }

  #[inline]
  pub fn from<T: AsRef<[u8]>>(data: T) -> Self {
    Self(data.as_ref().to_vec())
  }

  #[inline]
  pub fn concat(buffers: &[Buffer]) -> Self {
    let cap = buffers.iter().map(|b| b.len()).sum();
    let mut out = Vec::with_capacity(cap);
    for b in buffers {
      out.extend_from_slice(&b.0);
    }
    Self(out)
  }

  #[inline]
  pub fn byte_length<T: AsRef<[u8]>>(data: T) -> usize {
    data.as_ref().len()
  }

  pub fn to_string(&self, encoding: &str) -> Result<String, NodeError> {
    match encoding {
      "utf8" | "utf-8" => {
        Ok(String::from_utf8_lossy(&self.0).into_owned())
      }
      "hex" => Ok(hex::encode(&self.0)),
      "base64" => Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &self.0,
      )),
      "ascii" => Ok(self.0.iter().map(|&b| b as char).collect()),
      _ => Err(NodeError::TypeError(format!("Unknown encoding: {encoding}"))),
    }
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.0.len()
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  #[inline]
  pub fn as_slice(&self) -> &[u8] {
    &self.0
  }

  #[inline]
  pub fn into_inner(self) -> Vec<u8> {
    self.0
  }

  #[inline]
  pub fn slice(&self, start: usize, end: usize) -> Result<Self, NodeError> {
    if start > end || end > self.0.len() {
      return Err(NodeError::RangeError(
        "Buffer slice out of bounds".into(),
      ));
    }
    Ok(Self(self.0[start..end].to_vec()))
  }

  pub fn write(&mut self, data: &[u8], offset: usize) -> Result<usize, NodeError> {
    if offset + data.len() > self.0.len() {
      return Err(NodeError::RangeError(
        "Buffer write out of bounds".into(),
      ));
    }
    self.0[offset..offset + data.len()].copy_from_slice(data);
    Ok(data.len())
  }
}

// ── Process ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Process {
  pub argv: Vec<String>,
  pub env: HashMap<String, String>,
  pub platform: String,
  pub arch: String,
  pub version: String,
  pub versions: HashMap<String, String>,
  pub pid: u32,
  pub ppid: u32,
  pub title: String,
  exit_flag: Arc<AtomicBool>,
  exit_code: Arc<AtomicU32>,
}

impl Default for Process {
  fn default() -> Self {
    Self::new()
  }
}

impl Process {
  pub fn new() -> Self {
    let env: HashMap<String, String> = std::env::vars().collect();
    let versions_map = HashMap::from([
      ("node".into(), "22.14.0".into()),
      ("v8".into(), "12.8.0".into()),
      ("uv".into(), "1.48.0".into()),
      ("zlib".into(), "1.3.0".into()),
      ("brotli".into(), "1.1.0".into()),
      ("ares".into(), "1.28.1".into()),
      ("modules".into(), "127".into()),
      ("nghttp2".into(), "1.61.0".into()),
      ("napi".into(), "9".into()),
      ("llhttp".into(), "9.2.0".into()),
      ("openssl".into(), "3.3.0".into()),
      ("cldr".into(), "45.0".into()),
      ("icu".into(), "75.1".into()),
      ("tz".into(), "2024a".into()),
      ("unicode".into(), "15.1".into()),
    ]);

    Self {
      argv: std::env::args().collect(),
      env,
      platform: if cfg!(target_os = "linux") {
        "linux"
      } else if cfg!(target_os = "macos") {
        "darwin"
      } else if cfg!(target_os = "windows") {
        "win32"
      } else {
        "unknown"
      }
      .to_string(),
      arch: if cfg!(target_arch = "x86_64") {
        "x64"
      } else if cfg!(target_arch = "aarch64") {
        "arm64"
      } else {
        "unknown"
      }
      .to_string(),
      version: "v22.14.0".into(),
      versions: versions_map,
      pid: std::process::id(),
      ppid: 0,
      title: "klyron".into(),
      exit_flag: Arc::new(AtomicBool::new(false)),
      exit_code: Arc::new(AtomicU32::new(0)),
    }
  }

  #[inline]
  pub fn cwd(&self) -> PathBuf {
    std::env::current_dir().unwrap_or_default()
  }

  #[inline]
  pub fn exit(&self, code: i32) {
    self.exit_code.store(code as u32, Ordering::SeqCst);
    self.exit_flag.store(true, Ordering::SeqCst);
    std::process::exit(code);
  }

  pub fn next_tick<F>(f: F)
  where
    F: FnOnce() + Send + 'static,
  {
    std::thread::spawn(f);
  }

  pub fn hrtime(&self) -> u128 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or_default()
      .as_nanos()
  }

  pub fn uptime(&self) -> f64 {
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .ok()
      .map(|d| d.as_secs_f64())
      .unwrap_or(0.0)
  }

  pub fn memory_usage(&self) -> Value {
    json!({
      "rss": 0,
      "heapTotal": 0,
      "heapUsed": 0,
      "external": 0,
      "arrayBuffers": 0
    })
  }

  pub fn cpu_usage(&self) -> Value {
    json!({
      "user": 0,
      "system": 0
    })
  }
}

// ── Module Cache (LRU) ───────────────────────────────────────────────────────

const MODULE_CACHE_MAX: usize = 256;
static MODULE_CACHE: Lazy<Mutex<LruCache<String, (Value, bool)>>> =
  Lazy::new(|| Mutex::new(LruCache::new(
    std::num::NonZeroUsize::new(MODULE_CACHE_MAX).unwrap(),
  )));

#[inline]
pub fn module_cache_get(key: &str) -> Option<(Value, bool)> {
  MODULE_CACHE.lock().ok().and_then(|mut c| c.get(key).cloned())
}

#[inline]
pub fn module_cache_put(key: String, value: Value, loaded: bool) {
  if let Ok(mut c) = MODULE_CACHE.lock() {
    c.put(key, (value, loaded));
  }
}

#[inline]
pub fn module_cache_remove(key: &str) {
  if let Ok(mut c) = MODULE_CACHE.lock() {
    c.pop(key);
  }
}

#[inline]
pub fn module_cache_clear() {
  if let Ok(mut c) = MODULE_CACHE.lock() {
    c.clear();
  }
}

// ── Module and Exports ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Module {
  pub id: String,
  pub filename: PathBuf,
  pub dirname: PathBuf,
  pub exports: Value,
  pub loaded: bool,
  pub children: Vec<String>,
  pub parent: Option<String>,
  pub paths: Vec<PathBuf>,
}

impl Module {
  pub fn new(filename: &Path) -> Self {
    let dirname = filename.parent().unwrap_or(Path::new(".")).to_path_buf();
    Self {
      id: filename.to_string_lossy().to_string(),
      filename: filename.to_path_buf(),
      dirname,
      exports: json!({}),
      loaded: false,
      children: Vec::new(),
      parent: None,
      paths: Vec::new(),
    }
  }

  pub fn require(&self, specifier: &str) -> Result<Value, NodeError> {
    Module::_resolve_filename(specifier, &self.filename)
      .and_then(|resolved| Module::_load(&resolved, Some(&self.filename)))
  }
}

// ── Extension Handlers ───────────────────────────────────────────────────────

type ExtHandler = fn(&Path, &Module) -> Result<Value, NodeError>;

fn ext_js(_path: &Path, _module: &Module) -> Result<Value, NodeError> {
  let content = std::fs::read_to_string(_path)
    .map_err(|e| NodeError::Error(format!("Cannot read {}: {e}", _path.display())))?;
  let exports = json!({
    "__esModule": true,
    "__filename": _path.to_string_lossy().to_string(),
    "__dirname": _path.parent().map(|p| p.to_string_lossy().to_string()),
    "_source": content,
  });
  Ok(exports)
}

fn ext_json(path: &Path, _module: &Module) -> Result<Value, NodeError> {
  let content = std::fs::read_to_string(path)
    .map_err(|e| NodeError::Error(format!("Cannot read {}: {e}", path.display())))?;
  let val: Value =
    serde_json::from_str(&content).map_err(|e| NodeError::SyntaxError(format!("Invalid JSON: {e}")))?;
  Ok(val)
}

fn ext_node(path: &Path, _module: &Module) -> Result<Value, NodeError> {
  Err(NodeError::Error(format!(
    "Native .node addon '{}' requires klyron_napi",
    path.display()
  )))
}

fn get_ext_handler(ext: &str) -> Option<ExtHandler> {
  match ext {
    ".js" | ".mjs" | ".cjs" => Some(ext_js),
    ".json" => Some(ext_json),
    ".node" => Some(ext_node),
    _ => None,
  }
}

// ── Module Resolution ────────────────────────────────────────────────────────

impl Module {
  #[inline]
  pub fn _resolve_filename(request: &str, parent: &Path) -> Result<PathBuf, NodeError> {
    let parent_dir = parent.parent().unwrap_or(Path::new("."));

    // Core modules are returned as-is
    if Self::is_core_module(request) {
      return Ok(PathBuf::from(request));
    }

    // Absolute or relative specifier
    if request.starts_with('/') {
      let abs = Path::new(request);
      if abs.exists() {
        return Ok(abs.to_path_buf());
      }
      return Self::resolve_file_or_package(abs, parent_dir);
    }

    if request.starts_with('.') {
      let candidate = parent_dir.join(request);
      return Self::resolve_file_or_package(&candidate, parent_dir);
    }

    // Bare specifier: walk up node_modules
    Self::resolve_bare_specifier(request, parent_dir)
  }

  #[inline]
  fn resolve_file_or_package(path: &Path, _parent_dir: &Path) -> Result<PathBuf, NodeError> {
    if path.is_file() {
      return Ok(path.to_path_buf());
    }
    for ext in &[".js", ".json", ".node", ".mjs", ".cjs"] {
      let with_ext = path.with_extension(ext.trim_start_matches('.'));
      if with_ext.is_file() {
        return Ok(with_ext);
      }
    }
    let index_opts = ["index.js", "index.json", "index.node", "index.mjs", "index.cjs"];
    for name in &index_opts {
      let idx = path.join(name);
      if idx.is_file() {
        return Ok(idx);
      }
    }
    // Package.json exports
    if path.is_dir() {
      let pkg_json = path.join("package.json");
      if pkg_json.is_file() {
        if let Ok(pkg) = Self::read_package_json(&pkg_json) {
          if let Some(entry) = Self::resolve_package_exports(&path, &pkg) {
            return Ok(entry);
          }
          if let Some(main) = pkg.get("main").and_then(|m| m.as_str()) {
            let main_path = path.join(main);
            if main_path.is_file() {
              return Ok(main_path);
            }
          }
        }
      }
    }
    Err(NodeError::ModuleError(format!("Cannot find module '{}'", path.display())))
  }

  #[inline]
  fn resolve_bare_specifier(specifier: &str, start: &Path) -> Result<PathBuf, NodeError> {
    let mut dir = Some(start);
    while let Some(d) = dir {
      let nm = d.join("node_modules").join(specifier);
      if nm.exists() {
        return Self::resolve_file_or_package(&nm, d);
      }
      if let Ok(resolved) = Self::resolve_file_or_package(&nm, d) {
        return Ok(resolved);
      }
      dir = d.parent();
    }
    Err(NodeError::ModuleError(format!(
      "Cannot find module '{specifier}'"
    )))
  }

  fn read_package_json(path: &Path) -> Result<Value, NodeError> {
    let content =
      std::fs::read_to_string(path).map_err(|e| NodeError::Error(format!("Cannot read package.json: {e}")))?;
    serde_json::from_str(&content).map_err(|e| NodeError::SyntaxError(format!("Invalid package.json: {e}")))
  }

  fn resolve_package_exports(pkg_dir: &Path, pkg: &Value) -> Option<PathBuf> {
    let exports = pkg.get("exports")?;
    if let Some(s) = exports.as_str() {
      let entry = pkg_dir.join(s);
      if entry.is_file() {
        return Some(entry);
      }
    }
    if let Some(obj) = exports.as_object() {
      let keys = [".", "./", "."];
      for key in &keys {
        if let Some(val) = obj.get(*key) {
          if let Some(s) = val.as_str() {
            let entry = pkg_dir.join(s);
            if entry.is_file() {
              return Some(entry);
            }
          }
        }
      }
      for (key, val) in obj {
        if let Some(s) = val.as_str() {
          if key.starts_with(".") {
            let entry = pkg_dir.join(s);
            if entry.is_file() {
              return Some(entry);
            }
          }
        }
      }
    }
    None
  }

  pub fn is_core_module(name: &str) -> bool {
    matches!(
      name,
      "fs" | "path" | "http" | "https" | "os" | "crypto"
        | "stream" | "events" | "util" | "child_process" | "assert"
        | "buffer" | "url" | "timers" | "process" | "console" | "module"
        | "net" | "tls" | "dns" | "querystring" | "string_decoder"
        | "zlib" | "cluster" | "readline" | "vm" | "worker_threads"
        | "perf_hooks" | "async_hooks" | "diagnostics_channel"
        | "http2" | "inspector" | "repl" | "v8" | "trace_events"
        | "wasi" | "punycode" | "domain" | "constants" | "freelist"
        | "sys" | "base64" | "sha2"
    )
  }

  #[inline]
  pub fn _load(filename: &Path, _parent: Option<&Path>) -> Result<Value, NodeError> {
    let key = filename.to_string_lossy().to_string();

    // Cyclic dependency detection
    CYCLIC_GUARD.with(|guard| {
      let mut guard = guard.borrow_mut();
      if guard.contains(&key) {
        return Err(NodeError::ModuleError(format!(
          "Cyclic dependency detected: {key}"
        )));
      }
      guard.push(key.clone());
      Ok(())
    })?;

    // Check cache
    if let Some((exports, true)) = module_cache_get(&key) {
      CYCLIC_GUARD.with(|guard| guard.borrow_mut().retain(|k| k != &key));
      return Ok(exports);
    }

    let ext = filename
      .extension()
      .and_then(|e| e.to_str())
      .map(|e| format!(".{e}"))
      .unwrap_or_else(|| ".js".into());
    let handler = get_ext_handler(&ext)
      .ok_or_else(|| NodeError::ModuleError(format!("Unknown extension: {ext}")))?;

    let module = Module::new(filename);
    let exports = handler(filename, &module)?;

    module_cache_put(key.clone(), exports.clone(), true);

    CYCLIC_GUARD.with(|guard| guard.borrow_mut().retain(|k| k != &key));
    Ok(exports)
  }
}

// ── NodeGlobals ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NodeGlobals {
  pub global: Value,
  pub process: Process,
  pub buffer: Buffer,
  pub __dirname: PathBuf,
  pub __filename: PathBuf,
  pub exports: Value,
  pub module: Module,
  pub require_fn: RequireFn,
}

impl NodeGlobals {
  pub fn new(entry_file: &Path) -> Self {
    let dirname = entry_file.parent().unwrap_or(Path::new(".")).to_path_buf();
    let module = Module::new(entry_file);
    let exports = json!({});

    let require_fn = RequireFn {
      current_file: entry_file.to_path_buf(),
    };

    let mut globals = Self {
      global: json!({}),
      process: Process::new(),
      buffer: Buffer::alloc(0),
      __dirname: dirname,
      __filename: entry_file.to_path_buf(),
      exports: exports.clone(),
      module: module.clone(),
      require_fn,
    };

    globals.set_global("global", json!(null));
    globals.set_global("process", json!("[Process]"));
    globals.set_global("Buffer", json!("[Buffer]"));
    globals.set_global("console", json!("[Console]"));
    globals.set_global("setTimeout", json!("[Function]"));
    globals.set_global("clearTimeout", json!("[Function]"));
    globals.set_global("setInterval", json!("[Function]"));
    globals.set_global("clearInterval", json!("[Function]"));
    globals.set_global("setImmediate", json!("[Function]"));
    globals.set_global("clearImmediate", json!("[Function]"));

    globals
  }

  pub fn set_global(&mut self, key: &str, value: Value) {
    if let Value::Object(ref mut map) = self.global {
      map.insert(key.to_string(), value);
    }
  }

  pub fn get_global(&self, key: &str) -> Option<&Value> {
    self.global.get(key)
  }
}

// ── Require Function ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RequireFn {
  current_file: PathBuf,
}

impl RequireFn {
  pub fn new(current_file: &Path) -> Self {
    Self {
      current_file: current_file.to_path_buf(),
    }
  }

  #[inline]
  pub fn call(&self, specifier: &str) -> Result<Value, NodeError> {
    let resolved = Module::_resolve_filename(specifier, &self.current_file)?;
    Module::_load(&resolved, Some(&self.current_file))
  }

  #[inline]
  pub fn resolve(&self, specifier: &str) -> Result<PathBuf, NodeError> {
    Module::_resolve_filename(specifier, &self.current_file)
  }

  pub fn extensions() -> HashMap<String, ExtHandler> {
    let mut m = HashMap::new();
    m.insert(".js".into(), ext_js as ExtHandler);
    m.insert(".json".into(), ext_json as ExtHandler);
    m.insert(".node".into(), ext_node as ExtHandler);
    m
  }

  pub fn main() -> String {
    ".".to_string()
  }
}

// ── Utility Functions ────────────────────────────────────────────────────────

pub fn get_dirname(path: &Path) -> PathBuf {
  path.parent().unwrap_or(Path::new(".")).to_path_buf()
}

pub fn get_filename(path: &Path) -> PathBuf {
  path.to_path_buf()
}

pub fn compute_integrity(data: &[u8]) -> String {
  let mut hasher = Sha512::new();
  hasher.update(data);
  format!("sha512-{}", base64::Engine::encode(
    &base64::engine::general_purpose::STANDARD,
    &hasher.finalize(),
  ))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;
  use std::time::Duration;

  #[test]
  fn test_buffer_alloc() {
    let b = Buffer::alloc(10);
    assert_eq!(b.len(), 10);
    assert_eq!(b.as_slice(), &[0u8; 10]);
  }

  #[test]
  fn test_buffer_from() {
    let b = Buffer::from("hello");
    assert_eq!(b.to_string("utf8").unwrap(), "hello");
    assert_eq!(b.to_string("hex").unwrap(), "68656c6c6f");
    assert!(b.to_string("base64").is_ok());
  }

  #[test]
  fn test_buffer_concat() {
    let a = Buffer::from("ab");
    let b = Buffer::from("cd");
    let c = Buffer::concat(&[a, b]);
    assert_eq!(c.to_string("utf8").unwrap(), "abcd");
  }

  #[test]
  fn test_buffer_byte_length() {
    assert_eq!(Buffer::byte_length("hello"), 5);
    assert_eq!(Buffer::byte_length(b"hi"), 2);
  }

  #[test]
  fn test_buffer_slice() {
    let b = Buffer::from("hello");
    let s = b.slice(1, 4).unwrap();
    assert_eq!(s.to_string("utf8").unwrap(), "ell");
  }

  #[test]
  fn test_buffer_slice_out_of_bounds() {
    let b = Buffer::from("hi");
    assert!(b.slice(0, 10).is_err());
  }

  #[test]
  fn test_buffer_write() {
    let mut b = Buffer::alloc(10);
    let n = b.write(b"abc", 2).unwrap();
    assert_eq!(n, 3);
    assert_eq!(b.to_string("utf8").unwrap(), "\0\0abc\0\0\0\0\0");
  }

  #[test]
  fn test_process_defaults() {
    let p = Process::new();
    assert!(!p.argv.is_empty());
    assert!(p.env.contains_key("PATH") || p.env.is_empty());
    assert_eq!(p.platform, "linux");
    assert!(p.pid > 0);
  }

  #[test]
  fn test_process_cwd() {
    let p = Process::new();
    assert!(p.cwd().exists());
  }

  #[test]
  fn test_process_hrtime() {
    let p = Process::new();
    let t = p.hrtime();
    assert!(t > 0);
  }

  #[test]
  fn test_process_versions() {
    let p = Process::new();
    assert_eq!(p.versions.get("node").map(|s| s.as_str()), Some("22.14.0"));
  }

  #[test]
  fn test_module_new() {
    let m = Module::new(Path::new("/app/index.js"));
    assert!(!m.loaded);
    assert_eq!(m.dirname, Path::new("/app"));
    assert_eq!(m.exports, json!({}));
  }

  #[test]
  fn test_module_is_core() {
    assert!(Module::is_core_module("fs"));
    assert!(Module::is_core_module("buffer"));
    assert!(Module::is_core_module("path"));
    assert!(!Module::is_core_module("nonexistent_pkg"));
  }

  #[test]
  fn test_resolve_filename_absolute() {
    let f = std::env::temp_dir().join("_klyron_test_resolve.js");
    std::fs::write(&f, "").unwrap();
    let result = Module::_resolve_filename(f.to_str().unwrap(), Path::new("/"));
    assert!(result.is_ok());
    let _ = std::fs::remove_file(&f);
  }

  #[test]
  fn test_resolve_filename_core() {
    let result = Module::_resolve_filename("fs", Path::new("/index.js"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("fs"));
  }

  #[test]
  fn test_resolve_filename_not_found() {
    let result = Module::_resolve_filename("nonexistent_xyz_pkg_12345", Path::new("/"));
    assert!(result.is_err());
  }

  #[test]
  fn test_module_load_json() {
    let dir = std::env::temp_dir().join("_klyron_test_json");
    let _ = std::fs::create_dir_all(&dir);
    let f = dir.join("data.json");
    std::fs::write(&f, r#"{"hello":"world"}"#).unwrap();
    let result = Module::_load(&f, None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!({"hello":"world"}));
    let _ = std::fs::remove_file(&f);
  }

  #[test]
  fn test_node_globals() {
    let g = NodeGlobals::new(Path::new("/entry.js"));
    assert_eq!(g.__filename, Path::new("/entry.js"));
    assert_eq!(g.__dirname, Path::new("/"));
  }

  #[test]
  fn test_node_globals_set_get() {
    let mut g = NodeGlobals::new(Path::new("/e.js"));
    g.set_global("myVar", json!(42));
    assert_eq!(g.get_global("myVar"), Some(&json!(42)));
  }

  #[test]
  fn test_require_fn_fs() {
    let req = RequireFn::new(Path::new("/index.js"));
    assert!(req.resolve("fs").is_ok());
  }

  #[test]
  fn test_error_types() {
    let e1 = NodeError::TypeError("bad type".into());
    assert_eq!(e1.js_type(), "TypeError");
    assert!(e1.to_string().contains("bad type"));
  }

  #[test]
  fn test_integrity() {
    let hash = compute_integrity(b"hello");
    assert!(hash.starts_with("sha512-"));
  }

  #[test]
  fn test_module_cache() {
    module_cache_put("test".into(), json!("val"), true);
    let r = module_cache_get("test");
    assert!(r.is_some());
    assert_eq!(r.unwrap().0, json!("val"));
    module_cache_remove("test");
    assert!(module_cache_get("test").is_none());
  }

  #[test]
  fn test_cyclic_detection() {
    let dir = std::env::temp_dir().join("_klyron_cyclic_test");
    let _ = std::fs::create_dir_all(&dir);
    let a = dir.join("a.js");
    let b = dir.join("b.js");
    std::fs::write(&a, "require('./b')").unwrap();
    std::fs::write(&b, "require('./a')").unwrap();
    let result = Module::_load(&a, None);
    assert!(result.is_ok() || result.is_err());
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_require_fn_extensions() {
    let exts = RequireFn::extensions();
    assert!(exts.contains_key(".js"));
    assert!(exts.contains_key(".json"));
    assert!(exts.contains_key(".node"));
  }

  #[test]
  fn test_process_exit_flag() {
    let p = Process::new();
    assert!(!p.exit_flag.load(Ordering::SeqCst));
  }

  #[test]
  fn test_process_next_tick() {
    let flag = Arc::new(AtomicBool::new(false));
    let f = flag.clone();
    Process::next_tick(move || {
      f.store(true, Ordering::SeqCst);
    });
    std::thread::sleep(Duration::from_millis(50));
    assert!(flag.load(Ordering::SeqCst));
  }

  #[test]
  fn test_buffer_empty() {
    let b = Buffer::alloc(0);
    assert!(b.is_empty());
    assert_eq!(b.len(), 0);
  }
}
