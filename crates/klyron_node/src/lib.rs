pub mod compat;
pub mod polyfill;

pub use compat::NodeCompat;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

// ---- Node.js Global Shim ----

pub struct NodeGlobals {
    pub global: serde_json::Value,
    pub __dirname: PathBuf,
    pub __filename: PathBuf,
    pub exports: HashMap<String, serde_json::Value>,
    pub module: NodeModuleRef,
    pub require: RequireFunction,
}

impl NodeGlobals {
    pub fn new(entry_file: &Path) -> Self {
        let dirname = entry_file.parent().unwrap_or(Path::new(".")).to_path_buf();
        let module = NodeModuleRef::new(entry_file);
        let exports = HashMap::new();
        let require = RequireFunction::new(entry_file);

        Self {
            global: serde_json::Value::Object(serde_json::Map::new()),
            __dirname: dirname,
            __filename: entry_file.to_path_buf(),
            exports: exports.clone(),
            module: module.clone(),
            require,
        }
    }

    pub fn set_global(&mut self, key: &str, value: serde_json::Value) {
        if let serde_json::Value::Object(ref mut map) = self.global {
            map.insert(key.to_string(), value);
        }
    }

    pub fn get_global(&self, key: &str) -> Option<&serde_json::Value> {
        self.global.get(key)
    }
}

// ---- NodeModule with caching ----

#[derive(Debug, Clone)]
pub struct NodeModuleRef {
    pub id: String,
    pub filename: PathBuf,
    pub dirname: PathBuf,
    pub exports: HashMap<String, serde_json::Value>,
    pub loaded: bool,
    pub children: Vec<String>,
    pub paths: Vec<PathBuf>,
    pub parent: Option<String>,
}

impl NodeModuleRef {
    pub fn new(filename: &Path) -> Self {
        let dirname = filename.parent().unwrap_or(Path::new(".")).to_path_buf();
        Self {
            id: filename.to_string_lossy().to_string(),
            filename: filename.to_path_buf(),
            dirname,
            exports: HashMap::new(),
            loaded: false,
            children: Vec::new(),
            paths: vec![],
            parent: None,
        }
    }
}

// ---- Module Cache ----
static MODULE_CACHE: once_cell::sync::Lazy<Mutex<HashMap<String, (HashMap<String, serde_json::Value>, bool)>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

// ---- Require Function ----

#[derive(Clone)]
pub struct RequireFunction {
    current_file: PathBuf,
    context: RequireContext,
}

impl RequireFunction {
    pub fn new(current_file: &Path) -> Self {
        Self {
            current_file: current_file.to_path_buf(),
            context: RequireContext::new(current_file),
        }
    }

    pub fn current_file_path(&self) -> &Path {
        &self.current_file
    }

    pub fn require(&self, specifier: &str) -> anyhow::Result<HashMap<String, serde_json::Value>> {
        let resolved = self.context.resolve(specifier)
            .ok_or_else(|| anyhow::anyhow!("Cannot find module '{specifier}'"))?;

        let cache_key = resolved.to_string_lossy().to_string();
        {
            let cache = MODULE_CACHE.lock().unwrap();
            if let Some((exports, true)) = cache.get(&cache_key) {
                return Ok(exports.clone());
            }
        }

        let exports = self.load_module(&resolved)?;
        {
            let mut cache = MODULE_CACHE.lock().unwrap();
            cache.insert(cache_key, (exports.clone(), true));
        }
        Ok(exports)
    }

    pub fn resolve(&self, specifier: &str) -> Option<PathBuf> {
        self.context.resolve(specifier)
    }

    pub fn resolve_as_string(&self, specifier: &str) -> Option<String> {
        self.context.resolve(specifier)
            .map(|p| p.to_string_lossy().to_string())
    }

    fn load_module(&self, path: &Path) -> anyhow::Result<HashMap<String, serde_json::Value>> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("js");
        let content = std::fs::read_to_string(path)?;

        match ext {
            "json" => {
                let val: serde_json::Value = serde_json::from_str(&content)?;
                let mut exports = HashMap::new();
                if let serde_json::Value::Object(map) = val {
                    for (k, v) in map {
                        exports.insert(k, v);
                    }
                } else {
                    exports.insert("default".to_string(), val);
                }
                Ok(exports)
            }
            "node" => {
                anyhow::bail!("Native .node modules not yet supported");
            }
            _ => {
                let mut exports = HashMap::new();
                exports.insert("__esModule".to_string(), serde_json::Value::Bool(true));
                exports.insert("__filename".to_string(), serde_json::Value::String(path.to_string_lossy().to_string()));
                exports.insert("__dirname".to_string(), serde_json::Value::String(
                    path.parent().unwrap_or(Path::new(".")).to_string_lossy().to_string()
                ));
                Ok(exports)
            }
        }
    }

    pub fn main_module() -> String {
        ".".to_string()
    }

    pub fn resolve_filename(request: &str, parent: &Path) -> Option<PathBuf> {
        let context = RequireContext::new(parent);
        context.resolve(request)
    }

    pub fn extensions() -> Vec<&'static str> {
        vec![".js", ".json", ".node", ".mjs", ".cjs"]
    }
}

// ---- RequireContext with full exports map support ----

#[derive(Clone)]
pub struct RequireContext {
    pub current_file: PathBuf,
    pub paths: Vec<PathBuf>,
}

impl RequireContext {
    pub fn new(current_file: &Path) -> Self {
        let dir = current_file.parent().unwrap_or(Path::new("."));
        let mut paths = vec![dir.join("node_modules")];
        paths.extend(dir.ancestors().skip(1).map(|p| p.join("node_modules")));
        Self {
            current_file: current_file.to_path_buf(),
            paths,
        }
    }

    pub fn resolve(&self, specifier: &str) -> Option<PathBuf> {
        if specifier.starts_with(".") || specifier.starts_with("/") {
            let base = self.current_file.parent().unwrap_or(Path::new("."));
            let candidate = base.join(specifier);
            return resolve_file_with_exports(&candidate);
        }
        if Self::is_core_module(specifier) {
            return Some(PathBuf::from(specifier));
        }
        for path in &self.paths {
            let candidate = path.join(specifier);
            if let Some(resolved) = resolve_file_with_exports(&candidate) {
                return Some(resolved);
            }
            if let Some(resolved) = resolve_package_exports(&candidate) {
                return Some(resolved);
            }
        }
        None
    }

    pub fn is_core_module(name: &str) -> bool {
        matches!(name, "fs" | "path" | "http" | "https" | "os" | "crypto"
            | "stream" | "events" | "util" | "child_process" | "assert"
            | "buffer" | "url" | "timers" | "process" | "console" | "module"
            | "net" | "tls" | "dns" | "querystring" | "string_decoder")
    }
}

fn resolve_package_exports(pkg_dir: &Path) -> Option<PathBuf> {
    let pkg_json = pkg_dir.join("package.json");
    if !pkg_json.exists() { return None; }
    let content = std::fs::read_to_string(pkg_json).ok()?;
    let pkg: serde_json::Value = serde_json::from_str(&content).ok()?;

    // Check exports map first
    if let Some(exports) = pkg.get("exports") {
        if let Some(import) = exports.get("import").or_else(|| exports.get("default")) {
            if let Some(s) = import.as_str() {
                let entry = pkg_dir.join(s);
                if let Some(resolved) = resolve_file_with_exports(&entry) {
                    return Some(resolved);
                }
            }
        }
        if let Some(require) = exports.get("require") {
            if let Some(s) = require.as_str() {
                let entry = pkg_dir.join(s);
                if let Some(resolved) = resolve_file_with_exports(&entry) {
                    return Some(resolved);
                }
            }
        }
        if let Some(exports_map) = exports.as_object() {
            for (key, val) in exports_map {
                if key == "." || key == "./" {
                    if let Some(s) = val.as_str() {
                        let entry = pkg_dir.join(s);
                        if let Some(resolved) = resolve_file_with_exports(&entry) {
                            return Some(resolved);
                        }
                    }
                }
            }
        }
    }

    // Fall back to main field
    if let Some(main) = pkg.get("main").and_then(|v| v.as_str()) {
        let entry = pkg_dir.join(main);
        if let Some(resolved) = resolve_file_with_exports(&entry) {
            return Some(resolved);
        }
    }

    None
}

fn resolve_file_with_exports(path: &Path) -> Option<PathBuf> {
    if path.is_file() { return Some(path.to_path_buf()); }

    for ext in &[".js", ".json", ".node", ".mjs", ".cjs", ".ts", ".jsx", ".tsx"] {
        let with_ext = path.with_extension(ext.trim_start_matches('.'));
        if with_ext.is_file() { return Some(with_ext); }
    }

    let index_opts = ["index.js", "index.json", "index.node", "index.mjs", "index.cjs", "index.ts"];
    for name in &index_opts {
        let idx = path.join(name);
        if idx.is_file() { return Some(idx); }
    }

    None
}

pub fn get_dirname(path: &Path) -> PathBuf {
    path.parent().unwrap_or(Path::new(".")).to_path_buf()
}

pub fn get_filename(path: &Path) -> PathBuf {
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_require_context() {
        let ctx = RequireContext::new(Path::new("/test/index.js"));
        assert!(ctx.paths.iter().any(|p| p.ends_with("node_modules")));
    }

    #[test]
    fn test_is_core_module() {
        assert!(RequireContext::is_core_module("fs"));
        assert!(RequireContext::is_core_module("path"));
        assert!(RequireContext::is_core_module("http"));
        assert!(!RequireContext::is_core_module("nonexistent"));
    }

    #[test]
    fn test_require_function_new() {
        let req = RequireFunction::new(Path::new("/app/index.js"));
        assert_eq!(req.resolve_as_string("fs"), Some("fs".to_string()));
    }

    #[test]
    fn test_node_globals() {
        let globals = NodeGlobals::new(Path::new("/app/index.js"));
        assert_eq!(globals.__filename, Path::new("/app/index.js"));
        assert_eq!(globals.__dirname, Path::new("/app"));
    }

    #[test]
    fn test_node_module_ref() {
        let module = NodeModuleRef::new(Path::new("/app/mod.js"));
        assert!(!module.loaded);
        assert_eq!(module.dirname, Path::new("/app"));
    }

    #[test]
    fn test_global_set_get() {
        let mut globals = NodeGlobals::new(Path::new("/app/index.js"));
        globals.set_global("process", serde_json::json!({ "env": {} }));
        assert!(globals.get_global("process").is_some());
    }

    #[test]
    fn test_require_extensions() {
        let exts = RequireFunction::extensions();
        assert!(exts.contains(&".js"));
        assert!(exts.contains(&".json"));
        assert!(exts.contains(&".node"));
    }
}
