use serde::{Deserialize, Serialize};

/// Authentication method for registry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistryAuth {
    None,
    Token,
    Basic { username: String, password: String },
    Bearer { token: String },
}

impl Default for RegistryAuth {
    fn default() -> Self {
        Self::None
    }
}

/// Registry type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistryType {
    Npm,
    Github,
    Gitlab,
    Gitee,
    Unpkg,
    Jsr,
    Custom(String),
}

impl Default for RegistryType {
    fn default() -> Self {
        Self::Npm
    }
}

impl RegistryType {
    pub fn default_url(&self) -> &str {
        match self {
            Self::Npm => "https://registry.npmjs.org",
            Self::Github => "https://npm.pkg.github.com",
            Self::Gitlab => "https://gitlab.com/api/v4/packages/npm",
            Self::Gitee => "https://packages.gitee.com/npm",
            Self::Unpkg => "https://unpkg.com",
            Self::Jsr => "https://npm.jsr.io",
            Self::Custom(url) => url,
        }
    }
}

/// Single registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub registry_type: RegistryType,
    pub url: String,
    #[serde(default)]
    pub auth: RegistryAuth,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default = "default_priority")]
    pub priority: u32,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub timeout_seconds: u32,
    #[serde(default = "default_retry")]
    pub max_retries: u32,
}

fn default_priority() -> u32 { 100 }
fn default_true() -> bool { true }
fn default_retry() -> u32 { 3 }

impl RegistryConfig {
    pub fn npm_default() -> Self {
        Self {
            name: "npmjs".into(),
            registry_type: RegistryType::Npm,
            url: "https://registry.npmjs.org".into(),
            auth: RegistryAuth::None,
            scopes: vec!["*".into()],
            priority: 100,
            enabled: true,
            timeout_seconds: 30,
            max_retries: 3,
        }
    }

    pub fn github(owner: &str) -> Self {
        Self {
            name: format!("github-{owner}"),
            registry_type: RegistryType::Github,
            url: format!("https://npm.pkg.github.com/{owner}"),
            auth: RegistryAuth::Bearer {
                token: std::env::var("GITHUB_TOKEN").unwrap_or_default(),
            },
            scopes: vec![format!("@{owner}")],
            priority: 50,
            enabled: true,
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}

/// Multi-registry manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryManager {
    pub registries: Vec<RegistryConfig>,
    pub fallback_registry: RegistryConfig,
}

impl Default for RegistryManager {
    fn default() -> Self {
        Self {
            registries: vec![RegistryConfig::npm_default()],
            fallback_registry: RegistryConfig::npm_default(),
        }
    }
}

impl RegistryManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_registry(&mut self, config: RegistryConfig) {
        self.registries.push(config);
        self.registries.sort_by_key(|r| r.priority);
    }

    pub fn get_registry_for_package(&self, package_name: &str) -> &RegistryConfig {
        for reg in &self.registries {
            if !reg.enabled {
                continue;
            }
            for scope in &reg.scopes {
                if scope == "*" {
                    return reg;
                }
                if scope == package_name || package_name.starts_with(scope) {
                    return reg;
                }
            }
        }
        &self.fallback_registry
    }

    pub fn resolve_url(&self, package_name: &str, version: &str) -> String {
        let reg = self.get_registry_for_package(package_name);
        let base = reg.url.trim_end_matches('/');
        let scope = if package_name.starts_with('@') {
            let parts: Vec<&str> = package_name.splitn(2, '/').collect();
            format!("@{}/{}", parts[0].trim_start_matches('@'), parts.get(1).unwrap_or(&""))
        } else {
            package_name.to_string()
        };
        format!("{base}/{scope}/-/{scope}-{version}.tgz")
    }
}

/// Load registries from ~/.klyron/config.json
pub fn load_registry_config(path: Option<&std::path::Path>) -> RegistryManager {
    let config_path = path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
            std::path::PathBuf::from(home).join(".klyron").join("config.json")
        });

    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(manager) = serde_json::from_str::<RegistryManager>(&content) {
                return manager;
            }
        }
    }

    RegistryManager::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_manager_default() {
        let manager = RegistryManager::new();
        assert_eq!(manager.registries.len(), 1);
        assert_eq!(manager.registries[0].name, "npmjs");
    }

    #[test]
    fn test_get_registry_for_package() {
        let mut manager = RegistryManager::new();
        manager.add_registry(RegistryConfig::github("myorg"));

        let pkg = manager.get_registry_for_package("@myorg/foo");
        assert_eq!(pkg.name, "github-myorg");

        let pkg = manager.get_registry_for_package("lodash");
        assert_eq!(pkg.name, "npmjs");
    }

    #[test]
    fn test_resolve_url() {
        let manager = RegistryManager::new();
        let url = manager.resolve_url("lodash", "4.17.21");
        assert!(url.contains("lodash"));
        assert!(url.contains("4.17.21"));
        assert!(url.starts_with("https://registry.npmjs.org"));
    }

    #[test]
    fn test_scoped_package_url() {
        let mut manager = RegistryManager::new();
        manager.add_registry(RegistryConfig {
            name: "myorg".into(),
            registry_type: RegistryType::Custom("https://npm.myorg.com".into()),
            url: "https://npm.myorg.com".into(),
            auth: RegistryAuth::None,
            scopes: vec!["@myorg".into()],
            priority: 10,
            enabled: true,
            timeout_seconds: 30,
            max_retries: 3,
        });

        let url = manager.resolve_url("@myorg/foo", "1.0.0");
        assert!(url.starts_with("https://npm.myorg.com"));
        assert!(url.contains("@myorg"));
    }

    #[test]
    fn test_registry_priority() {
        let mut manager = RegistryManager::new();
        let high_prio = RegistryConfig {
            name: "fast-mirror".into(),
            registry_type: RegistryType::Npm,
            url: "https://fast-npm.example.com".into(),
            auth: RegistryAuth::None,
            scopes: vec!["*".into()],
            priority: 1,
            enabled: true,
            timeout_seconds: 5,
            max_retries: 1,
        };
        manager.add_registry(high_prio);

        let pkg = manager.get_registry_for_package("lodash");
        assert_eq!(pkg.priority, 1); // priority 1 should be first
    }

    #[test]
    fn test_disabled_registry() {
        let mut manager = RegistryManager::new();
        manager.add_registry(RegistryConfig {
            name: "disabled".into(),
            registry_type: RegistryType::Npm,
            url: "https://disabled.example.com".into(),
            auth: RegistryAuth::None,
            scopes: vec!["*".into()],
            priority: 1,
            enabled: false,
            timeout_seconds: 30,
            max_retries: 3,
        });
        // Should pick npmjs (not disabled)
        let pkg = manager.get_registry_for_package("test");
        assert_eq!(pkg.name, "npmjs");
    }
}
