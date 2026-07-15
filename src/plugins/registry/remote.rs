use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::info;

pub struct RemotePlugin {
    pub name: String,
    pub version: String,
    pub registry_url: String,
    pub download_url: String,
    pub integrity: String,
}

pub struct RemoteRegistry {
    registry_url: String,
    cache_dir: PathBuf,
}

impl RemoteRegistry {
    pub fn new(registry_url: &str, cache_dir: PathBuf) -> Self {
        Self {
            registry_url: registry_url.to_string(),
            cache_dir,
        }
    }

    pub fn search(&self, query: &str) -> Result<Vec<RemotePlugin>> {
        let url = format!("{}/api/v1/plugins/search?q={}", self.registry_url, urlencoding(query));
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        let resp = client.get(&url).send()?;
        let plugins: Vec<RemotePlugin> = resp.json()?;
        Ok(plugins)
    }

    pub fn get_plugin_info(&self, name: &str) -> Result<RemotePlugin> {
        let url = format!("{}/api/v1/plugins/{}", self.registry_url, name);
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        let resp = client.get(&url).send()?;
        let plugin: RemotePlugin = resp.json()?;
        Ok(plugin)
    }

    pub fn download_plugin(&self, plugin: &RemotePlugin, dest: &Path) -> Result<PathBuf> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()?;
        let resp = client.get(&plugin.download_url).send()?;
        let bytes = resp.bytes()?;

        std::fs::create_dir_all(dest)?;
        let wasm_path = dest.join(format!("{}.wasm", plugin.name));
        std::fs::write(&wasm_path, &bytes)?;

        info!("Downloaded plugin {} v{} to {:?}", plugin.name, plugin.version, wasm_path);
        Ok(wasm_path)
    }

    fn registry_url(&self) -> &str {
        &self.registry_url
    }
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

impl Default for RemoteRegistry {
    fn default() -> Self {
        Self {
            registry_url: "https://registry.klyron.dev".to_string(),
            cache_dir: PathBuf::from("/tmp/klyron-remote-plugins"),
        }
    }
}
