use crate::PmError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub name: String,
    pub url: String,
    pub token: Option<String>,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegistryStore {
    registries: Vec<RegistryConfig>,
    scope_mappings: HashMap<String, String>,
}

impl RegistryStore {
    fn empty() -> Self {
        Self {
            registries: vec![
                RegistryConfig {
                    name: "npm".into(),
                    url: "https://registry.npmjs.org".into(),
                    token: None,
                    priority: 0,
                },
            ],
            scope_mappings: HashMap::new(),
        }
    }
}

fn store_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("klyron")
        .join("registries.json")
}

fn load_store() -> RegistryStore {
    let path = store_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_else(|_| RegistryStore::empty())
    } else {
        RegistryStore::empty()
    }
}

fn save_store(store: &RegistryStore) -> Result<(), PmError> {
    let path = store_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)
        .map_err(|e| PmError::IoError(e.to_string()))?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_registry(name: &str, url: &str) -> Result<(), PmError> {
    let mut store = load_store();
    if store.registries.iter().any(|r| r.name == name) {
        return Err(PmError::IoError(format!("Registry '{name}' already exists")));
    }
    let max_pri = store.registries.iter().map(|r| r.priority).max().unwrap_or(0);
    store.registries.push(RegistryConfig {
        name: name.to_string(),
        url: url.to_string(),
        token: None,
        priority: max_pri + 1,
    });
    save_store(&store)
}

pub fn remove_registry(name: &str) -> Result<(), PmError> {
    let mut store = load_store();
    let len_before = store.registries.len();
    store.registries.retain(|r| r.name != name);
    store.scope_mappings.retain(|_, v| v != name);
    if store.registries.len() == len_before {
        return Err(PmError::IoError(format!("Registry '{name}' not found")));
    }
    save_store(&store)
}

pub fn list_registries() -> Vec<RegistryConfig> {
    let store = load_store();
    let mut regs = store.registries;
    regs.sort_by_key(|r| r.priority);
    regs
}

pub fn ping_registry(url: &str) -> bool {
    let ping_url = format!("{}/-/ping", url.trim_end_matches('/'));
    match reqwest::blocking::get(&ping_url) {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

pub fn map_scope(scope: &str, registry: &str) -> Result<(), PmError> {
    let mut store = load_store();
    if !store.registries.iter().any(|r| r.name == registry) {
        return Err(PmError::IoError(format!("Registry '{registry}' not found")));
    }
    let scope_key = if scope.starts_with('@') { scope.to_string() } else { format!("@{scope}") };
    store.scope_mappings.insert(scope_key, registry.to_string());
    save_store(&store)
}

pub fn unmap_scope(scope: &str) -> Result<(), PmError> {
    let mut store = load_store();
    let scope_key = if scope.starts_with('@') { scope.to_string() } else { format!("@{scope}") };
    if store.scope_mappings.remove(&scope_key).is_none() {
        return Err(PmError::IoError(format!("Scope '{scope_key}' not mapped")));
    }
    save_store(&store)
}

pub fn list_mapped_scopes() -> Vec<(String, String)> {
    let store = load_store();
    let mut mappings: Vec<(String, String)> = store.scope_mappings.into_iter().collect();
    mappings.sort_by(|a, b| a.0.cmp(&b.0));
    mappings
}

pub fn resolve_registry(package_name: &str) -> RegistryConfig {
    let store = load_store();
    if let Some(at_pos) = package_name.find('/') {
        let scope = &package_name[..at_pos];
        if let Some(reg_name) = store.scope_mappings.get(scope) {
            if let Some(reg) = store.registries.iter().find(|r| r.name == *reg_name) {
                return reg.clone();
            }
        }
    }
    store.registries.into_iter().min_by_key(|r| r.priority).unwrap_or_else(|| RegistryConfig {
        name: "npm".into(),
        url: "https://registry.npmjs.org".into(),
        token: None,
        priority: 0,
    })
}
