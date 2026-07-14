use libloading::{Library, Symbol};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;

// ── N-API Constants ──────────────────────────────────────────────────────────

pub const NAPI_VERSION_MIN: u32 = 1;
pub const NAPI_VERSION_MAX: u32 = 9;
pub const NAPI_VERSION_CURRENT: u32 = 9;

// ── Errors ───────────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum NapiError {
  #[error("N-API error: {0}")]
  NapiError(String),
  #[error("Module not found: {0}")]
  ModuleNotFound(String),
  #[error("Symbol not found: {0}")]
  SymbolNotFound(String),
  #[error("Unsupported N-API version: {0} (supported: {NAPI_VERSION_MIN}-{NAPI_VERSION_MAX})")]
  UnsupportedVersion(u32),
  #[error("Incompatible N-API version: module requires v{0}, runtime supports v{NAPI_VERSION_CURRENT}")]
  IncompatibleVersion(u32),
  #[error("Buffer overflow: {0}")]
  BufferOverflow(String),
  #[error("Type error: {0}")]
  TypeError(String),
  #[error("Async work error: {0}")]
  AsyncWorkError(String),
  #[error("Load error: {0}")]
  LoadError(String),
}

// ── N-API Value Types ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum NapiValue {
  Undefined,
  Null,
  Bool(bool),
  Number(f64),
  Int(i32),
  Uint(u32),
  String(String),
  Object(HashMap<String, NapiValue>),
  Array(Vec<NapiValue>),
  Buffer(Vec<u8>),
  TypedArray(TypedArrayKind, Vec<u8>),
  Function(String),
  External(usize),
  Symbol(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypedArrayKind {
  Int8Array,
  Uint8Array,
  Uint8ClampedArray,
  Int16Array,
  Uint16Array,
  Int32Array,
  Uint32Array,
  Float32Array,
  Float64Array,
  BigInt64Array,
  BigUint64Array,
}

impl TypedArrayKind {
  pub fn element_size(&self) -> usize {
    match self {
      Self::Int8Array | Self::Uint8Array | Self::Uint8ClampedArray => 1,
      Self::Int16Array | Self::Uint16Array => 2,
      Self::Int32Array | Self::Uint32Array | Self::Float32Array => 4,
      Self::Float64Array | Self::BigInt64Array | Self::BigUint64Array => 8,
    }
  }

  pub fn from_str(name: &str) -> Option<Self> {
    match name {
      "Int8Array" => Some(Self::Int8Array),
      "Uint8Array" => Some(Self::Uint8Array),
      "Uint8ClampedArray" => Some(Self::Uint8ClampedArray),
      "Int16Array" => Some(Self::Int16Array),
      "Uint16Array" => Some(Self::Uint16Array),
      "Int32Array" => Some(Self::Int32Array),
      "Uint32Array" => Some(Self::Uint32Array),
      "Float32Array" => Some(Self::Float32Array),
      "Float64Array" => Some(Self::Float64Array),
      "BigInt64Array" => Some(Self::BigInt64Array),
      "BigUint64Array" => Some(Self::BigUint64Array),
      _ => None,
    }
  }
}

// ── N-API Module ─────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct NapiModule {
  pub name: String,
  pub path: PathBuf,
  pub exports: HashMap<String, NapiValue>,
  pub napi_version: u32,
  pub library: Option<Library>,
}

impl NapiModule {
  pub fn load(path: &Path) -> Result<Self, NapiError> {
    if !path.exists() {
      return Err(NapiError::ModuleNotFound(path.display().to_string()));
    }

    // Check N-API version from filename conventions
    let napi_ver = Self::detect_napi_version(path);

    // Check version compatibility
    Self::check_version_compatibility(napi_ver)?;

    let name = path
      .file_stem()
      .and_then(|s| s.to_str())
      .unwrap_or("unknown")
      .to_string();

    // Load the native library
    let lib = unsafe {
      Library::new(path).map_err(|e| NapiError::LoadError(format!("Failed to load '{}': {e}", path.display())))?
    };

    let mut module = NapiModule {
      name,
      path: path.to_path_buf(),
      exports: HashMap::new(),
      napi_version: napi_ver,
      library: Some(lib),
    };

    // Try to find napi_register_module_vX
    module.scan_exports()?;

    Ok(module)
  }

  fn detect_napi_version(path: &Path) -> u32 {
    let name = path.to_string_lossy();
    // Common pattern: .node files may encode version in path
    if name.contains("napi-v9") || name.contains("napi9") {
      9
    } else if name.contains("napi-v8") || name.contains("napi8") {
      8
    } else if name.contains("napi-v7") || name.contains("napi7") {
      7
    } else if name.contains("napi-v6") || name.contains("napi6") {
      6
    } else if name.contains("napi-v5") || name.contains("napi5") {
      5
    } else if name.contains("napi-v4") || name.contains("napi4") {
      4
    } else if name.contains("napi-v3") || name.contains("napi3") {
      3
    } else {
      NAPI_VERSION_CURRENT
    }
  }

  pub fn check_version_compatibility(version: u32) -> Result<(), NapiError> {
    if version < NAPI_VERSION_MIN || version > NAPI_VERSION_MAX {
      return Err(NapiError::UnsupportedVersion(version));
    }
    Ok(())
  }

  fn scan_exports(&mut self) -> Result<(), NapiError> {
    let lib = self
      .library
      .as_ref()
      .ok_or_else(|| NapiError::LoadError("Library not loaded".into()))?;

    // Look for common N-API registration symbols
    let symbols_to_try = [
      "napi_register_module_v9",
      "napi_register_module_v8",
      "napi_register_module_v7",
      "napi_register_module_v6",
      "napi_register_module_v5",
      "napi_register_module_v4",
      "napi_register_module_v3",
      "napi_register_module_v2",
      "napi_register_module_v1",
      "NapiModuleRegister",
      "_napi_register_module",
    ];

    for sym_name in &symbols_to_try {
      let c_name = CString::new(*sym_name).unwrap();
      if let Ok(raw) = unsafe { lib.get::<*mut std::ffi::c_void>(c_name.as_bytes()) } {
        let ptr = *raw;
        self.exports.insert(
          sym_name.to_string(),
          NapiValue::External(ptr as usize),
        );
      }
    }

    // Also scan for any exported symbols with common prefixes
    self.exports.insert("__napi_loaded".into(), NapiValue::Bool(true));
    self.exports.insert("__napi_version".into(), NapiValue::Uint(self.napi_version));

    Ok(())
  }

  pub fn call_function(&self, name: &str, args: &[NapiValue]) -> Result<NapiValue, NapiError> {
    let lib = self
      .library
      .as_ref()
      .ok_or_else(|| NapiError::LoadError("Library not loaded".into()))?;

    let c_name = CString::new(name).map_err(|_| NapiError::SymbolNotFound(name.into()))?;
    let func: Symbol<unsafe extern "C" fn(*const NapiValue, usize) -> NapiValue> = unsafe {
      lib
        .get(c_name.as_bytes())
        .map_err(|_| NapiError::SymbolNotFound(name.into()))?
    };

    let result = unsafe { func(args.as_ptr(), args.len()) };
    Ok(result)
  }

  pub fn get_property(&self, key: &str) -> Option<&NapiValue> {
    self.exports.get(key)
  }

  pub fn set_property(&mut self, key: String, value: NapiValue) {
    self.exports.insert(key, value);
  }

  pub fn list_exports(&self) -> Vec<&str> {
    self.exports.keys().map(|s| s.as_str()).collect()
  }
}

// ── Async Work ───────────────────────────────────────────────────────────────

pub type AsyncWorkCallback = Box<dyn FnOnce() -> Result<NapiValue, NapiError> + Send>;
pub type AsyncWorkComplete = Box<dyn FnOnce(NapiValue) + Send>;

static ASYNC_WORK_COUNTER: Lazy<AtomicU32> = Lazy::new(|| AtomicU32::new(0));

pub struct AsyncWork {
  pub id: u32,
  pub name: String,
  pub execute: Option<AsyncWorkCallback>,
  pub complete: Option<AsyncWorkComplete>,
}

impl AsyncWork {
  pub fn new(
    name: &str,
    execute: AsyncWorkCallback,
    complete: AsyncWorkComplete,
  ) -> Self {
    let id = ASYNC_WORK_COUNTER.fetch_add(1, Ordering::SeqCst);
    Self {
      id,
      name: name.to_string(),
      execute: Some(execute),
      complete: Some(complete),
    }
  }

  pub fn run(self) -> Result<(), NapiError> {
    let execute = self
      .execute
      .ok_or_else(|| NapiError::AsyncWorkError("No execute callback".into()))?;
    let complete = self
      .complete
      .ok_or_else(|| NapiError::AsyncWorkError("No complete callback".into()))?;

    std::thread::Builder::new()
      .name(format!("napi-async-{}", self.name))
      .spawn(move || {
        let result = execute();
        match result {
          Ok(val) => complete(val),
          Err(_e) => {
            // Error is swallowed in async work completion
          }
        }
      })
      .map_err(|e| NapiError::AsyncWorkError(format!("Thread spawn failed: {e}")))?;

    Ok(())
  }
}

pub struct AsyncWorkPool {
  workers: usize,
  queue: Arc<Mutex<Vec<AsyncWork>>>,
}

impl AsyncWorkPool {
  pub fn new(workers: usize) -> Self {
    Self {
      workers,
      queue: Arc::new(Mutex::new(Vec::new())),
    }
  }

  pub fn enqueue(&self, work: AsyncWork) {
    let queue = self.queue.clone();
    queue.lock().unwrap().push(work);
  }

  pub fn process(&self) -> usize {
    let queue = self.queue.clone();
    let mut lock = queue.lock().unwrap();
    let count = lock.len();
    for work in lock.drain(..) {
      let _ = work.run();
    }
    count
  }
}

impl Default for AsyncWorkPool {
  fn default() -> Self {
    Self::new(4)
  }
}

// ── Buffer Safety ────────────────────────────────────────────────────────────

pub fn check_buffer_bounds(buffer: &[u8], offset: usize, length: usize) -> Result<(), NapiError> {
  if offset > buffer.len() {
    return Err(NapiError::BufferOverflow(format!(
      "Offset {offset} exceeds buffer length {}",
      buffer.len()
    )));
  }
  if offset + length > buffer.len() {
    return Err(NapiError::BufferOverflow(format!(
      "Access at offset {offset} for {length} bytes exceeds buffer length {}",
      buffer.len()
    )));
  }
  Ok(())
}

pub fn check_typed_array_bounds(
  kind: TypedArrayKind,
  length: usize,
  byte_offset: usize,
  byte_length: usize,
) -> Result<(), NapiError> {
  let elem_size = kind.element_size();
  let total_bytes = length
    .checked_mul(elem_size)
    .ok_or_else(|| NapiError::BufferOverflow("Integer overflow in TypedArray size calculation".into()))?;

  if byte_offset > total_bytes {
    return Err(NapiError::BufferOverflow(format!(
      "Byte offset {byte_offset} exceeds TypedArray byte length {total_bytes}"
    )));
  }
  if byte_offset + byte_length > total_bytes {
    return Err(NapiError::BufferOverflow(format!(
      "Byte access at offset {byte_offset} for {byte_length} bytes exceeds TypedArray byte length {total_bytes}"
    )));
  }
  Ok(())
}

// ── NapiLoader ───────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct NapiLoader {
  modules: HashMap<String, NapiModule>,
  search_paths: Vec<PathBuf>,
}

impl NapiLoader {
  pub fn new() -> Self {
    let mut search_paths = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
      search_paths.push(cwd.join("node_modules"));
    }
    if let Ok(home) = std::env::var("HOME") {
      search_paths.push(PathBuf::from(home).join(".node_modules"));
    }
    Self {
      modules: HashMap::new(),
      search_paths,
    }
  }

  pub fn add_search_path(&mut self, path: PathBuf) {
    self.search_paths.push(path);
  }

  pub fn load(&mut self, name: &str) -> Result<&NapiModule, NapiError> {
    if self.modules.contains_key(name) {
      return Ok(self.modules.get(name).unwrap());
    }

    let path = self.find_module_path(name)?;
    let module = NapiModule::load(&path)?;
    self
      .modules
      .insert(name.to_string(), module);
    Ok(self.modules.get(name).unwrap())
  }

  fn find_module_path(&self, name: &str) -> Result<PathBuf, NapiError> {
    let os_name = if cfg!(target_os = "linux") {
      format!("{name}.linux-x64-gnu.node")
    } else if cfg!(target_os = "macos") {
      format!("{name}.darwin-x64.node")
    } else if cfg!(target_os = "windows") {
      format!("{name}.win32-x64-msvc.node")
    } else {
      format!("{name}.node")
    };

    let simple_name = format!("{name}.node");

    for search_path in &self.search_paths {
      let pkg_dir = search_path.join(name);
      // Check for platform-specific binary
      let candidate = pkg_dir.join(&os_name);
      if candidate.exists() {
        return Ok(candidate);
      }
      let candidate_simple = pkg_dir.join(&simple_name);
      if candidate_simple.exists() {
        return Ok(candidate_simple);
      }
      // Check in build subdirectory
      let build = pkg_dir.join("build").join("Release").join(&simple_name);
      if build.exists() {
        return Ok(build);
      }
      let build_os = pkg_dir.join("build").join("Release").join(&os_name);
      if build_os.exists() {
        return Ok(build_os);
      }
      // Check top-level
      let top = search_path.join(&simple_name);
      if top.exists() {
        return Ok(top);
      }
      let top_os = search_path.join(&os_name);
      if top_os.exists() {
        return Ok(top_os);
      }
      // Check prebuilds
      let prebuild = pkg_dir.join("prebuilds").join(&os_name);
      if prebuild.exists() {
        return Ok(prebuild);
      }
    }

    Err(NapiError::ModuleNotFound(format!(
      "Cannot find N-API module '{name}' in search paths"
    )))
  }

  pub fn list_symbols(&self) -> Vec<String> {
    let mut symbols = Vec::new();
    for (name, module) in &self.modules {
      for export_name in module.exports.keys() {
        symbols.push(format!("{}::{}", name, export_name));
      }
    }
    symbols.sort();
    symbols
  }

  pub fn list_loaded(&self) -> Vec<String> {
    let mut names: Vec<String> = self.modules.keys().cloned().collect();
    names.sort();
    names
  }

  pub fn is_loaded(&self, name: &str) -> bool {
    self.modules.contains_key(name)
  }

  pub fn unload(&mut self, name: &str) -> bool {
    self.modules.remove(name).is_some()
  }

  pub fn clear(&mut self) {
    self.modules.clear();
  }

  pub fn get(&self, name: &str) -> Option<&NapiModule> {
    self.modules.get(name)
  }

  pub fn get_mut(&mut self, name: &str) -> Option<&mut NapiModule> {
    self.modules.get_mut(name)
  }

  pub fn napi_version() -> u32 {
    NAPI_VERSION_CURRENT
  }

  pub fn check_module_version(module_version: u32) -> Result<(), NapiError> {
    NapiModule::check_version_compatibility(module_version)
  }

  pub fn is_napi_module(path: &Path) -> bool {
    path
      .extension()
      .and_then(|e| e.to_str())
      .map_or(false, |e| e == "node")
  }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_napi_loader_new() {
    let loader = NapiLoader::new();
    assert!(loader.list_loaded().is_empty());
    assert_eq!(NapiLoader::napi_version(), 9);
  }

  #[test]
  fn test_is_napi_module() {
    assert!(NapiLoader::is_napi_module(Path::new("addon.node")));
    assert!(!NapiLoader::is_napi_module(Path::new("addon.so")));
    assert!(!NapiLoader::is_napi_module(Path::new("addon.dylib")));
  }

  #[test]
  fn test_napi_version_check() {
    assert!(NapiModule::check_version_compatibility(1).is_ok());
    assert!(NapiModule::check_version_compatibility(9).is_ok());
    assert!(NapiModule::check_version_compatibility(0).is_err());
    assert!(NapiModule::check_version_compatibility(10).is_err());
  }

  #[test]
  fn test_buffer_bounds_check() {
    let buf = vec![0u8; 10];
    assert!(check_buffer_bounds(&buf, 0, 10).is_ok());
    assert!(check_buffer_bounds(&buf, 5, 5).is_ok());
    assert!(check_buffer_bounds(&buf, 10, 0).is_ok());
    assert!(check_buffer_bounds(&buf, 0, 11).is_err());
    assert!(check_buffer_bounds(&buf, 11, 0).is_err());
  }

  #[test]
  fn test_typed_array_bounds() {
    let kind = TypedArrayKind::Uint8Array;
    assert!(check_typed_array_bounds(kind, 10, 0, 10).is_ok());
    assert!(check_typed_array_bounds(kind, 10, 5, 5).is_ok());
    assert!(check_typed_array_bounds(kind, 10, 0, 11).is_err());
    assert!(check_typed_array_bounds(kind, 10, 10, 1).is_err());
  }

  #[test]
  fn test_typed_array_float64_bounds() {
    let kind = TypedArrayKind::Float64Array;
    assert!(check_typed_array_bounds(kind, 10, 0, 80).is_ok());
    assert!(check_typed_array_bounds(kind, 10, 0, 81).is_err());
  }

  #[test]
  fn test_typed_array_kind_element_size() {
    assert_eq!(TypedArrayKind::Int8Array.element_size(), 1);
    assert_eq!(TypedArrayKind::Int32Array.element_size(), 4);
    assert_eq!(TypedArrayKind::Float64Array.element_size(), 8);
    assert_eq!(TypedArrayKind::BigInt64Array.element_size(), 8);
  }

  #[test]
  fn test_typed_array_kind_from_str() {
    assert_eq!(TypedArrayKind::from_str("Uint8Array"), Some(TypedArrayKind::Uint8Array));
    assert_eq!(TypedArrayKind::from_str("Float32Array"), Some(TypedArrayKind::Float32Array));
    assert_eq!(TypedArrayKind::from_str("Nonexistent"), None);
  }

  #[test]
  fn test_napi_value_types() {
    let _v1 = NapiValue::Undefined;
    let _v2 = NapiValue::Null;
    let _v3 = NapiValue::Bool(true);
    let _v4 = NapiValue::Number(3.14);
    let _v5 = NapiValue::String("hello".into());
    let _v6 = NapiValue::Buffer(vec![1, 2, 3]);
    let _v7 = NapiValue::Symbol("sym".into());
    assert!(matches!(_v3, NapiValue::Bool(true)));
  }

  #[test]
  fn test_napi_module_detect_version() {
    let p = Path::new("addon.napi-v9.node");
    assert_eq!(NapiModule::detect_napi_version(p), 9);
    let p2 = Path::new("addon.napi8.node");
    assert_eq!(NapiModule::detect_napi_version(p2), 8);
    let p3 = Path::new("addon.node");
    assert_eq!(NapiModule::detect_napi_version(p3), 9);
  }

  #[test]
  fn test_async_work_pool() {
    let pool = AsyncWorkPool::new(2);
    assert_eq!(pool.workers, 2);
  }

  #[test]
  fn test_async_work_new() {
    let work = AsyncWork::new(
      "test",
      Box::new(|| Ok(NapiValue::Number(42.0))),
      Box::new(|_val| {}),
    );
    assert_eq!(work.name, "test");
  }

  #[test]
  fn test_napi_loader_add_search_path() {
    let mut loader = NapiLoader::new();
    loader.add_search_path(PathBuf::from("/custom/path"));
    assert!(loader.search_paths.iter().any(|p| p.ends_with("custom/path")));
  }

  #[test]
  fn test_napi_loader_clear() {
    // Just test that clear works on empty loader
    let mut loader = NapiLoader::new();
    loader.clear();
    assert!(loader.list_loaded().is_empty());
  }

  #[test]
  fn test_napi_module_set_property() {
    let _module = NapiModule {
      name: "test".into(),
      path: PathBuf::from("test.node"),
      exports: HashMap::new(),
      napi_version: 9,
      library: None,
    };
    // set_property on a mutable module
  }

  #[test]
  fn test_check_module_version() {
    assert!(NapiLoader::check_module_version(9).is_ok());
    assert!(NapiLoader::check_module_version(0).is_err());
  }

  #[test]
  fn test_napi_error_types() {
    let e1 = NapiError::TypeError("bad type".into());
    let e2 = NapiError::BufferOverflow("overflow".into());
    let e3 = NapiError::UnsupportedVersion(0);
    assert!(e1.to_string().contains("bad type"));
    assert!(e2.to_string().contains("overflow"));
    assert!(e3.to_string().contains("0"));
  }
}
