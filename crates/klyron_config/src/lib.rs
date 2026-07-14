use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct KlyronConfig {
    pub compiler: Option<CompilerConfig>,
    pub project: Option<ProjectConfig>,
    pub registries: Option<HashMap<String, RegistryConfig>>,
    pub plugins: Option<Vec<String>>,
    pub telemetry: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CompilerConfig {
    pub target: Option<String>,
    pub minify: Option<bool>,
    pub sourcemap: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: Option<String>,
    pub version: Option<String>,
    pub entry: Option<String>,
    pub out: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub url: String,
    pub auth_token: Option<String>,
}

pub const CONFIG_FILE: &str = "klyron.toml";

pub fn find_config(dir: &Path) -> Option<PathBuf> {
    let mut current = Some(dir);
    while let Some(d) = current {
        let candidate = d.join(CONFIG_FILE);
        if candidate.exists() { return Some(candidate); }
        current = d.parent();
    }
    None
}

pub fn load_config(dir: &Path) -> anyhow::Result<KlyronConfig> {
    let path = find_config(dir).ok_or_else(|| anyhow::anyhow!("No {} found", CONFIG_FILE))?;
    let content = std::fs::read_to_string(&path)?;
    let config: KlyronConfig = basic_parse(&content)?;
    Ok(config)
}

pub fn get_config_value(dir: &Path, key: &str) -> Option<String> {
    let config = load_config(dir).ok()?;
    match key {
        "name" => config.project.as_ref()?.name.clone(),
        "version" => config.project.as_ref()?.version.clone(),
        "entry" => config.project.as_ref()?.entry.clone(),
        "out" => config.project.as_ref()?.out.clone(),
        "telemetry" => config.telemetry.map(|v| v.to_string()),
        _ => None,
    }
}

pub fn set_config_value(dir: &Path, key: &str, value: &str) -> anyhow::Result<()> {
    let path = find_config(dir).unwrap_or_else(|| dir.join(CONFIG_FILE));
    let content = if path.exists() { std::fs::read_to_string(&path)? } else { String::new() };
    let mut config: KlyronConfig = basic_parse(&content).unwrap_or_default();

    match key {
        "name" => { config.project.get_or_insert_with(Default::default).name = Some(value.to_string()); }
        "version" => { config.project.get_or_insert_with(Default::default).version = Some(value.to_string()); }
        "entry" => { config.project.get_or_insert_with(Default::default).entry = Some(value.to_string()); }
        "out" => { config.project.get_or_insert_with(Default::default).out = Some(value.to_string()); }
        "telemetry" => { config.telemetry = Some(value.parse().unwrap_or(false)); }
        _ => anyhow::bail!("Unknown config key: {key}"),
    }

    let toml_str = basic_stringify(&config);
    std::fs::write(&path, toml_str)?;
    Ok(())
}

fn basic_parse(content: &str) -> anyhow::Result<KlyronConfig> {
    let mut config = KlyronConfig::default();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() { continue; }
        if let Some((k, v)) = line.split_once('=') {
            let k = k.trim().to_string();
            let v = v.trim().trim_matches('"').to_string();
            match k.as_str() {
                "name" => { config.project.get_or_insert_with(Default::default).name = Some(v); }
                "version" => { config.project.get_or_insert_with(Default::default).version = Some(v); }
                "telemetry" => { config.telemetry = Some(v.parse().unwrap_or(false)); }
                _ => {}
            }
        }
    }
    Ok(config)
}

fn basic_stringify(config: &KlyronConfig) -> String {
    let mut out = String::from("# Klyron configuration\n\n");
    if let Some(ref project) = config.project {
        out.push_str("[project]\n");
        if let Some(ref name) = project.name { out.push_str(&format!("name = \"{name}\"\n")); }
        if let Some(ref version) = project.version { out.push_str(&format!("version = \"{version}\"\n")); }
    }
    if let Some(telemetry) = config.telemetry {
        out.push_str(&format!("\ntelemetry = {telemetry}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basic_parse() {
        let content = "name = \"test\"\nversion = \"1.0.0\"\n";
        let config = basic_parse(content).unwrap();
        assert_eq!(config.project.unwrap().name.unwrap(), "test");
    }
}
