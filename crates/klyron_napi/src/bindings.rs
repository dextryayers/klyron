use libloading::{Library, Symbol};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use crate::types::{NapiError, NapiValue, NAPI_VERSION_CURRENT, NAPI_VERSION_MAX, NAPI_VERSION_MIN};

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

        let napi_ver = Self::detect_napi_version(path);
        Self::check_version_compatibility(napi_ver)?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let lib = unsafe {
            Library::new(path).map_err(|e| {
                NapiError::LoadError(format!("Failed to load '{}': {e}", path.display()))
            })?
        };

        let mut module = NapiModule {
            name,
            path: path.to_path_buf(),
            exports: HashMap::new(),
            napi_version: napi_ver,
            library: Some(lib),
        };

        module.scan_exports()?;

        Ok(module)
    }

    pub(crate) fn detect_napi_version(path: &Path) -> u32 {
        let name = path.to_string_lossy();
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
                self.exports
                    .insert(sym_name.to_string(), NapiValue::External(ptr as usize));
            }
        }

        self.exports
            .insert("__napi_loaded".into(), NapiValue::Bool(true));
        self.exports
            .insert("__napi_version".into(), NapiValue::Uint(self.napi_version));

        Ok(())
    }

    pub fn call_function(&self, name: &str, args: &[NapiValue]) -> Result<NapiValue, NapiError> {
        let lib = self
            .library
            .as_ref()
            .ok_or_else(|| NapiError::LoadError("Library not loaded".into()))?;

        let c_name =
            CString::new(name).map_err(|_| NapiError::SymbolNotFound(name.into()))?;
        let func: Symbol<unsafe extern "C" fn(*const NapiValue, usize) -> NapiValue> = unsafe {
            lib.get(c_name.as_bytes())
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
    pub fn new(name: &str, execute: AsyncWorkCallback, complete: AsyncWorkComplete) -> Self {
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
                if let Ok(val) = result {
                    complete(val)
                }
            })
            .map_err(|e| NapiError::AsyncWorkError(format!("Thread spawn failed: {e}")))?;

        Ok(())
    }
}

pub struct AsyncWorkPool {
    #[allow(dead_code)]
    pub(crate) workers: usize,
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

#[derive(Default)]
pub struct NapiLoader {
    modules: HashMap<String, NapiModule>,
    pub(crate) search_paths: Vec<PathBuf>,
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
        self.modules.insert(name.to_string(), module);
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
            let candidate = pkg_dir.join(&os_name);
            if candidate.exists() {
                return Ok(candidate);
            }
            let candidate_simple = pkg_dir.join(&simple_name);
            if candidate_simple.exists() {
                return Ok(candidate_simple);
            }
            let build = pkg_dir.join("build").join("Release").join(&simple_name);
            if build.exists() {
                return Ok(build);
            }
            let build_os = pkg_dir.join("build").join("Release").join(&os_name);
            if build_os.exists() {
                return Ok(build_os);
            }
            let top = search_path.join(&simple_name);
            if top.exists() {
                return Ok(top);
            }
            let top_os = search_path.join(&os_name);
            if top_os.exists() {
                return Ok(top_os);
            }
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
        path.extension()
            .and_then(|e| e.to_str())
            .map_or(false, |e| e == "node")
    }
}
