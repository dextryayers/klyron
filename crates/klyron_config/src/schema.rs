use serde::{Deserialize, Serialize};

use crate::KlyronConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidation {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ConfigValidation {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn into_result(self) -> anyhow::Result<()> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            anyhow::bail!("Config errors:\n  {}", self.errors.join("\n  "))
        }
    }
}

pub fn validate_config(config: &KlyronConfig) -> ConfigValidation {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if let Some(ref project) = config.project {
        if let Some(ref name) = project.name {
            if name.trim().is_empty() {
                errors.push("project.name must not be empty".into());
            }
            if name.contains(' ') {
                warnings.push("project.name should not contain spaces".into());
            }
        }
        if let Some(ref version) = project.version {
            if !is_valid_semver(version) {
                errors.push(format!("project.version '{version}' is not valid semver"));
            }
        }
    }

    if let Some(ref compiler) = config.compiler {
        if let Some(ref target) = compiler.target {
            if target.is_empty() {
                warnings.push("compiler.target is empty, using default".into());
            }
        }
    }

    if let Some(ref plugins) = config.plugins {
        for (i, plugin) in plugins.iter().enumerate() {
            if plugin.trim().is_empty() {
                warnings.push(format!("plugins[{}] is empty", i));
            }
        }
    }

    ConfigValidation { errors, warnings }
}

pub fn validate_schema(config: &KlyronConfig) -> ConfigValidation {
    let mut result = validate_config(config);

    if let Some(ref server) = config.server {
        if let Some(port) = server.port {
            if port == 0 {
                result.errors.push("server.port must not be 0".into());
            }
            if port > 65535 {
                result.errors.push("server.port must be <= 65535".into());
            }
        }
        if let Some(ref host) = server.host {
            if host.is_empty() {
                result.warnings.push("server.host is empty".into());
            }
        }
    }

    if let Some(ref registries) = config.registries {
        for (name, reg) in registries {
            if reg.url.is_empty() {
                result.errors.push(format!("registries.{name}.url must not be empty"));
            }
            if !reg.url.starts_with("http") {
                result.warnings.push(format!("registries.{name}.url should start with http"));
            }
        }
    }

    result
}

fn is_valid_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 || parts.len() > 4 {
        return false;
    }
    parts.iter().all(|p| {
        let trimmed = p.trim();
        if trimmed.is_empty() {
            return false;
        }
        trimmed.chars().all(|c| c.is_ascii_digit())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompilerConfig, ProjectConfig, RegistryConfig, ServerConfig};

    #[test]
    fn test_validation_valid_config() {
        let config = KlyronConfig {
            project: Some(ProjectConfig {
                name: Some("valid-app".into()),
                version: Some("1.2.3".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let v = validate_config(&config);
        assert!(v.is_valid());
        assert!(!v.has_warnings());
        assert!(v.into_result().is_ok());
    }

    #[test]
    fn test_validation_empty_name() {
        let config = KlyronConfig {
            project: Some(ProjectConfig {
                name: Some("".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let v = validate_config(&config);
        assert!(!v.errors.is_empty());
        assert!(v.errors.iter().any(|e| e.contains("empty")));
    }

    #[test]
    fn test_validation_space_name_warning() {
        let config = KlyronConfig {
            project: Some(ProjectConfig {
                name: Some("my app".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let v = validate_config(&config);
        assert!(v.has_warnings());
        assert!(v.warnings.iter().any(|w| w.contains("spaces")));
    }

    #[test]
    fn test_validation_invalid_semver() {
        let config = KlyronConfig {
            project: Some(ProjectConfig {
                version: Some("not-semver".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let v = validate_config(&config);
        assert!(!v.errors.is_empty());
        assert!(v.errors.iter().any(|e| e.contains("semver")));
    }

    #[test]
    fn test_validation_empty_plugin_warning() {
        let config = KlyronConfig {
            plugins: Some(vec!["valid-plugin".into(), "".into()]),
            ..Default::default()
        };
        let v = validate_config(&config);
        assert!(v.has_warnings());
    }

    #[test]
    fn test_is_valid_semver() {
        assert!(is_valid_semver("1.2.3"));
        assert!(is_valid_semver("0.0.1"));
        assert!(is_valid_semver("10.20.30.40"));
        assert!(!is_valid_semver(""));
        assert!(!is_valid_semver("1"));
        assert!(!is_valid_semver("1.2.3.4.5"));
        assert!(!is_valid_semver("1.2.three"));
        assert!(!is_valid_semver(".1.2"));
    }

    #[test]
    fn test_config_validation_struct() {
        let v = ConfigValidation { errors: vec![], warnings: vec![] };
        assert!(v.is_valid());
        assert!(!v.has_warnings());
        assert!(v.clone().into_result().is_ok());

        let v2 = ConfigValidation {
            errors: vec!["some error".into()],
            warnings: vec!["warning".into()],
        };
        assert!(!v2.is_valid());
        assert!(v2.has_warnings());
        assert!(v2.into_result().is_err());
    }

    #[test]
    fn test_validate_schema_server_port() {
        let config = KlyronConfig {
            server: Some(ServerConfig {
                port: Some(0),
                ..Default::default()
            }),
            ..Default::default()
        };
        let v = validate_schema(&config);
        assert!(v.errors.iter().any(|e| e.contains("port")));

        let config2 = KlyronConfig {
            server: Some(ServerConfig {
                port: Some(70000),
                ..Default::default()
            }),
            ..Default::default()
        };
        let v2 = validate_schema(&config2);
        assert!(v2.errors.iter().any(|e| e.contains("65535")));
    }

    #[test]
    fn test_validate_schema_registries() {
        let config = KlyronConfig {
            registries: Some(HashMap::from([
                ("default".into(), RegistryConfig { url: "".into(), auth_token: None }),
            ])),
            ..Default::default()
        };
        let v = validate_schema(&config);
        assert!(v.errors.iter().any(|e| e.contains("url")));
    }

    #[test]
    fn test_compiler_target_empty_warning() {
        let config = KlyronConfig {
            compiler: Some(CompilerConfig {
                target: Some("".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let v = validate_config(&config);
        assert!(v.has_warnings());
        assert!(v.warnings.iter().any(|w| w.contains("compiler.target")));
    }

    use std::collections::HashMap;
}
