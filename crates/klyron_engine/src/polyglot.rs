use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Cross-language data types for FFI bridge
#[derive(Debug, Clone)]
pub enum FFIValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<FFIValue>),
    Map(HashMap<String, FFIValue>),
    Buffer(Vec<u8>),
}

impl FFIValue {
    pub fn from_json(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Bool(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() { Self::Int(i) }
                else if let Some(f) = n.as_f64() { Self::Float(f) }
                else { Self::Null }
            }
            serde_json::Value::String(s) => Self::String(s.clone()),
            serde_json::Value::Array(arr) => Self::Array(arr.iter().map(Self::from_json).collect()),
            serde_json::Value::Object(obj) => Self::Map(
                obj.iter().map(|(k, v)| (k.clone(), Self::from_json(v))).collect()
            ),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Null => serde_json::Value::Null,
            Self::Bool(b) => serde_json::Value::Bool(*b),
            Self::Int(i) => serde_json::json!(i),
            Self::Float(f) => serde_json::json!(f),
            Self::String(s) => serde_json::Value::String(s.clone()),
            Self::Array(arr) => serde_json::Value::Array(arr.iter().map(|v| v.to_json()).collect()),
            Self::Map(m) => {
                let obj: serde_json::Map<String, serde_json::Value> = m.iter()
                    .map(|(k, v)| (k.clone(), v.to_json()))
                    .collect();
                serde_json::Value::Object(obj)
            }
            Self::Buffer(b) => serde_json::Value::Array(b.iter().map(|n| serde_json::json!(n)).collect()),
        }
    }
}

/// Engine metadata
#[derive(Debug, Clone)]
pub struct EngineMetadata {
    pub name: String,
    pub language: String,
    pub version: String,
    pub available: bool,
}

/// Polyglot engine error
#[derive(Debug, Clone)]
pub struct EngineError(pub String);

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for EngineError {}

impl From<String> for EngineError {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Common trait for all language engine runtimes
#[async_trait]
pub trait PolyglotEngine: Send + Sync {
    fn language(&self) -> &'static str;
    async fn eval(&self, code: &str) -> Result<String, EngineError>;
    async fn health_check(&self) -> Result<(), EngineError>;
    fn metadata(&self) -> EngineMetadata;
}

/// WASM-based engine using wasmtime
#[allow(dead_code)]
pub struct WasmEngine {
    module: wasmtime::Module,
    store: wasmtime::Store<()>,
    linker: wasmtime::Linker<()>,
    language_name: &'static str,
}

impl WasmEngine {
    pub fn new(wasm_bytes: &[u8], language: &'static str) -> Result<Self, String> {
        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::new(&engine, wasm_bytes)
            .map_err(|e| format!("Failed to compile WASM module: {e}"))?;
        let store = wasmtime::Store::new(&engine, ());
        let linker = wasmtime::Linker::new(&engine);
        Ok(Self { module, store, linker, language_name: language })
    }

    pub fn from_file(path: &Path, language: &'static str) -> Result<Self, String> {
        let wasm_bytes = std::fs::read(path)
            .map_err(|e| format!("Failed to read WASM file: {e}"))?;
        Self::new(&wasm_bytes, language)
    }
}

#[async_trait]
impl PolyglotEngine for WasmEngine {
    fn language(&self) -> &'static str {
        self.language_name
    }

    async fn eval(&self, _code: &str) -> Result<String, EngineError> {
        Err(EngineError("WASM eval not yet implemented".to_string()))
    }

    async fn health_check(&self) -> Result<(), EngineError> {
        Ok(())
    }

    fn metadata(&self) -> EngineMetadata {
        EngineMetadata {
            name: format!("wasm-{}", self.language_name),
            language: self.language_name.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            available: true,
        }
    }
}

/// Subprocess-based engine for native language runtimes
pub struct SubprocessEngine {
    binary_path: PathBuf,
    protocol: JsonProtocol,
    language_name: &'static str,
}

/// JSON-based stdin/stdout protocol for subprocess communication
pub struct JsonProtocol;

impl JsonProtocol {
    pub fn new() -> Self {
        Self
    }

    pub fn encode_request(&self, action: &str, code: &str) -> String {
        serde_json::json!({
            "action": action,
            "code": code,
        }).to_string()
    }

    pub fn decode_response(&self, data: &str) -> Result<String, String> {
        let parsed: serde_json::Value = serde_json::from_str(data)
            .map_err(|e| format!("Invalid response JSON: {e}"))?;
        parsed["output"].as_str()
            .map(|s| s.to_string())
            .or_else(|| parsed["error"].as_str().map(|s| s.to_string()))
            .ok_or_else(|| "Missing output field in response".to_string())
    }
}

impl SubprocessEngine {
    pub fn new(binary: &str, language: &'static str) -> Self {
        Self {
            binary_path: PathBuf::from(binary),
            protocol: JsonProtocol::new(),
            language_name: language,
        }
    }

    async fn execute(&self, code: &str) -> Result<String, EngineError> {
        use std::process::{Command, Stdio};
        use std::io::Write;

        let request = self.protocol.encode_request("eval", code);
        let mut child = Command::new(&self.binary_path)
            .arg("--eval")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| EngineError(format!("Failed to spawn {}: {e}", self.language_name)))?;

        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(request.as_bytes())
                .map_err(|e| EngineError(format!("Failed to write to stdin: {e}")))?;
        }

        let output = child.wait_with_output()
            .map_err(|e| EngineError(format!("Failed to wait for process: {e}")))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            self.protocol.decode_response(&stdout)
                .map_err(|e| EngineError(format!("Protocol error: {e}")))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(EngineError(format!("Process failed ({}): {stderr}", output.status)))
        }
    }
}

#[async_trait]
impl PolyglotEngine for SubprocessEngine {
    fn language(&self) -> &'static str {
        self.language_name
    }

    async fn eval(&self, code: &str) -> Result<String, EngineError> {
        self.execute(code).await
    }

    async fn health_check(&self) -> Result<(), EngineError> {
        use std::process::Command;
        Command::new(&self.binary_path)
            .arg("--version")
            .output()
            .map_err(|e| EngineError(format!("Health check failed: {e}")))?;
        Ok(())
    }

    fn metadata(&self) -> EngineMetadata {
        EngineMetadata {
            name: format!("subprocess-{}", self.language_name),
            language: self.language_name.to_string(),
            version: "unknown".to_string(),
            available: true,
        }
    }
}

/// Resolved module information
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    pub path: PathBuf,
    pub engine: EngineKind,
}

/// Polyglot engine kinds for module resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineKind {
    V8,
    Boa,
    QuickJS,
    JSC,
    Php,
    Python,
    Ruby,
    Go,
    Rust,
    C,
    Cpp,
    Zig,
}

impl EngineKind {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "js" | "ts" | "tsx" | "jsx" | "mjs" | "cjs" => Self::V8,
            "php" => Self::Php,
            "py" => Self::Python,
            "rb" => Self::Ruby,
            "go" => Self::Go,
            "rs" => Self::Rust,
            "c" => Self::C,
            "cpp" | "cc" | "cxx" => Self::Cpp,
            "zig" => Self::Zig,
            _ => Self::V8,
        }
    }
}

impl std::fmt::Display for EngineKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V8 => write!(f, "v8"),
            Self::Boa => write!(f, "boa"),
            Self::QuickJS => write!(f, "quickjs"),
            Self::JSC => write!(f, "jsc"),
            Self::Php => write!(f, "php"),
            Self::Python => write!(f, "python"),
            Self::Ruby => write!(f, "ruby"),
            Self::Go => write!(f, "go"),
            Self::Rust => write!(f, "rust"),
            Self::C => write!(f, "c"),
            Self::Cpp => write!(f, "cpp"),
            Self::Zig => write!(f, "zig"),
        }
    }
}

/// Module resolver for cross-language imports
pub struct ModuleResolver {
    extensions: HashMap<String, EngineKind>,
    module_paths: Vec<PathBuf>,
}

impl ModuleResolver {
    pub fn new() -> Self {
        let mut extensions = HashMap::new();
        extensions.insert("js".to_string(), EngineKind::V8);
        extensions.insert("ts".to_string(), EngineKind::V8);
        extensions.insert("tsx".to_string(), EngineKind::V8);
        extensions.insert("jsx".to_string(), EngineKind::V8);
        extensions.insert("mjs".to_string(), EngineKind::V8);
        extensions.insert("cjs".to_string(), EngineKind::V8);
        extensions.insert("php".to_string(), EngineKind::Php);
        extensions.insert("py".to_string(), EngineKind::Python);
        extensions.insert("rb".to_string(), EngineKind::Ruby);
        extensions.insert("go".to_string(), EngineKind::Go);
        extensions.insert("rs".to_string(), EngineKind::Rust);
        extensions.insert("c".to_string(), EngineKind::C);
        extensions.insert("cpp".to_string(), EngineKind::Cpp);
        extensions.insert("cc".to_string(), EngineKind::Cpp);
        extensions.insert("cxx".to_string(), EngineKind::Cpp);
        extensions.insert("zig".to_string(), EngineKind::Zig);

        Self {
            extensions,
            module_paths: Vec::new(),
        }
    }

    pub fn with_module_path(mut self, path: PathBuf) -> Self {
        self.module_paths.push(path);
        self
    }

    /// Resolve a specifier to a module path and engine
    pub fn resolve(&self, specifier: &str, referrer: &str) -> Result<ResolvedModule, String> {
        let path = self.resolve_path(specifier, referrer)?;
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("js")
            .to_lowercase();
        let engine = self.extensions.get(&ext)
            .copied()
            .unwrap_or(EngineKind::V8);
        Ok(ResolvedModule { path, engine })
    }

    fn resolve_path(&self, specifier: &str, referrer: &str) -> Result<PathBuf, String> {
        // Check if it's a relative path
        if specifier.starts_with("./") || specifier.starts_with("../") {
            let referrer_dir = Path::new(referrer).parent()
                .ok_or_else(|| "Invalid referrer path".to_string())?;
            let candidate = referrer_dir.join(specifier);

            // Try with common extensions
            let extensions = [".js", ".ts", ".tsx", ".jsx", ".mjs", ".cjs", ".json"];
            for ext in &extensions {
                let with_ext = candidate.with_extension(ext.trim_start_matches('.'));
                if with_ext.exists() {
                    return Ok(with_ext);
                }
            }
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        // Search module paths
        for base in &self.module_paths {
            let candidate = base.join(specifier);
            if candidate.exists() {
                return Ok(candidate);
            }
            // Try node_modules
            let node_module = base.join("node_modules").join(specifier);
            if node_module.exists() {
                return Ok(node_module);
            }
        }

        // Try bare specifier in node_modules of referrer
        let referrer_dir = Path::new(referrer).parent().unwrap_or(Path::new("."));
        let node_mod = referrer_dir.join("node_modules").join(specifier);

        // Try with index.js
        let index = node_mod.join("index.js");
        if index.exists() {
            return Ok(index);
        }
        let index_ts = node_mod.join("index.ts");
        if index_ts.exists() {
            return Ok(index_ts);
        }

        Err(format!("Cannot resolve module: {specifier} from {referrer}"))
    }

    /// Register a new extension-to-engine mapping
    pub fn register_extension(&mut self, ext: &str, engine: EngineKind) {
        self.extensions.insert(ext.to_string(), engine);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_value_from_json() {
        let json = serde_json::json!({"name": "test", "count": 42});
        let ffi = FFIValue::from_json(&json);
        match ffi {
            FFIValue::Map(ref m) => {
                assert_eq!(m.len(), 2);
            }
            _ => panic!("Expected Map"),
        }
    }

    #[test]
    fn test_ffi_value_roundtrip() {
        let original = serde_json::json!({"a": 1, "b": [2, 3, null]});
        let ffi = FFIValue::from_json(&original);
        let result = ffi.to_json();
        assert_eq!(original, result);
    }

    #[test]
    fn test_engine_kind_from_extension() {
        assert_eq!(EngineKind::from_extension("js"), EngineKind::V8);
        assert_eq!(EngineKind::from_extension("php"), EngineKind::Php);
        assert_eq!(EngineKind::from_extension("py"), EngineKind::Python);
        assert_eq!(EngineKind::from_extension("rs"), EngineKind::Rust);
        assert_eq!(EngineKind::from_extension("zig"), EngineKind::Zig);
    }

    #[test]
    fn test_module_resolver_extension_map() {
        let _resolver = ModuleResolver::new();
        let js_ext = EngineKind::from_extension("js");
        let php_ext = EngineKind::from_extension("php");
        assert_eq!(js_ext, EngineKind::V8);
        assert_eq!(php_ext, EngineKind::Php);
    }

    #[test]
    fn test_wasm_engine_metadata() {
        let engine = wasmtime::Engine::default();
        let wasm = wat::parse_str("(module)").unwrap();
        let module = wasmtime::Module::new(&engine, &wasm).unwrap();
        let store = wasmtime::Store::new(&engine, ());
        let linker = wasmtime::Linker::new(&engine);

        let wasm_engine = WasmEngine {
            module,
            store,
            linker,
            language_name: "test",
        };

        let meta = wasm_engine.metadata();
        assert_eq!(meta.language, "test");
        assert!(meta.available);
    }

    #[test]
    fn test_subprocess_engine_creation() {
        let engine = SubprocessEngine::new("php", "php");
        assert_eq!(engine.language(), "php");
        assert!(!engine.binary_path.to_string_lossy().is_empty());
    }

    #[test]
    fn test_json_protocol() {
        let protocol = JsonProtocol::new();
        let request = protocol.encode_request("eval", "1 + 1");
        assert!(request.contains("eval"));
        assert!(request.contains("1 + 1"));

        let response = protocol.decode_response(r#"{"output": "2"}"#).unwrap();
        assert_eq!(response, "2");

        let error = protocol.decode_response(r#"{"error": "syntax error"}"#).unwrap();
        assert_eq!(error, "syntax error");
    }

    #[test]
    fn test_resolved_module_creation() {
        let module = ResolvedModule {
            path: PathBuf::from("test.php"),
            engine: EngineKind::Php,
        };
        assert_eq!(module.engine, EngineKind::Php);
        assert!(module.path.to_string_lossy().ends_with("test.php"));
    }
}
