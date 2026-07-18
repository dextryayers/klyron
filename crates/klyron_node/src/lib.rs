pub mod buffer;
pub mod event;
pub mod module;
pub mod require;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub use buffer::Buffer;
pub use event::EventEmitter;
pub use module::Module;
pub use require::RequireFn;

use serde_json::{json, Value};
use thiserror::Error;

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

    pub fn cwd(&self) -> PathBuf {
        std::env::current_dir().unwrap_or_default()
    }

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
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
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
        let require_fn = RequireFn::new(entry_file);

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

pub fn get_dirname(path: &Path) -> PathBuf {
    path.parent().unwrap_or(Path::new(".")).to_path_buf()
}

pub fn get_filename(path: &Path) -> PathBuf {
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

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
    fn test_error_types() {
        let e1 = NodeError::TypeError("bad type".into());
        assert_eq!(e1.js_type(), "TypeError");
        assert!(e1.to_string().contains("bad type"));
    }
}
