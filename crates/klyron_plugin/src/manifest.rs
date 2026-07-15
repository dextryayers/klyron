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
