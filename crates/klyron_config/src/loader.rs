use std::path::{Path, PathBuf};

use crate::{CompilerConfig, KlyronConfig, ProjectConfig};

pub const CONFIG_FILE_NAMES: &[&str] = &[
    "klyron.toml",
    "klyron.config.ts",
    "klyron.config.js",
    "klyron.json",
    "klyron.yaml",
    "klyron.yml",
    "package.json",
];

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

pub fn get_config_value(dir: &Path, key: &str) -> Option<String> {
    let config = load_config(dir).ok()?;
    resolve_value(&config, key)
}

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

pub fn parse_config(content: &str, path: &Path) -> anyhow::Result<KlyronConfig> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("toml") | None => {
            toml::from_str(content).map_err(|e| anyhow::anyhow!("TOML parse error in {}: {e}", path.display()))
        }
        Some("json") => {
            serde_json::from_str(content).map_err(|e| anyhow::anyhow!("JSON parse error in {}: {e}", path.display()))
        }
        Some("yaml" | "yml") => {
            serde_yaml::from_str(content).map_err(|e| anyhow::anyhow!("YAML parse error in {}: {e}", path.display()))
        }
        Some("ts" | "js") => {
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

pub fn deep_merge(base: &mut KlyronConfig, overrides: KlyronConfig) {
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

pub fn apply_env_overrides(config: &mut KlyronConfig) {
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

fn set_project<F: FnOnce(&mut ProjectConfig)>(config: &mut KlyronConfig, f: F) {
    let p = config.project.get_or_insert_with(ProjectConfig::default);
    f(p);
}

fn set_compiler<F: FnOnce(&mut CompilerConfig)>(config: &mut KlyronConfig, f: F) {
    let c = config.compiler.get_or_insert_with(CompilerConfig::default);
    f(c);
}

pub fn set_config_value_from(config: &mut KlyronConfig, key: &str, value: &str) -> anyhow::Result<()> {
    apply_value(config, key, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_find_config() {
        let dir = std::env::current_dir().unwrap();
        let _result = find_config(&dir);
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
        struct Guard(String, Option<String>);
        impl Drop for Guard {
            fn drop(&mut self) {
                match &self.1 {
                    Some(v) => unsafe { std::env::set_var(&self.0, v); },
                    None => unsafe { std::env::remove_var(&self.0); },
                }
            }
        }
        let prev = std::env::var("KLYRON_TELEMETRY").ok();
        let _guard = Guard("KLYRON_TELEMETRY".into(), prev);
        unsafe { std::env::set_var("KLYRON_TELEMETRY", "false"); }
        let mut config = KlyronConfig::default();
        config.telemetry = Some(true);
        apply_env_overrides(&mut config);
        assert_eq!(config.telemetry, Some(false));
    }

    #[test]
    fn test_get_set_value() {
        let tmp = std::env::temp_dir().join("klyron_test_loader");
        let _ = fs::create_dir_all(&tmp);
        let config_path = tmp.join("klyron.toml");
        fs::write(&config_path, "").unwrap();

        let result = set_config_value(&tmp, "name", "myapp");
        assert!(result.is_ok());

        let val = get_config_value(&tmp, "name");
        assert_eq!(val, Some("myapp".into()));
    }

    #[test]
    fn test_parse_toml_config() {
        let toml_str = r#"
[project]
name = "test-app"
version = "1.0.0"

[compiler]
target = "wasm"
minify = true
"#;
        let config = parse_config(toml_str, Path::new("klyron.toml")).unwrap();
        assert_eq!(config.project.as_ref().unwrap().name.as_ref().unwrap(), "test-app");
        assert_eq!(config.project.as_ref().unwrap().version.as_ref().unwrap(), "1.0.0");
        assert_eq!(config.compiler.as_ref().unwrap().target.as_ref().unwrap(), "wasm");
        assert_eq!(config.compiler.as_ref().unwrap().minify, Some(true));
    }

    #[test]
    fn test_parse_json_config() {
        let json_str = r#"{
            "project": { "name": "json-app", "version": "2.0.0" },
            "telemetry": false
        }"#;
        let config = parse_config(json_str, Path::new("klyron.json")).unwrap();
        assert_eq!(config.project.as_ref().unwrap().name.as_ref().unwrap(), "json-app");
        assert_eq!(config.project.as_ref().unwrap().version.as_ref().unwrap(), "2.0.0");
        assert_eq!(config.telemetry, Some(false));
    }

    #[test]
    fn test_parse_yaml_config() {
        let yaml_str = r#"
project:
  name: yaml-app
  version: 3.0.0
telemetry: true
"#;
        let config = parse_config(yaml_str, Path::new("klyron.yaml")).unwrap();
        assert_eq!(config.project.as_ref().unwrap().name.as_ref().unwrap(), "yaml-app");
        assert_eq!(config.project.as_ref().unwrap().version.as_ref().unwrap(), "3.0.0");
        assert_eq!(config.telemetry, Some(true));
    }

    #[test]
    fn test_parse_js_config() {
        let config = parse_config("", Path::new("klyron.config.js")).unwrap();
        assert_eq!(config.project.as_ref().unwrap().r#type.as_ref().unwrap(), "js");
    }

    #[test]
    fn test_parse_ts_config() {
        let config = parse_config("", Path::new("klyron.config.ts")).unwrap();
        assert_eq!(config.project.as_ref().unwrap().r#type.as_ref().unwrap(), "ts");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = KlyronConfig {
            telemetry: Some(true),
            project: Some(ProjectConfig {
                name: Some("roundtrip".into()),
                version: Some("3.0.0".into()),
                ..Default::default()
            }),
            compiler: Some(CompilerConfig {
                target: Some("x86_64".into()),
                minify: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: KlyronConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.telemetry, Some(true));
        assert_eq!(deserialized.project.unwrap().name.unwrap(), "roundtrip");
    }

    #[test]
    fn test_resolve_value_nested() {
        let config = KlyronConfig {
            project: Some(ProjectConfig {
                name: Some("myapp".into()),
                version: Some("1.2.3".into()),
                entry: Some("src/main.ts".into()),
                ..Default::default()
            }),
            compiler: Some(CompilerConfig {
                target: Some("wasm".into()),
                minify: Some(true),
                ..Default::default()
            }),
            server: Some(ServerConfig {
                host: Some("0.0.0.0".into()),
                port: Some(3000),
                ..Default::default()
            }),
            telemetry: Some(false),
            ..Default::default()
        };
        assert_eq!(resolve_value(&config, "name"), Some("myapp".into()));
        assert_eq!(resolve_value(&config, "version"), Some("1.2.3".into()));
        assert_eq!(resolve_value(&config, "entry"), Some("src/main.ts".into()));
        assert_eq!(resolve_value(&config, "telemetry"), Some("false".into()));
        assert_eq!(resolve_value(&config, "compiler.target"), Some("wasm".into()));
        assert_eq!(resolve_value(&config, "compiler.minify"), Some("true".into()));
        assert_eq!(resolve_value(&config, "server.host"), Some("0.0.0.0".into()));
        assert_eq!(resolve_value(&config, "server.port"), Some("3000".into()));
        assert_eq!(resolve_value(&config, "nonexistent"), None);
    }

    #[test]
    fn test_apply_value() {
        let mut config = KlyronConfig::default();
        apply_value(&mut config, "name", "test-app").unwrap();
        assert_eq!(config.project.as_ref().unwrap().name.as_ref().unwrap(), "test-app");

        apply_value(&mut config, "telemetry", "true").unwrap();
        assert_eq!(config.telemetry, Some(true));

        apply_value(&mut config, "compiler.target", "arm").unwrap();
        assert_eq!(config.compiler.as_ref().unwrap().target.as_ref().unwrap(), "arm");
    }

    #[test]
    fn test_apply_value_errors() {
        let mut config = KlyronConfig::default();
        assert!(apply_value(&mut config, "unknown.key", "val").is_err());
        assert!(apply_value(&mut config, "telemetry", "not-a-bool").is_err());
        assert!(apply_value(&mut config, "compiler.minify", "not-bool").is_err());
    }
}
