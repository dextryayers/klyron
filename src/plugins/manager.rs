use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::info;

use super::registry::PluginRegistry;
use super::types::{PluginConfig, PluginEntry, PluginMetadata, PluginPackage};

pub struct PluginManager {
    registry: Arc<RwLock<PluginRegistry>>,
    plugins_dir: PathBuf,
    config_path: PathBuf,
}

impl PluginManager {
    pub fn new(plugins_dir: PathBuf, config_path: PathBuf) -> Self {
        Self {
            registry: Arc::new(RwLock::new(PluginRegistry::new(plugins_dir.clone()))),
            plugins_dir,
            config_path,
        }
    }

    pub fn registry(&self) -> Arc<RwLock<PluginRegistry>> {
        self.registry.clone()
    }

    pub fn install(&self, source: &str, name: Option<&str>) -> Result<PluginPackage> {
        std::fs::create_dir_all(&self.plugins_dir)?;

        let plugin_name = name.unwrap_or_else(|| {
            Path::new(source)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
        }).to_string();

        let plugin_dir = self.plugins_dir.join(&plugin_name);
        std::fs::create_dir_all(&plugin_dir)?;

        let source_path = Path::new(source);
        if source_path.exists() && source_path.is_file() {
            let dest = plugin_dir.join(format!("{}.wasm", plugin_name));
            std::fs::copy(source_path, &dest)?;

            let manifest_src = source_path.parent().unwrap_or(Path::new(".")).join("klyron-plugin.json");
            if manifest_src.exists() {
                std::fs::copy(&manifest_src, plugin_dir.join("klyron-plugin.json"))?;
            }
        }

        let package = PluginPackage {
            name: plugin_name.clone(),
            version: "0.1.0".to_string(),
            source: source.to_string(),
            integrity: String::new(),
            manifest: serde_json::json!({}),
        };

        info!("Installed plugin: {} from {}", plugin_name, source);
        Ok(package)
    }

    pub fn uninstall(&self, name: &str) -> Result<()> {
        let plugin_dir = self.plugins_dir.join(name);
        if plugin_dir.exists() {
            std::fs::remove_dir_all(&plugin_dir)?;
        }
        self.registry.write().unregister(name);
        info!("Uninstalled plugin: {}", name);
        Ok(())
    }

    pub fn enable(&self, name: &str) -> Result<()> {
        self.registry.write().enable(name)
    }

    pub fn disable(&self, name: &str) -> Result<()> {
        self.registry.write().disable(name)
    }

    pub fn list(&self) -> Vec<String> {
        self.registry.read().list()
    }

    pub fn search(&self, query: &str) -> Vec<PluginPackage> {
        let q = query.to_lowercase();
        let mut results = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if name.to_lowercase().contains(&q) {
                    results.push(PluginPackage {
                        name: name.to_string(),
                        version: "0.1.0".to_string(),
                        source: path.to_string_lossy().to_string(),
                        integrity: String::new(),
                        manifest: serde_json::json!({}),
                    });
                }
            }
        }

        results
    }

    pub fn load_plugins(&self) -> Vec<String> {
        self.registry.write().load_all()
    }

    pub fn refresh(&self) -> Result<()> {
        self.registry.write().refresh_installed()?;
        Ok(())
    }

    pub fn get_info(&self, name: &str) -> Option<PluginEntry> {
        self.registry.read().get_info(name).map(|info| {
            PluginEntry {
                name: info.name.clone(),
                version: info.version.clone(),
                enabled: info.enabled,
                permissions: Vec::new(),
                config: None,
            }
        })
    }
}
