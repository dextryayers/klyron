pub mod remote;
pub mod local;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::Result;
use parking_lot::RwLock;
use tracing::info;

use super::types::{PluginConfig, PluginMetadata};

pub struct PluginRegistry {
    plugins: HashMap<String, PluginEntry>,
    plugins_dir: PathBuf,
}

pub struct PluginEntry {
    pub name: String,
    pub version: String,
    pub metadata: PluginMetadata,
    pub config: PluginConfig,
    pub enabled: bool,
    pub install_path: PathBuf,
}

impl PluginRegistry {
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            plugins_dir,
        }
    }

    pub fn register(&mut self, name: &str, version: &str, metadata: PluginMetadata, path: PathBuf) {
        let entry = PluginEntry {
            name: name.to_string(),
            version: version.to_string(),
            metadata,
            config: PluginConfig::default(),
            enabled: true,
            install_path: path,
        };
        self.plugins.insert(name.to_string(), entry);
        info!("Registered plugin: {}", name);
    }

    pub fn unregister(&mut self, name: &str) {
        self.plugins.remove(name);
        info!("Unregistered plugin: {}", name);
    }

    pub fn get(&self, name: &str) -> Option<&PluginEntry> {
        self.plugins.get(name)
    }

    pub fn get_info(&self, name: &str) -> Option<&PluginEntry> {
        self.plugins.get(name)
    }

    pub fn list(&self) -> Vec<String> {
        let mut names: Vec<String> = self.plugins.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn enable(&mut self, name: &str) -> Result<()> {
        let entry = self.plugins.get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;
        entry.enabled = true;
        Ok(())
    }

    pub fn disable(&mut self, name: &str) -> Result<()> {
        let entry = self.plugins.get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;
        entry.enabled = false;
        Ok(())
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.plugins.get(name).map(|e| e.enabled).unwrap_or(false)
    }

    pub fn load_all(&mut self) -> Vec<String> {
        let mut loaded = Vec::new();
        if !self.plugins_dir.exists() {
            return loaded;
        }
        if let Ok(entries) = std::fs::read_dir(&self.plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = match path.file_name().and_then(|s| s.to_str()) {
                    Some(n) => n.to_string(),
                    None => continue,
                };
                if self.plugins.contains_key(&name) {
                    continue;
                }
                let manifest_path = path.join("klyron-plugin.json");
                if !manifest_path.exists() {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                        let version = manifest.get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("0.1.0")
                            .to_string();
                        let metadata = PluginMetadata::new(&name, &version);
                        self.register(&name, &version, metadata, path);
                        loaded.push(name);
                    }
                }
            }
        }
        loaded
    }

    pub fn refresh_installed(&mut self) -> Result<()> {
        self.load_all();
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}
