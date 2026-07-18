use std::cell::RefCell;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use lru::LruCache;
use once_cell::sync::Lazy;
use serde_json::Value;
use sha2::{Digest, Sha512};
use std::sync::Mutex;

use crate::NodeError;

thread_local! {
    static CYCLIC_GUARD: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

const MODULE_CACHE_MAX: usize = 256;
static MODULE_CACHE: Lazy<Mutex<LruCache<String, (Value, bool)>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(MODULE_CACHE_MAX).unwrap())));

type ExtHandler = fn(&Path, &Module) -> Result<Value, NodeError>;

fn ext_js(path: &Path, _module: &Module) -> Result<Value, NodeError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| NodeError::Error(format!("Cannot read {}: {e}", path.display())))?;
    let exports = serde_json::json!({
        "__esModule": true,
        "__filename": path.to_string_lossy().to_string(),
        "__dirname": path.parent().map(|p| p.to_string_lossy().to_string()),
        "_source": content,
    });
    Ok(exports)
}

fn ext_json(path: &Path, _module: &Module) -> Result<Value, NodeError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| NodeError::Error(format!("Cannot read {}: {e}", path.display())))?;
    let val: Value = serde_json::from_str(&content)
        .map_err(|e| NodeError::SyntaxError(format!("Invalid JSON: {e}")))?;
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
            exports: serde_json::json!({}),
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

    pub fn _resolve_filename(request: &str, parent: &Path) -> Result<PathBuf, NodeError> {
        let parent_dir = parent.parent().unwrap_or(Path::new("."));

        if Self::is_core_module(request) {
            return Ok(PathBuf::from(request));
        }

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

        Self::resolve_bare_specifier(request, parent_dir)
    }

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
        let index_opts = [
            "index.js",
            "index.json",
            "index.node",
            "index.mjs",
            "index.cjs",
        ];
        for name in &index_opts {
            let idx = path.join(name);
            if idx.is_file() {
                return Ok(idx);
            }
        }
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
        Err(NodeError::ModuleError(format!(
            "Cannot find module '{}'",
            path.display()
        )))
    }

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
        let content = std::fs::read_to_string(path)
            .map_err(|e| NodeError::Error(format!("Cannot read package.json: {e}")))?;
        serde_json::from_str(&content)
            .map_err(|e| NodeError::SyntaxError(format!("Invalid package.json: {e}")))
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
                    if key.starts_with('.') {
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

    pub fn _load(filename: &Path, _parent: Option<&Path>) -> Result<Value, NodeError> {
        let key = filename.to_string_lossy().to_string();

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

pub fn module_cache_get(key: &str) -> Option<(Value, bool)> {
    MODULE_CACHE
        .lock()
        .ok()
        .and_then(|mut c| c.get(key).cloned())
}

pub fn module_cache_put(key: String, value: Value, loaded: bool) {
    if let Ok(mut c) = MODULE_CACHE.lock() {
        c.put(key, (value, loaded));
    }
}

pub fn module_cache_remove(key: &str) {
    if let Ok(mut c) = MODULE_CACHE.lock() {
        c.pop(key);
    }
}

pub fn module_cache_clear() {
    if let Ok(mut c) = MODULE_CACHE.lock() {
        c.clear();
    }
}

pub fn compute_integrity(data: &[u8]) -> String {
    let mut hasher = Sha512::new();
    hasher.update(data);
    format!(
        "sha512-{}",
        base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &hasher.finalize(),
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_new() {
        let m = Module::new(Path::new("/app/index.js"));
        assert!(!m.loaded);
        assert_eq!(m.dirname, Path::new("/app"));
        assert_eq!(m.exports, serde_json::json!({}));
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
        assert_eq!(result.unwrap(), serde_json::json!({"hello":"world"}));
        let _ = std::fs::remove_file(&f);
    }

    #[test]
    fn test_module_cache() {
        module_cache_put("test".into(), serde_json::json!("val"), true);
        let r = module_cache_get("test");
        assert!(r.is_some());
        assert_eq!(r.unwrap().0, serde_json::json!("val"));
        module_cache_remove("test");
        assert!(module_cache_get("test").is_none());
    }

    #[test]
    fn test_integrity() {
        let hash = compute_integrity(b"hello");
        assert!(hash.starts_with("sha512-"));
    }
}
