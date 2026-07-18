use std::collections::HashMap;
use std::sync::Mutex;

#[cfg(feature = "native")]
use crate::ffi;

pub struct JSCModule {
    pub name: String,
    pub source: String,
    pub resolved: bool,
    pub evaluated: bool,
}

pub struct JSCModuleSystem {
    modules: Mutex<HashMap<String, JSCModule>>,
    base_path: String,
}

impl JSCModuleSystem {
    pub fn new(base_path: &str) -> Self {
        Self {
            modules: Mutex::new(HashMap::new()),
            base_path: base_path.to_string(),
        }
    }

    pub fn register(&self, name: &str, source: &str) {
        let mut modules = self.modules.lock().unwrap();
        modules.insert(name.to_string(), JSCModule {
            name: name.to_string(),
            source: source.to_string(),
            resolved: false,
            evaluated: false,
        });
    }

    pub fn get(&self, name: &str) -> Option<String> {
        let modules = self.modules.lock().unwrap();
        modules.get(name).map(|m| m.source.clone())
    }

    pub fn resolve(&self, specifier: &str, base: &str) -> Result<String, String> {
        if specifier.starts_with("file://") || specifier.starts_with('/') {
            return Ok(specifier.to_string());
        }
        if specifier.starts_with('.') {
            let base_dir = std::path::Path::new(base)
                .parent()
                .unwrap_or(std::path::Path::new("."));
            let resolved = base_dir.join(specifier);
            return Ok(resolved.to_string_lossy().to_string());
        }
        let resolved = std::path::Path::new(&self.base_path)
            .join("node_modules")
            .join(specifier);
        if resolved.exists() {
            Ok(resolved.to_string_lossy().to_string())
        } else {
            Err(format!("Module not found: {specifier}"))
        }
    }

    #[cfg(feature = "native")]
    pub fn compile(&self, engine: &crate::ffi::JSCEnginePtr, source: &str, origin: &str) -> Result<*mut ffi::JSCValueHandle, String> {
        engine.module_compile(source, Some(origin))
    }

    #[cfg(feature = "native")]
    pub fn instantiate(&self, engine: &crate::ffi::JSCEnginePtr, module: *mut ffi::JSCValueHandle) -> Result<(), String> {
        engine.module_instantiate(module)
    }

    #[cfg(feature = "native")]
    pub fn evaluate(&self, engine: &crate::ffi::JSCEnginePtr, module: *mut ffi::JSCValueHandle) -> Result<String, String> {
        engine.module_evaluate(module)
    }
}
