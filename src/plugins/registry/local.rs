use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tracing::info;

use super::PluginRegistry;

pub struct LocalPluginLoader {
    plugins_dir: PathBuf,
}

impl LocalPluginLoader {
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self { plugins_dir }
    }

    pub fn scan(&self) -> Result<Vec<PathBuf>> {
        let mut found = Vec::new();
        if !self.plugins_dir.exists() {
            return Ok(found);
        }
        for entry in std::fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("klyron-plugin.json");
                let wasm_path = path.join(format!(
                    "{}.wasm",
                    path.file_name().unwrap().to_string_lossy()
                ));
                if manifest_path.exists() && wasm_path.exists() {
                    found.push(path);
                }
            }
        }
        Ok(found)
    }

    pub fn load_into_registry(&self, registry: &mut PluginRegistry) -> Result<Vec<String>> {
        let loaded = registry.load_all();
        info!("Loaded {} plugins from local filesystem", loaded.len());
        Ok(loaded)
    }

    pub fn find_plugin(&self, name: &str) -> Option<PathBuf> {
        let path = self.plugins_dir.join(name);
        if path.exists() && path.is_dir() {
            Some(path)
        } else {
            None
        }
    }

    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }
}
