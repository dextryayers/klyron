use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct NodeGlobals {
    pub global: serde_json::Value,
    pub __dirname: PathBuf,
    pub __filename: PathBuf,
}
impl NodeGlobals {
    pub fn new(entry_file: &Path) -> Self { Self { global: serde_json::Value::Object(serde_json::Map::new()), __dirname: entry_file.parent().unwrap_or(Path::new(".")).to_path_buf(), __filename: entry_file.to_path_buf() } }
    pub fn set_global(&mut self, key: &str, value: serde_json::Value) { if let serde_json::Value::Object(ref mut m) = self.global { m.insert(key.into(), value); } }
    pub fn get_global(&self, key: &str) -> Option<&serde_json::Value> { self.global.get(key) }
}

#[derive(Debug, Clone)]
pub struct NodeModuleRef {
    pub id: String,
    pub filename: PathBuf,
    pub loaded: bool,
}
impl NodeModuleRef {
    pub fn new(filename: &Path) -> Self { Self { id: filename.to_string_lossy().to_string(), filename: filename.to_path_buf(), loaded: false } }
}

pub struct RequireFunction;
impl RequireFunction {
    pub fn new(_current_file: &Path) -> Self { Self }
    pub fn require(&self, specifier: &str) -> anyhow::Result<HashMap<String, serde_json::Value>> { let _ = specifier; anyhow::bail!("require not available in bindings") }
    pub fn resolve(&self, specifier: &str) -> Option<PathBuf> { let _ = specifier; None }
    pub fn extensions() -> Vec<&'static str> { vec![".js", ".json", ".node", ".mjs", ".cjs"] }
}

pub struct RequireContext;
impl RequireContext {
    pub fn new(_current_file: &Path) -> Self { Self }
    pub fn is_core_module(name: &str) -> bool { matches!(name, "fs"|"path"|"http"|"https"|"os"|"crypto"|"stream"|"events"|"util"|"child_process"|"assert"|"buffer"|"url"|"timers"|"process"|"console"|"module"|"net"|"tls"|"dns") }
}
