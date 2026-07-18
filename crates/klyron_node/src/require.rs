use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::module;
use crate::NodeError;

type ExtHandler = fn(&Path, &module::Module) -> Result<Value, NodeError>;

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

    pub fn call(&self, specifier: &str) -> Result<Value, NodeError> {
        let resolved = module::Module::_resolve_filename(specifier, &self.current_file)?;
        module::Module::_load(&resolved, Some(&self.current_file))
    }

    pub fn resolve(&self, specifier: &str) -> Result<PathBuf, NodeError> {
        module::Module::_resolve_filename(specifier, &self.current_file)
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

fn ext_js(path: &Path, _module: &module::Module) -> Result<Value, NodeError> {
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

fn ext_json(path: &Path, _module: &module::Module) -> Result<Value, NodeError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| NodeError::Error(format!("Cannot read {}: {e}", path.display())))?;
    let val: Value = serde_json::from_str(&content)
        .map_err(|e| NodeError::SyntaxError(format!("Invalid JSON: {e}")))?;
    Ok(val)
}

fn ext_node(path: &Path, _module: &module::Module) -> Result<Value, NodeError> {
    Err(NodeError::Error(format!(
        "Native .node addon '{}' requires klyron_napi",
        path.display()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_require_fn_fs() {
        let req = RequireFn::new(Path::new("/index.js"));
        assert!(req.resolve("fs").is_ok());
    }

    #[test]
    fn test_require_fn_extensions() {
        let exts = RequireFn::extensions();
        assert!(exts.contains_key(".js"));
        assert!(exts.contains_key(".json"));
        assert!(exts.contains_key(".node"));
    }
}
