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
