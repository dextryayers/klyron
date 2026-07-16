use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub enum ModuleType {
    EsModule,
    CommonJs,
    Json,
    Wasm,
}

#[derive(Debug, Clone)]
pub struct ESModule {
    pub specifier: String,
    pub source: String,
    pub module_type: ModuleType,
    pub resolved_path: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug)]
pub struct ESModuleLoader {
    modules: Mutex<HashMap<String, ESModule>>,
    base_path: String,
    extensions: Vec<String>,
}

impl ESModuleLoader {
    pub fn new(base_path: &str) -> Self {
        Self {
            modules: Mutex::new(HashMap::new()),
            base_path: base_path.to_string(),
            extensions: vec![".js".to_string(), ".mjs".to_string(), ".json".to_string()],
        }
    }

    pub fn resolve(&self, specifier: &str, base: &str) -> Result<String, String> {
        if specifier.starts_with("file://") || specifier.starts_with('/') {
            let path = specifier.strip_prefix("file://").unwrap_or(specifier);
            return Ok(path.to_string());
        }
        if specifier.starts_with('.') {
            let base_dir = Path::new(base).parent().unwrap_or(Path::new("."));
            let resolved = base_dir.join(specifier);
            for ext in &self.extensions {
                let with_ext = resolved.with_extension(ext.trim_start_matches('.'));
                if with_ext.exists() {
                    return Ok(with_ext.to_string_lossy().to_string());
                }
            }
            if resolved.exists() {
                return Ok(resolved.to_string_lossy().to_string());
            }
            return Err(format!("Module not found: {}", specifier));
        }
        let node_modules = Path::new(&self.base_path).join("node_modules").join(specifier);
        if node_modules.exists() {
            return Ok(node_modules.to_string_lossy().to_string());
        }
        Err(format!("Module not found: {}", specifier))
    }

    pub fn load(&self, path: &str) -> Result<String, String> {
        let modules = self.modules.lock().unwrap();
        if let Some(module) = modules.get(path) {
            return Ok(module.source.clone());
        }
        drop(modules);
        std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to load {}: {}", path, e))
    }

    pub fn register(&self, name: &str, source: &str, module_type: ModuleType) {
        let mut modules = self.modules.lock().unwrap();
        modules.insert(
            name.to_string(),
            ESModule {
                specifier: name.to_string(),
                source: source.to_string(),
                module_type,
                resolved_path: None,
                dependencies: Vec::new(),
            },
        );
    }

    pub fn instantiate(&self, path: &str, source: &str) -> Result<(), String> {
        self.register(path, source, ModuleType::EsModule);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<ESModule> {
        self.modules.lock().ok()?.get(name).cloned()
    }

    pub fn has(&self, name: &str) -> bool {
        self.modules.lock().map(|m| m.contains_key(name)).unwrap_or(false)
    }

    pub fn resolve_import_map(&self, specifier: &str) -> Option<String> {
        let import_map_path = Path::new(&self.base_path).join("import_map.json");
        if !import_map_path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(import_map_path).ok()?;
        let map: HashMap<String, String> = serde_json::from_str(&content).ok()?;
        map.get(specifier).cloned()
    }

    pub fn set_extensions(&mut self, exts: Vec<String>) {
        self.extensions = exts;
    }
}

pub trait ModuleLoader {
    fn resolve(&self, specifier: &str, base: &str) -> Result<String, String>;
    fn load(&self, path: &str) -> Result<String, String>;
    fn register(&self, name: &str, source: &str);
    fn has(&self, name: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_es_module_loader_new() {
        let loader = ESModuleLoader::new("/base/path");
        assert!(!loader.has("nonexistent"));
    }

    #[test]
    fn test_register_and_get() {
        let loader = ESModuleLoader::new("/base/path");
        loader.register("mod", "export const x = 1;", ModuleType::EsModule);
        assert!(loader.has("mod"));
        let module = loader.get("mod").unwrap();
        assert_eq!(module.specifier, "mod");
        assert_eq!(module.source, "export const x = 1;");
    }

    #[test]
    fn test_register_json_module() {
        let loader = ESModuleLoader::new("/base/path");
        loader.register("data.json", r#"{"key":"value"}"#, ModuleType::Json);
        let module = loader.get("data.json").unwrap();
        assert_eq!(module.source, r#"{"key":"value"}"#);
    }

    #[test]
    fn test_instantiate() {
        let loader = ESModuleLoader::new("/base/path");
        loader.instantiate("mod", "code").unwrap();
        assert!(loader.has("mod"));
    }

    #[test]
    fn test_resolve_absolute_path() {
        let loader = ESModuleLoader::new("/base");
        let result = loader.resolve("/absolute/path.js", "/base");
        assert_eq!(result.unwrap(), "/absolute/path.js");
    }

    #[test]
    fn test_resolve_file_url() {
        let loader = ESModuleLoader::new("/base");
        let result = loader.resolve("file:///some/path.js", "/base");
        assert_eq!(result.unwrap(), "/some/path.js");
    }

    #[test]
    fn test_resolve_nonexistent_relative() {
        let loader = ESModuleLoader::new("/nonexistent_base");
        let result = loader.resolve("./nonexistent_file.js", "/nonexistent_base/mod.js");
        assert!(result.is_err());
    }

    #[test]
    fn test_module_type_enum() {
        match ModuleType::EsModule {
            ModuleType::EsModule => {}
            _ => panic!("expected EsModule"),
        }
        match ModuleType::CommonJs {
            ModuleType::CommonJs => {}
            _ => panic!("expected CommonJs"),
        }
    }

    #[test]
    fn test_es_module_structure() {
        let module = ESModule {
            specifier: "test".to_string(),
            source: "code".to_string(),
            module_type: ModuleType::Wasm,
            resolved_path: Some("/path".to_string()),
            dependencies: vec!["dep".to_string()],
        };
        assert_eq!(module.specifier, "test");
        assert_eq!(module.dependencies.len(), 1);
    }

    #[test]
    fn test_set_extensions() {
        let mut loader = ESModuleLoader::new("/base");
        loader.set_extensions(vec![".ts".to_string(), ".js".to_string()]);
        let result = loader.resolve("./mod.ts", "/base/mod.js");
        assert!(result.is_err()); // no filesystem, so expected
    }

    #[test]
    fn test_resolve_import_map_nonexistent() {
        let loader = ESModuleLoader::new("/nonexistent");
        let result = loader.resolve_import_map("some_module");
        assert!(result.is_none());
    }

    #[test]
    fn test_register_deduplication() {
        let loader = ESModuleLoader::new("/base");
        loader.register("mod", "v1", ModuleType::EsModule);
        loader.register("mod", "v2", ModuleType::EsModule);
        let module = loader.get("mod").unwrap();
        assert_eq!(module.source, "v2");
    }
}
