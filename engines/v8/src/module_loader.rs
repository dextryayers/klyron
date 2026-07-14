use crate::error::V8Error;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

pub struct V8ModuleLoader {
    modules: Mutex<HashMap<String, String>>,
    base_path: String,
}

impl V8ModuleLoader {
    pub fn new(base_path: &str) -> Self {
        Self {
            modules: Mutex::new(HashMap::new()),
            base_path: base_path.to_string(),
        }
    }

    pub fn resolve(&self, specifier: &str, base: &str) -> Result<String, V8Error> {
        if specifier.starts_with("file://") || specifier.starts_with('/') {
            return Ok(specifier.to_string());
        }
        if specifier.starts_with('.') {
            let base_dir = Path::new(base).parent().unwrap_or(Path::new("."));
            let resolved = base_dir.join(specifier);
            return Ok(resolved.to_string_lossy().to_string());
        }
        let resolved = Path::new(&self.base_path).join("node_modules").join(specifier);
        if resolved.exists() {
            Ok(resolved.to_string_lossy().to_string())
        } else {
            Err(V8Error::CompileError(format!("Module not found: {}", specifier)))
        }
    }

    pub fn load(&self, path: &str) -> Result<String, V8Error> {
        let modules = self.modules.lock().unwrap();
        if let Some(content) = modules.get(path) {
            return Ok(content.clone());
        }
        drop(modules);
        std::fs::read_to_string(path)
            .map_err(|e| V8Error::CompileError(format!("Failed to load {}: {}", path, e)))
    }

    pub fn register(&self, name: &str, source: &str) {
        let mut modules = self.modules.lock().unwrap();
        modules.insert(name.to_string(), source.to_string());
    }

    pub fn instantiate(&self, path: &str, source: &str) -> Result<(), V8Error> {
        self.register(path, source);
        Ok(())
    }
}
