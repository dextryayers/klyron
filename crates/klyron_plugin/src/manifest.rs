use serde::{Deserialize, Serialize};

pub const KLYRON_API_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Option<Vec<String>>,
    pub license: Option<String>,
    pub klyron_api: Option<String>,
    pub permissions: Vec<String>,
    pub dependencies: Option<Vec<PluginDependency>>,
    pub hooks: Option<Vec<String>>,
    pub sandbox: Option<SandboxConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version: String,
    pub optional: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCompat {
    pub min_version: String,
    pub max_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub max_memory_bytes: Option<u64>,
    pub max_fuel: Option<u64>,
    pub max_cpu_ms: Option<u64>,
    pub allowed_domains: Option<Vec<String>>,
    pub allowed_paths: Option<Vec<String>>,
    pub allowed_env: Option<Vec<String>>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(64 * 1024 * 1024),
            max_fuel: Some(1_000_000),
            max_cpu_ms: Some(5000),
            allowed_domains: None,
            allowed_paths: None,
            allowed_env: None,
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookPhase {
    OnBeforeInstall,
    OnAfterInstall,
    OnBeforeBuild,
    OnAfterBuild,
    OnBeforeServe,
    OnAfterServe,
    OnBeforeTest,
    OnAfterTest,
}

impl HookPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookPhase::OnBeforeInstall => "on_before_install",
            HookPhase::OnAfterInstall => "on_after_install",
            HookPhase::OnBeforeBuild => "on_before_build",
            HookPhase::OnAfterBuild => "on_after_build",
            HookPhase::OnBeforeServe => "on_before_serve",
            HookPhase::OnAfterServe => "on_after_serve",
            HookPhase::OnBeforeTest => "on_before_test",
            HookPhase::OnAfterTest => "on_after_test",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "on_before_install" | "onBeforeInstall" => Some(HookPhase::OnBeforeInstall),
            "on_after_install" | "onAfterInstall" => Some(HookPhase::OnAfterInstall),
            "on_before_build" | "onBeforeBuild" => Some(HookPhase::OnBeforeBuild),
            "on_after_build" | "onAfterBuild" => Some(HookPhase::OnAfterBuild),
            "on_before_serve" | "onBeforeServe" => Some(HookPhase::OnBeforeServe),
            "on_after_serve" | "onAfterServe" => Some(HookPhase::OnAfterServe),
            "on_before_test" | "onBeforeTest" => Some(HookPhase::OnBeforeTest),
            "on_after_test" | "onAfterTest" => Some(HookPhase::OnAfterTest),
            _ => None,
        }
    }

    pub fn all() -> Vec<HookPhase> {
        vec![
            HookPhase::OnBeforeInstall,
            HookPhase::OnAfterInstall,
            HookPhase::OnBeforeBuild,
            HookPhase::OnAfterBuild,
            HookPhase::OnBeforeServe,
            HookPhase::OnAfterServe,
            HookPhase::OnBeforeTest,
            HookPhase::OnAfterTest,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub manifest: PluginManifest,
    pub enabled: bool,
    pub install_path: String,
    pub installed_at: String,
    pub wasm_hash: String,
    pub size_bytes: u64,
    pub compat: PluginCompat,
}

impl PluginInfo {
    pub fn is_compatible(&self) -> bool {
        let api = semver_version(KLYRON_API_VERSION);
        let min = semver_version(&self.compat.min_version);
        let max = semver_version(&self.compat.max_version);
        api >= min && api <= max
    }
}

fn semver_version(v: &str) -> Vec<u32> {
    v.split('.')
        .filter_map(|p| p.parse::<u32>().ok())
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMarketplaceEntry {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub rating: f64,
    pub tags: Vec<String>,
}

pub fn default_compat() -> PluginCompat {
    PluginCompat {
        min_version: "1.0.0".to_string(),
        max_version: "1.0.0".to_string(),
    }
}

#[derive(Debug, Clone)]
pub struct SandboxTestReport {
    pub plugin_name: String,
    pub hook_calls: Vec<String>,
    pub memory_ops: Vec<MemoryOp>,
    pub total_fuel_consumed: u64,
    pub execution_time_ms: u64,
    pub timed_out: bool,
    pub passed: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MemoryOp {
    pub op_type: String,
    pub offset: u64,
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let m = PluginManifest {
            name: "test-plugin".into(),
            version: "1.0.0".into(),
            description: Some("A test plugin".into()),
            authors: Some(vec!["author".into()]),
            license: Some("MIT".into()),
            klyron_api: Some("1.0.0".into()),
            permissions: vec!["stdio".into()],
            dependencies: Some(vec![PluginDependency {
                name: "dep1".into(),
                version: "0.2.0".into(),
                optional: Some(false),
            }]),
            hooks: Some(vec!["on_before_build".into()]),
            sandbox: Some(SandboxConfig::default()),
        };
        assert_eq!(m.name, "test-plugin");
        assert_eq!(m.version, "1.0.0");
        assert_eq!(m.permissions, vec!["stdio"]);
        assert!(m.hooks.is_some());
        assert!(m.sandbox.is_some());
    }

    #[test]
    fn test_manifest_json_roundtrip() {
        let m = PluginManifest {
            name: "json-test".into(),
            version: "2.0.0".into(),
            description: None,
            authors: None,
            license: None,
            klyron_api: None,
            permissions: vec![],
            dependencies: None,
            hooks: None,
            sandbox: None,
        };
        let json = serde_json::to_string(&m).unwrap();
        let deserialized: PluginManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "json-test");
        assert_eq!(deserialized.version, "2.0.0");
    }

    #[test]
    fn test_dependency_creation() {
        let dep = PluginDependency {
            name: "my-dep".into(),
            version: "0.5.0".into(),
            optional: Some(true),
        };
        assert_eq!(dep.name, "my-dep");
        assert_eq!(dep.version, "0.5.0");
        assert_eq!(dep.optional, Some(true));
    }

    #[test]
    fn test_hook_phase_as_str() {
        assert_eq!(HookPhase::OnBeforeInstall.as_str(), "on_before_install");
        assert_eq!(HookPhase::OnAfterInstall.as_str(), "on_after_install");
        assert_eq!(HookPhase::OnBeforeBuild.as_str(), "on_before_build");
        assert_eq!(HookPhase::OnAfterBuild.as_str(), "on_after_build");
        assert_eq!(HookPhase::OnBeforeServe.as_str(), "on_before_serve");
        assert_eq!(HookPhase::OnAfterServe.as_str(), "on_after_serve");
        assert_eq!(HookPhase::OnBeforeTest.as_str(), "on_before_test");
        assert_eq!(HookPhase::OnAfterTest.as_str(), "on_after_test");
    }

    #[test]
    fn test_hook_phase_from_str_snake() {
        assert_eq!(HookPhase::from_str("on_before_install"), Some(HookPhase::OnBeforeInstall));
        assert_eq!(HookPhase::from_str("on_after_install"), Some(HookPhase::OnAfterInstall));
        assert_eq!(HookPhase::from_str("on_before_build"), Some(HookPhase::OnBeforeBuild));
        assert_eq!(HookPhase::from_str("on_after_build"), Some(HookPhase::OnAfterBuild));
        assert_eq!(HookPhase::from_str("on_before_serve"), Some(HookPhase::OnBeforeServe));
        assert_eq!(HookPhase::from_str("on_after_serve"), Some(HookPhase::OnAfterServe));
        assert_eq!(HookPhase::from_str("on_before_test"), Some(HookPhase::OnBeforeTest));
        assert_eq!(HookPhase::from_str("on_after_test"), Some(HookPhase::OnAfterTest));
    }

    #[test]
    fn test_hook_phase_from_str_camel() {
        assert_eq!(HookPhase::from_str("onBeforeInstall"), Some(HookPhase::OnBeforeInstall));
        assert_eq!(HookPhase::from_str("onAfterInstall"), Some(HookPhase::OnAfterInstall));
        assert_eq!(HookPhase::from_str("onBeforeBuild"), Some(HookPhase::OnBeforeBuild));
        assert_eq!(HookPhase::from_str("onAfterBuild"), Some(HookPhase::OnAfterBuild));
        assert_eq!(HookPhase::from_str("onBeforeServe"), Some(HookPhase::OnBeforeServe));
        assert_eq!(HookPhase::from_str("onAfterServe"), Some(HookPhase::OnAfterServe));
        assert_eq!(HookPhase::from_str("onBeforeTest"), Some(HookPhase::OnBeforeTest));
        assert_eq!(HookPhase::from_str("onAfterTest"), Some(HookPhase::OnAfterTest));
    }

    #[test]
    fn test_hook_phase_from_str_invalid() {
        assert_eq!(HookPhase::from_str("invalid"), None);
        assert_eq!(HookPhase::from_str(""), None);
        assert_eq!(HookPhase::from_str("on_before_foo"), None);
    }

    #[test]
    fn test_hook_phase_all() {
        let all = HookPhase::all();
        assert_eq!(all.len(), 8);
        assert!(all.contains(&HookPhase::OnBeforeInstall));
        assert!(all.contains(&HookPhase::OnAfterInstall));
        assert!(all.contains(&HookPhase::OnBeforeBuild));
        assert!(all.contains(&HookPhase::OnAfterBuild));
        assert!(all.contains(&HookPhase::OnBeforeServe));
        assert!(all.contains(&HookPhase::OnAfterServe));
        assert!(all.contains(&HookPhase::OnBeforeTest));
        assert!(all.contains(&HookPhase::OnAfterTest));
    }

    #[test]
    fn test_hook_phase_roundtrip() {
        for phase in HookPhase::all() {
            let s = phase.as_str();
            let parsed = HookPhase::from_str(s);
            assert_eq!(parsed, Some(phase));
        }
    }

    #[test]
    fn test_plugin_info_compatible() {
        let manifest = PluginManifest {
            name: "compat-test".into(),
            version: "1.0.0".into(),
            description: None,
            authors: None,
            license: None,
            klyron_api: Some("1.0.0".into()),
            permissions: vec![],
            dependencies: None,
            hooks: None,
            sandbox: None,
        };
        let info = PluginInfo {
            manifest,
            enabled: true,
            install_path: "/tmp/plugin".into(),
            installed_at: "2024-01-01".into(),
            wasm_hash: "abc".into(),
            size_bytes: 100,
            compat: PluginCompat {
                min_version: "1.0.0".into(),
                max_version: "1.0.0".into(),
            },
        };
        assert!(info.is_compatible());
    }

    #[test]
    fn test_plugin_info_incompatible() {
        let manifest = PluginManifest {
            name: "incompat-test".into(),
            version: "1.0.0".into(),
            description: None,
            authors: None,
            license: None,
            klyron_api: Some("1.0.0".into()),
            permissions: vec![],
            dependencies: None,
            hooks: None,
            sandbox: None,
        };
        let info = PluginInfo {
            manifest,
            enabled: true,
            install_path: "/tmp/plugin".into(),
            installed_at: "2024-01-01".into(),
            wasm_hash: "abc".into(),
            size_bytes: 100,
            compat: PluginCompat {
                min_version: "2.0.0".into(),
                max_version: "3.0.0".into(),
            },
        };
        assert!(!info.is_compatible());
    }

    #[test]
    fn test_default_compat() {
        let compat = default_compat();
        assert_eq!(compat.min_version, "1.0.0");
        assert_eq!(compat.max_version, "1.0.0");
    }

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.max_memory_bytes, Some(64 * 1024 * 1024));
        assert_eq!(config.max_fuel, Some(1_000_000));
        assert_eq!(config.max_cpu_ms, Some(5000));
        assert!(config.allowed_domains.is_none());
        assert!(config.allowed_paths.is_none());
        assert!(config.allowed_env.is_none());
    }

    #[test]
    fn test_plugin_info_semver_edge() {
        let manifest = PluginManifest {
            name: "edge".into(),
            version: "0.5.0".into(),
            ..Default::default()
        };
        let info = PluginInfo {
            manifest,
            enabled: false,
            install_path: "/p".into(),
            installed_at: "now".into(),
            wasm_hash: "x".into(),
            size_bytes: 0,
            compat: PluginCompat {
                min_version: "0.5.0".into(),
                max_version: "2.0.0".into(),
            },
        };
        assert!(info.is_compatible());
    }

    #[test]
    fn test_manifest_default_impl() {
        let m = PluginManifest {
            name: "defaults".into(),
            version: "0.0.1".into(),
            ..Default::default()
        };
        assert_eq!(m.description, None);
        assert_eq!(m.authors, None);
        assert!(m.permissions.is_empty());
        assert_eq!(m.name, "defaults");
    }

    impl Default for PluginManifest {
        fn default() -> Self {
            Self {
                name: String::new(),
                version: "0.1.0".into(),
                description: None,
                authors: None,
                license: None,
                klyron_api: None,
                permissions: Vec::new(),
                dependencies: None,
                hooks: None,
                sandbox: None,
            }
        }
    }
}
