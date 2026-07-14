use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

pub trait CommonModuleLoader: Send + Sync {
    fn resolve(&self, specifier: &str, base: &str) -> Result<String, String>;
    fn load(&self, path: &str) -> Result<String, String>;
    fn register(&self, name: &str, source: &str);
    fn resolve_path(&self, specifier: &str, base: &str) -> Result<String, String> {
        if specifier.starts_with("file://") || specifier.starts_with('/') {
            let path = specifier.strip_prefix("file://").unwrap_or(specifier);
            return Ok(path.to_string());
        }
        if specifier.starts_with('.') {
            let base_dir = Path::new(base).parent().unwrap_or(Path::new("."));
            let resolved = base_dir.join(specifier);
            let path = resolved.to_string_lossy().to_string();
            return Ok(path);
        }
        Err(format!("Unresolved module: {}", specifier))
    }
}

#[derive(Debug)]
pub struct SharedModuleLoader {
    modules: Mutex<HashMap<String, String>>,
    base_path: String,
}

impl SharedModuleLoader {
    pub fn new(base_path: &str) -> Self {
        Self {
            modules: Mutex::new(HashMap::new()),
            base_path: base_path.to_string(),
        }
    }
}

impl CommonModuleLoader for SharedModuleLoader {
    fn resolve(&self, specifier: &str, base: &str) -> Result<String, String> {
        if specifier.starts_with("file://") || specifier.starts_with('/') {
            return Ok(specifier.to_string());
        }
        if specifier.starts_with('.') {
            let base_dir = Path::new(base).parent().unwrap_or(Path::new("."));
            let resolved = base_dir.join(specifier);
            return Ok(resolved.to_string_lossy().to_string());
        }
        let resolved = Path::new(&self.base_path)
            .join("node_modules")
            .join(specifier);
        if resolved.exists() {
            Ok(resolved.to_string_lossy().to_string())
        } else {
            Err(format!("Module not found: {}", specifier))
        }
    }

    fn load(&self, path: &str) -> Result<String, String> {
        let modules = self.modules.lock().unwrap();
        if let Some(content) = modules.get(path) {
            return Ok(content.clone());
        }
        drop(modules);
        std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to load {}: {}", path, e))
    }

    fn register(&self, name: &str, source: &str) {
        let mut modules = self.modules.lock().unwrap();
        modules.insert(name.to_string(), source.to_string());
    }
}
