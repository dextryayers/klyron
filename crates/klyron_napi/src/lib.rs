use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NapiModule {
    pub name: String,
    pub exports: HashMap<String, serde_json::Value>,
}

pub struct NapiLoader {
    loaded_modules: HashMap<String, NapiModule>,
}

impl NapiLoader {
    pub fn new() -> Self {
        Self { loaded_modules: HashMap::new() }
    }

    pub fn load(&mut self, name: &str) -> anyhow::Result<&NapiModule> {
        if self.loaded_modules.contains_key(name) {
            return Ok(self.loaded_modules.get(name).unwrap());
        }
        let module = self.load_native_module(name)?;
        self.loaded_modules.insert(name.to_string(), module);
        Ok(self.loaded_modules.get(name).unwrap())
    }

    fn load_native_module(&self, name: &str) -> anyhow::Result<NapiModule> {
        let node_modules_path = std::env::current_dir()
            .unwrap_or_default()
            .join("node_modules")
            .join(name);

        let binding_path = if cfg!(target_os = "linux") {
            node_modules_path.join(&format!("{name}.linux-x64-gnu.node"))
        } else if cfg!(target_os = "macos") {
            node_modules_path.join(&format!("{name}.darwin-x64.node"))
        } else {
            node_modules_path.join(&format!("{name}.win32-x64-msvc.node"))
        };

        if !binding_path.exists() {
            anyhow::bail!("N-API module '{name}' not found at: {}", binding_path.display());
        }

        println!("Loading N-API module: {} (from {})", name, binding_path.display());

        Ok(NapiModule {
            name: name.to_string(),
            exports: HashMap::new(),
        })
    }

    pub fn list_loaded(&self) -> Vec<String> {
        self.loaded_modules.keys().cloned().collect()
    }

    pub fn is_loaded(&self, name: &str) -> bool {
        self.loaded_modules.contains_key(name)
    }

    pub fn unload(&mut self, name: &str) -> bool {
        self.loaded_modules.remove(name).is_some()
    }

    pub fn clear(&mut self) {
        self.loaded_modules.clear();
    }

    pub fn symbol_count(&self) -> usize {
        self.loaded_modules.values().map(|m| m.exports.len()).sum()
    }

    pub fn napi_version(&self) -> u32 { 9 }

    pub fn is_napi_module(name: &str) -> bool {
        name.ends_with(".node")
    }
}

impl Default for NapiLoader {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_napi_loader_new() {
        let loader = NapiLoader::new();
        assert_eq!(loader.list_loaded().len(), 0);
        assert_eq!(loader.napi_version(), 9);
    }

    #[test]
    fn test_is_napi_module() {
        assert!(NapiLoader::is_napi_module("addon.node"));
        assert!(!NapiLoader::is_napi_module("addon.so"));
    }
}
