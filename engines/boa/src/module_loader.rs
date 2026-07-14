use crate::error::BoaError;
use klyron_engine_common::module_loader::CommonModuleLoader;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

pub struct BoaModuleLoader {
    modules: Mutex<HashMap<String, String>>,
    base_path: String,
}

impl BoaModuleLoader {
    pub fn new(base_path: &str) -> Self {
        Self {
            modules: Mutex::new(HashMap::new()),
            base_path: base_path.to_string(),
        }
    }

    pub fn resolve(&self, specifier: &str, base: &str) -> Result<String, BoaError> {
        if specifier.starts_with("file://") || specifier.starts_with('/') {
            return Ok(specifier.to_string());
        }
        if specifier.starts_with('.') {
            let base_dir = Path::new(base).parent().unwrap_or(Path::new("."));
            let resolved = base_dir.join(specifier);
            let resolved = resolved.to_string_lossy().to_string();
            return Ok(resolved);
        }
        let resolved = Path::new(&self.base_path).join("node_modules").join(specifier);
        if resolved.exists() {
            Ok(resolved.to_string_lossy().to_string())
        } else {
            Err(BoaError::CompileError(format!("Module not found: {}", specifier)))
        }
    }

    pub fn load(&self, path: &str) -> Result<String, BoaError> {
        let modules = self.modules.lock().unwrap();
        if let Some(content) = modules.get(path) {
            return Ok(content.clone());
        }
        drop(modules);
        std::fs::read_to_string(path)
            .map_err(|e| BoaError::CompileError(format!("Failed to load {}: {}", path, e)))
    }

    pub fn register(&self, name: &str, source: &str) {
        let mut modules = self.modules.lock().unwrap();
        modules.insert(name.to_string(), source.to_string());
    }

    pub fn instantiate(&self, path: &str, source: &str) -> Result<(), BoaError> {
        self.register(path, source);
        Ok(())
    }
}

impl CommonModuleLoader for BoaModuleLoader {
    fn resolve(&self, specifier: &str, base: &str) -> Result<String, String> {
        self.resolve(specifier, base).map_err(|e| e.to_string())
    }

    fn load(&self, path: &str) -> Result<String, String> {
        self.load(path).map_err(|e| e.to_string())
    }

    fn register(&self, name: &str, source: &str) {
        self.register(name, source)
    }
}
