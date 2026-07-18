pub mod loader;
pub mod schema;
pub mod watch;

pub use loader::{find_config, load_config, get_config_value, set_config_value, parse_config, deep_merge, apply_env_overrides, CONFIG_FILE_NAMES};
pub use schema::{validate_config, validate_schema, ConfigValidation};
pub use watch::{ConfigWatcher, ConfigEvent};

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigLayer {
    Defaults(KlyronConfig),
    File(PathBuf, KlyronConfig),
    Cli(KlyronConfig),
    Env(String, String),
}

impl ConfigLayer {
    pub fn is_file(&self) -> bool {
        matches!(self, Self::File(..))
    }

    pub fn source_name(&self) -> &str {
        match self {
            Self::Defaults(_) => "defaults",
            Self::File(p, _) => p.to_str().unwrap_or("unknown"),
            Self::Cli(_) => "cli",
            Self::Env(_, _) => "env",
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct KlyronConfig {
    pub compiler: Option<CompilerConfig>,
    pub project: Option<ProjectConfig>,
    pub registries: Option<HashMap<String, RegistryConfig>>,
    pub plugins: Option<Vec<String>>,
    pub telemetry: Option<bool>,
    pub server: Option<ServerConfig>,
    pub build: Option<BuildConfig>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    pub target: Option<String>,
    pub minify: Option<bool>,
    pub sourcemap: Option<bool>,
    pub optimize: Option<bool>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: Option<String>,
    pub version: Option<String>,
    pub entry: Option<String>,
    pub out: Option<String>,
    pub r#type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub url: String,
    pub auth_token: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub dir: Option<PathBuf>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub out_dir: Option<PathBuf>,
    pub minify: Option<bool>,
    pub sourcemap: Option<bool>,
    pub target: Option<String>,
}

pub const DEFAULT_CONFIG: KlyronConfig = KlyronConfig {
    compiler: None,
    project: None,
    registries: None,
    plugins: None,
    telemetry: Some(true),
    server: None,
    build: None,
};

#[derive(Debug, Default)]
pub struct ConfigBuilder {
    layers: Vec<ConfigLayer>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn with_defaults(mut self, config: KlyronConfig) -> Self {
        self.layers.push(ConfigLayer::Defaults(config));
        self
    }

    pub fn with_file(mut self, path: PathBuf, config: KlyronConfig) -> Self {
        self.layers.push(ConfigLayer::File(path, config));
        self
    }

    pub fn with_cli(mut self, config: KlyronConfig) -> Self {
        self.layers.push(ConfigLayer::Cli(config));
        self
    }

    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.layers.push(ConfigLayer::Env(key, value));
        self
    }

    pub fn build(self) -> KlyronConfig {
        let mut merged = KlyronConfig::default();
        for layer in self.layers {
            match layer {
                ConfigLayer::Defaults(c) | ConfigLayer::File(_, c) | ConfigLayer::Cli(c) => {
                    loader::deep_merge(&mut merged, c);
                }
                ConfigLayer::Env(key, val) => {
                    if let Some(config_key) = key.strip_prefix("KLYRON_") {
                        let config_key = config_key.to_lowercase().replace('_', ".");
                        let _ = loader::set_config_value_from(&mut merged, &config_key, &val);
                    }
                }
            }
        }
        loader::apply_env_overrides(&mut merged);
        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .with_defaults(KlyronConfig {
                telemetry: Some(true),
                ..Default::default()
            })
            .with_cli(KlyronConfig {
                telemetry: Some(false),
                ..Default::default()
            })
            .build();
        assert_eq!(config.telemetry, Some(false));
    }

    #[test]
    fn test_config_builder_with_env() {
        let config = ConfigBuilder::new()
            .with_defaults(KlyronConfig {
                telemetry: Some(true),
                ..Default::default()
            })
            .with_env("KLYRON_TELEMETRY".into(), "false".into())
            .build();
        assert_eq!(config.telemetry, Some(false));
    }

    #[test]
    fn test_config_builder_env_no_prefix() {
        let config = ConfigBuilder::new()
            .with_env("OTHER_VAR".into(), "value".into())
            .build();
        assert_eq!(config.telemetry, None);
    }

    #[test]
    fn test_config_builder_ordering() {
        let config = ConfigBuilder::new()
            .with_defaults(KlyronConfig {
                telemetry: Some(true),
                ..Default::default()
            })
            .with_cli(KlyronConfig {
                telemetry: Some(false),
                ..Default::default()
            })
            .build();
        assert_eq!(config.telemetry, Some(false));
    }

    #[test]
    fn test_default_config_const() {
        assert_eq!(DEFAULT_CONFIG.telemetry, Some(true));
        assert!(DEFAULT_CONFIG.compiler.is_none());
        assert!(DEFAULT_CONFIG.project.is_none());
        assert!(DEFAULT_CONFIG.server.is_none());
        assert!(DEFAULT_CONFIG.build.is_none());
    }

    #[test]
    fn test_config_layer_source_name() {
        let def = ConfigLayer::Defaults(KlyronConfig::default());
        assert_eq!(def.source_name(), "defaults");

        let file = ConfigLayer::File(PathBuf::from("klyron.toml"), KlyronConfig::default());
        assert_eq!(file.source_name(), "klyron.toml");
        assert!(file.is_file());

        let cli = ConfigLayer::Cli(KlyronConfig::default());
        assert_eq!(cli.source_name(), "cli");

        let env = ConfigLayer::Env("KEY".into(), "VAL".into());
        assert_eq!(env.source_name(), "env");
    }

    #[test]
    fn test_deep_merge_all_fields() {
        let mut base = KlyronConfig::default();
        let over = KlyronConfig {
            telemetry: Some(false),
            compiler: Some(CompilerConfig {
                target: Some("wasm".into()),
                minify: Some(true),
                ..Default::default()
            }),
            build: Some(BuildConfig {
                minify: Some(true),
                ..Default::default()
            }),
            plugins: Some(vec!["plugin-a".into()]),
            ..Default::default()
        };
        deep_merge(&mut base, over);
        assert_eq!(base.telemetry, Some(false));
        assert_eq!(base.compiler.unwrap().target.unwrap(), "wasm");
    }
}
