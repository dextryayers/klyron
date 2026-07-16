use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleExport {
    pub name: String,
    pub module_path: PathBuf,
    pub content_hash: String,
    pub dependencies: Vec<String>,
}

pub struct HotpatchManager {
    modules: Mutex<HashMap<String, ModuleExport>>,
    dependency_graph: Mutex<HashMap<String, HashSet<String>>>,
    watched_paths: Mutex<HashSet<PathBuf>>,
    content_hashes: Mutex<HashMap<PathBuf, String>>,
}

impl HotpatchManager {
    pub fn new() -> Self {
        Self {
            modules: Mutex::new(HashMap::new()),
            dependency_graph: Mutex::new(HashMap::new()),
            watched_paths: Mutex::new(HashSet::new()),
            content_hashes: Mutex::new(HashMap::new()),
        }
    }

    pub fn register_module(&self, name: &str, path: &Path, content: &[u8], dependencies: Vec<String>) {
        let hash = blake3::hash(content).to_hex().to_string();
        let export = ModuleExport {
            name: name.to_string(),
            module_path: path.to_path_buf(),
            content_hash: hash.clone(),
            dependencies: dependencies.clone(),
        };

        self.modules.lock().unwrap().insert(name.to_string(), export);
        self.content_hashes.lock().unwrap().insert(path.to_path_buf(), hash);
        self.watched_paths.lock().unwrap().insert(path.to_path_buf());

        let mut graph = self.dependency_graph.lock().unwrap();
        let entry = graph.entry(name.to_string()).or_insert_with(HashSet::new);
        for dep in &dependencies {
            entry.insert(dep.clone());
        }
    }

    pub fn check_for_changes(&self) -> Vec<String> {
        let mut changed = Vec::new();
        let hashes = self.content_hashes.lock().unwrap();

        for (path, old_hash) in hashes.iter() {
            if let Ok(content) = std::fs::read(path) {
                let new_hash = blake3::hash(&content).to_hex().to_string();
                if new_hash != *old_hash {
                    if let Some(stem) = path.file_stem() {
                        changed.push(stem.to_string_lossy().to_string());
                    }
                }
            }
        }
        changed
    }

    pub fn update_module(&self, name: &str, content: &[u8]) -> Result<(), String> {
        let mut modules = self.modules.lock().unwrap();
        if let Some(export) = modules.get_mut(name) {
            let new_hash = blake3::hash(content).to_hex().to_string();
            export.content_hash = new_hash.clone();
            self.content_hashes.lock().unwrap()
                .insert(export.module_path.clone(), new_hash);
            Ok(())
        } else {
            Err(format!("Module '{}' not registered", name))
        }
    }

    pub fn get_dependents(&self, name: &str) -> Vec<String> {
        let graph = self.dependency_graph.lock().unwrap();
        let mut dependents = Vec::new();
        for (module, deps) in graph.iter() {
            if deps.contains(name) {
                dependents.push(module.clone());
            }
        }
        dependents
    }

    pub fn get_module(&self, name: &str) -> Option<ModuleExport> {
        self.modules.lock().unwrap().get(name).cloned()
    }

    pub fn all_modules(&self) -> Vec<ModuleExport> {
        self.modules.lock().unwrap().values().cloned().collect()
    }

    pub fn clear(&self) {
        self.modules.lock().unwrap().clear();
        self.dependency_graph.lock().unwrap().clear();
        self.watched_paths.lock().unwrap().clear();
        self.content_hashes.lock().unwrap().clear();
    }
}

impl Default for HotpatchManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_hotpatch_manager_new() {
        let manager = HotpatchManager::new();
        assert!(manager.all_modules().is_empty());
    }

    #[test]
    fn test_register_module() {
        let manager = HotpatchManager::new();
        let path = Path::new("/tmp/test_module.js");
        manager.register_module("test_mod", path, b"content", vec![]);
        let module = manager.get_module("test_mod").unwrap();
        assert_eq!(module.name, "test_mod");
        assert_eq!(module.module_path, path);
        assert!(!module.content_hash.is_empty());
    }

    #[test]
    fn test_register_module_with_deps() {
        let manager = HotpatchManager::new();
        let path = Path::new("/tmp/mod.js");
        manager.register_module("main", path, b"main content", vec!["dep1".to_string(), "dep2".to_string()]);
        let module = manager.get_module("main").unwrap();
        assert_eq!(module.dependencies.len(), 2);
    }

    #[test]
    fn test_get_module_nonexistent() {
        let manager = HotpatchManager::new();
        assert!(manager.get_module("nonexistent").is_none());
    }

    #[test]
    fn test_update_module() {
        let manager = HotpatchManager::new();
        let path = Path::new("/tmp/test.js");
        manager.register_module("m", path, b"old", vec![]);
        let old_hash = manager.get_module("m").unwrap().content_hash;
        manager.update_module("m", b"new content").unwrap();
        let new_hash = manager.get_module("m").unwrap().content_hash;
        assert_ne!(old_hash, new_hash);
    }

    #[test]
    fn test_update_nonexistent_module() {
        let manager = HotpatchManager::new();
        let result = manager.update_module("nonexistent", b"content");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_dependents() {
        let manager = HotpatchManager::new();
        let path = Path::new("/tmp/test.js");
        manager.register_module("parent", path, b"parent", vec!["child".to_string()]);
        let dependents = manager.get_dependents("child");
        assert_eq!(dependents, vec!["parent"]);
    }

    #[test]
    fn test_get_dependents_none() {
        let manager = HotpatchManager::new();
        let path = Path::new("/tmp/test.js");
        manager.register_module("orphan", path, b"orphan", vec![]);
        let dependents = manager.get_dependents("orphan");
        assert!(dependents.is_empty());
    }

    #[test]
    fn test_all_modules() {
        let manager = HotpatchManager::new();
        let p = Path::new("/tmp/a.js");
        manager.register_module("a", p, b"a", vec![]);
        manager.register_module("b", p, b"b", vec![]);
        assert_eq!(manager.all_modules().len(), 2);
    }

    #[test]
    fn test_clear() {
        let manager = HotpatchManager::new();
        let p = Path::new("/tmp/test.js");
        manager.register_module("m", p, b"content", vec![]);
        manager.clear();
        assert!(manager.all_modules().is_empty());
    }

    #[test]
    fn test_content_hash_consistency() {
        let manager = HotpatchManager::new();
        let p = Path::new("/tmp/test.js");
        manager.register_module("m", p, b"same content", vec![]);
        let hash1 = manager.get_module("m").unwrap().content_hash;
        manager.update_module("m", b"same content").unwrap();
        let hash2 = manager.get_module("m").unwrap().content_hash;
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_module_export_serialization() {
        let export = ModuleExport {
            name: "test".to_string(),
            module_path: PathBuf::from("/tmp/test.js"),
            content_hash: "abc123".to_string(),
            dependencies: vec!["dep".to_string()],
        };
        let json = serde_json::to_string(&export).unwrap();
        let deserialized: ModuleExport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test");
        assert_eq!(deserialized.dependencies, vec!["dep"]);
    }
}
