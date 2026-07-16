use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposerJson {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub project_type: Option<String>,
    pub require: Option<BTreeMap<String, String>>,
    #[serde(rename = "require-dev")]
    pub require_dev: Option<BTreeMap<String, String>>,
    pub autoload: Option<ComposerAutoload>,
    #[serde(rename = "autoload-dev")]
    pub autoload_dev: Option<ComposerAutoload>,
    pub scripts: Option<BTreeMap<String, serde_json::Value>>,
    pub extra: Option<serde_json::Value>,
    pub config: Option<serde_json::Value>,
    #[serde(rename = "minimum-stability")]
    pub minimum_stability: Option<String>,
    #[serde(rename = "prefer-stable")]
    pub prefer_stable: Option<bool>,
    pub repositories: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposerAutoload {
    #[serde(rename = "psr-4")]
    pub psr4: Option<BTreeMap<String, String>>,
    #[serde(rename = "psr-0")]
    pub psr0: Option<BTreeMap<String, String>>,
    pub classmap: Option<Vec<String>>,
    pub files: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposerLock {
    #[serde(rename = "_readme")]
    pub readme: Option<String>,
    #[serde(rename = "content-hash")]
    pub content_hash: Option<String>,
    pub packages: Option<Vec<ComposerLockPackage>>,
    #[serde(rename = "packages-dev")]
    pub packages_dev: Option<Vec<ComposerLockPackage>>,
    #[serde(rename = "platform-overrides")]
    pub platform_overrides: Option<BTreeMap<String, String>>,
    #[serde(rename = "plugin-api-version")]
    pub plugin_api_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposerLockPackage {
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub package_type: Option<String>,
    pub source: Option<ComposerSource>,
    pub dist: Option<ComposerDist>,
    pub autoload: Option<ComposerAutoload>,
    pub require: Option<BTreeMap<String, String>>,
    #[serde(rename = "require-dev")]
    pub require_dev: Option<BTreeMap<String, String>>,
    pub license: Option<Vec<String>>,
    pub authors: Option<Vec<serde_json::Value>>,
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub time: Option<String>,
    pub funding: Option<Vec<serde_json::Value>>,
    pub notification_url: Option<String>,
    pub extra: Option<serde_json::Value>,
    pub bin: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposerSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub url: String,
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposerDist {
    #[serde(rename = "type")]
    pub dist_type: String,
    pub url: String,
    pub reference: Option<String>,
    pub shasum: Option<String>,
}

impl ComposerJson {
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Cannot read composer.json: {e}"))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Invalid composer.json: {e}"))
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialize error: {e}"))
    }
}

impl ComposerLock {
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Cannot read composer.lock: {e}"))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Invalid composer.lock: {e}"))
    }

    pub fn find_package(&self, name: &str) -> Option<&ComposerLockPackage> {
        self.packages.as_ref()?.iter().find(|p| p.name == name)
    }

    pub fn all_packages(&self) -> Vec<&ComposerLockPackage> {
        let mut pkgs = Vec::new();
        if let Some(ref p) = self.packages {
            pkgs.extend(p.iter());
        }
        if let Some(ref p) = self.packages_dev {
            pkgs.extend(p.iter());
        }
        pkgs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_composer_json() {
        let json = r#"{
            "name": "laravel/laravel",
            "description": "Laravel Framework",
            "require": {
                "php": "^8.1",
                "laravel/framework": "^10.0"
            },
            "autoload": {
                "psr-4": {
                    "App\\": "app/"
                }
            }
        }"#;
        let composer: ComposerJson = serde_json::from_str(json).unwrap();
        assert_eq!(composer.name.as_deref(), Some("laravel/laravel"));
        assert!(composer.require.as_ref().unwrap().contains_key("php"));
    }

    #[test]
    fn test_parse_composer_lock() {
        let json = r#"{
            "_readme": "This file is locked",
            "content-hash": "abc123",
            "packages": [
                {
                    "name": "laravel/framework",
                    "version": "10.0.0",
                    "source": {
                        "type": "git",
                        "url": "https://github.com/laravel/framework",
                        "reference": "abc123"
                    },
                    "dist": {
                        "type": "zip",
                        "url": "https://api.github.com/repos/laravel/framework/zipball/abc123",
                        "reference": "abc123",
                        "shasum": ""
                    },
                    "require": {
                        "php": "^8.1",
                        "illuminate/support": "^10.0"
                    }
                }
            ]
        }"#;
        let lock: ComposerLock = serde_json::from_str(json).unwrap();
        assert_eq!(lock.packages.as_ref().unwrap().len(), 1);
        assert_eq!(lock.packages.as_ref().unwrap()[0].name, "laravel/framework");
    }
}
