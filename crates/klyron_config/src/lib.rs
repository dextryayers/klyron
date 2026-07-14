use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

// ── ConfigLayer ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigLayer {
    Defaults(KlyronConfig),
    File(PathBuf, KlyronConfig),
    Cli(KlyronConfig),
    Env(String, String),
}

impl ConfigLayer {
    #[inline]
    pub fn is_file(&self) -> bool {
        matches!(self, Self::File(..))
    }

    #[inline]
    pub fn source_name(&self) -> &str {
        match self {
            Self::Defaults(_) => "defaults",
            Self::File(p, _) => p.to_str().unwrap_or("unknown"),
            Self::Cli(_) => "cli",
            Self::Env(_, _) => "env",
        }
    }
}

// ── Core Config Types ─────────────────────────────────────────────────────

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

// ── Config File Names (ordered by priority) ───────────────────────────────

pub const CONFIG_FILE_NAMES: &[&str] = &[
    "klyron.toml",
    "klyron.config.ts",
    "klyron.config.js",
    "klyron.json",
    "package.json",
];

pub const DEFAULT_CONFIG: KlyronConfig = KlyronConfig {
    compiler: None,
    project: None,
    registries: None,
    plugins: None,
    telemetry: Some(true),
    server: None,
    build: None,
};

// ── Auto-discovery ────────────────────────────────────────────────────────

#[inline]
pub fn find_config(dir: &Path) -> Option<PathBuf> {
    let mut current = Some(dir);
    while let Some(d) = current {
        for name in CONFIG_FILE_NAMES {
            let candidate = d.join(name);
            if candidate.exists() {
                if name == &"package.json" {
                    if let Ok(content) = std::fs::read_to_string(&candidate) {
                        if content.contains("\"klyron\"") || content.contains("'klyron'") {
                            return Some(candidate);
                        }
                    }
                    continue;
                }
                return Some(candidate);
            }
        }
        current = d.parent();
    }
    None
}

// ── Load and Merge ────────────────────────────────────────────────────────

#[inline]
pub fn load_config(dir: &Path) -> anyhow::Result<KlyronConfig> {
    let mut merged = KlyronConfig::default();
    if let Some(path) = find_config(dir) {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Cannot read config {}: {e}", path.display()))?;
        let file_config = parse_config(&content, &path)?;
        deep_merge(&mut merged, file_config);
    }
    apply_env_overrides(&mut merged);
    Ok(merged)
}

#[inline]
pub fn get_config_value(dir: &Path, key: &str) -> Option<String> {
    let config = load_config(dir).ok()?;
    resolve_value(&config, key)
}

#[inline]
pub fn set_config_value(dir: &Path, key: &str, value: &str) -> anyhow::Result<()> {
    let path = find_config(dir).unwrap_or_else(|| dir.join("klyron.toml"));
    let content = if path.exists() { std::fs::read_to_string(&path)? } else { String::new() };
    let mut config = if path.exists() { parse_config(&content, &path).unwrap_or_default() } else { KlyronConfig::default() };
    apply_value(&mut config, key, value)?;
    let toml_str = toml::to_string_pretty(&config)
        .map_err(|e| anyhow::anyhow!("Failed to serialize config: {e}"))?;
    std::fs::write(&path, toml_str)?;
    Ok(())
}

// ── Config Builder ────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct ConfigBuilder {
    layers: Vec<ConfigLayer>,
}

impl ConfigBuilder {
    #[inline]
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    #[inline]
    pub fn with_defaults(mut self, config: KlyronConfig) -> Self {
        self.layers.push(ConfigLayer::Defaults(config));
        self
    }

    #[inline]
    pub fn with_file(mut self, path: PathBuf, config: KlyronConfig) -> Self {
        self.layers.push(ConfigLayer::File(path, config));
        self
    }

    #[inline]
    pub fn with_cli(mut self, config: KlyronConfig) -> Self {
        self.layers.push(ConfigLayer::Cli(config));
        self
    }

    #[inline]
    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.layers.push(ConfigLayer::Env(key, value));
        self
    }

    pub fn build(self) -> KlyronConfig {
        let mut merged = KlyronConfig::default();
        for layer in self.layers {
            match layer {
                ConfigLayer::Defaults(c)
                | ConfigLayer::File(_, c)
                | ConfigLayer::Cli(c) => deep_merge(&mut merged, c),
                ConfigLayer::Env(key, val) => {
                    if let Some(config_key) = key.strip_prefix("KLYRON_") {
                        let config_key = config_key.to_lowercase().replace('_', ".");
                        let _ = apply_value(&mut merged, &config_key, &val);
                    }
                }
            }
        }
        apply_env_overrides(&mut merged);
        merged
    }
}

// ── Internal Helpers ──────────────────────────────────────────────────────

fn parse_config(content: &str, path: &Path) -> anyhow::Result<KlyronConfig> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("toml") | None => {
            toml::from_str(content).map_err(|e| anyhow::anyhow!("TOML parse error in {}: {e}", path.display()))
        }
        Some("json") => {
            serde_json::from_str(content).map_err(|e| anyhow::anyhow!("JSON parse error in {}: {e}", path.display()))
        }
        Some("ts" | "js") => {
            // For .ts/.js configs, we fall back to TOML-like or JSON embedded; store as-is in project
            let mut config = KlyronConfig::default();
            config.project = Some(ProjectConfig {
                name: None,
                version: None,
                entry: None,
                out: None,
                r#type: Some(path.extension().unwrap().to_string_lossy().to_string()),
            });
            Ok(config)
        }
        _ => toml::from_str(content).map_err(|e| anyhow::anyhow!("Config parse error: {e}")),
    }
}

fn deep_merge(base: &mut KlyronConfig, overrides: KlyronConfig) {
    macro_rules! merge_opt {
        ($base:expr, $over:expr) => {
            if $over.is_some() {
                $base = $over;
            }
        };
    }
    merge_opt!(base.compiler, overrides.compiler);
    merge_opt!(base.project, overrides.project);
    merge_opt!(base.registries, overrides.registries);
    merge_opt!(base.plugins, overrides.plugins);
    merge_opt!(base.telemetry, overrides.telemetry);
    merge_opt!(base.server, overrides.server);
    merge_opt!(base.build, overrides.build);
}

fn apply_env_overrides(config: &mut KlyronConfig) {
    for (key, val) in std::env::vars() {
        if let Some(config_key) = key.strip_prefix("KLYRON_") {
            let config_key = config_key.to_lowercase().replace('_', ".");
            let _ = apply_value(config, &config_key, &val);
        }
    }
}

fn resolve_value(config: &KlyronConfig, key: &str) -> Option<String> {
    match key {
        "name" => config.project.as_ref()?.name.clone(),
        "version" => config.project.as_ref()?.version.clone(),
        "entry" => config.project.as_ref()?.entry.clone(),
        "out" => config.project.as_ref()?.out.clone(),
        "telemetry" => config.telemetry.map(|v| v.to_string()),
        "compiler.target" => config.compiler.as_ref()?.target.clone(),
        "compiler.minify" => config.compiler.as_ref()?.minify.map(|v| v.to_string()),
        "compiler.sourcemap" => config.compiler.as_ref()?.sourcemap.map(|v| v.to_string()),
        "server.host" => config.server.as_ref()?.host.clone(),
        "server.port" => config.server.as_ref()?.port.map(|v| v.to_string()),
        _ => None,
    }
}

fn apply_value(config: &mut KlyronConfig, key: &str, value: &str) -> anyhow::Result<()> {
    match key {
        "name" => set_project(config, |p| p.name = Some(value.to_string())),
        "version" => set_project(config, |p| p.version = Some(value.to_string())),
        "entry" => set_project(config, |p| p.entry = Some(value.to_string())),
        "out" => set_project(config, |p| p.out = Some(value.to_string())),
        "telemetry" => config.telemetry = Some(value.parse::<bool>().map_err(|_| anyhow::anyhow!("Invalid bool: {value}"))?),
        "compiler.target" => set_compiler(config, |c| c.target = Some(value.to_string())),
        "compiler.minify" => {
            let v: bool = value.parse().map_err(|_| anyhow::anyhow!("Invalid bool: {value}"))?;
            set_compiler(config, |c| c.minify = Some(v));
        }
        "compiler.sourcemap" => {
            let v: bool = value.parse().map_err(|_| anyhow::anyhow!("Invalid bool: {value}"))?;
            set_compiler(config, |c| c.sourcemap = Some(v));
        }
        _ => anyhow::bail!("Unknown config key: {key}"),
    }
    Ok(())
}

#[inline]
fn set_project<F: FnOnce(&mut ProjectConfig)>(config: &mut KlyronConfig, f: F) {
    let p = config.project.get_or_insert_with(ProjectConfig::default);
    f(p);
}

#[inline]
fn set_compiler<F: FnOnce(&mut CompilerConfig)>(config: &mut KlyronConfig, f: F) {
    let c = config.compiler.get_or_insert_with(CompilerConfig::default);
    f(c);
}

// ── Validation ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidation {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ConfigValidation {
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    #[inline]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    #[inline]
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

fn is_valid_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 || parts.len() > 4 { return false; }
    parts.iter().all(|p| {
        let trimmed = p.trim();
        if trimmed.is_empty() { return false; }
        trimmed.chars().all(|c| c.is_ascii_digit())
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_config() {
        let dir = std::env::current_dir().unwrap();
        let result = find_config(&dir);
        // May or may not find config; just ensure no panic
        let _ = result;
    }

    #[test]
    fn test_deep_merge() {
        let mut base = KlyronConfig::default();
        let over = KlyronConfig {
            telemetry: Some(false),
            project: Some(ProjectConfig {
                name: Some("test".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        deep_merge(&mut base, over);
        assert_eq!(base.telemetry, Some(false));
        assert_eq!(base.project.unwrap().name.unwrap(), "test");
    }

    #[test]
    fn test_env_override() {
        unsafe { std::env::set_var("KLYRON_TELEMETRY", "false"); }
        let mut config = KlyronConfig::default();
        config.telemetry = Some(true);
        apply_env_overrides(&mut config);
        assert_eq!(config.telemetry, Some(false));
        unsafe { std::env::remove_var("KLYRON_TELEMETRY"); }
    }

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
    fn test_validation() {
        let config = KlyronConfig {
            project: Some(ProjectConfig {
                name: Some("".into()),
                version: Some("not-semver".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let validation = validate_config(&config);
        assert!(!validation.errors.is_empty());
    }

    #[test]
    fn test_get_set_value() {
        let tmp = std::env::temp_dir().join("klyron_test_config");
        let _ = std::fs::create_dir_all(&tmp);
        let config_path = tmp.join("klyron.toml");
        std::fs::write(&config_path, "").unwrap();

        let result = set_config_value(&tmp, "name", "myapp");
        assert!(result.is_ok());

        let val = get_config_value(&tmp, "name");
        assert_eq!(val, Some("myapp".into()));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
