use serde::{Deserialize, Serialize};

use crate::check::PermissionFlags;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub version: String,
    pub allowed: AllowedPolicies,
    pub denied: DeniedPolicies,
    pub settings: PolicySettings,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        PolicyConfig {
            version: "1.0".into(),
            allowed: AllowedPolicies::default(),
            denied: DeniedPolicies::default(),
            settings: PolicySettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedPolicies {
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
    #[serde(default)]
    pub env: Vec<String>,
    #[serde(default)]
    pub net: Vec<String>,
    #[serde(default)]
    pub run: Vec<String>,
    #[serde(default)]
    pub ffi: bool,
    #[serde(default)]
    pub all: bool,
}

impl Default for AllowedPolicies {
    fn default() -> Self {
        AllowedPolicies {
            read: vec![],
            write: vec![],
            env: vec![],
            net: vec![],
            run: vec![],
            ffi: false,
            all: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeniedPolicies {
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
    #[serde(default)]
    pub env: Vec<String>,
    #[serde(default)]
    pub net: Vec<String>,
    #[serde(default)]
    pub run: Vec<String>,
}

impl Default for DeniedPolicies {
    fn default() -> Self {
        DeniedPolicies {
            read: vec![],
            write: vec![],
            env: vec![],
            net: vec![],
            run: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySettings {
    #[serde(default)]
    pub prompt: bool,
    #[serde(default)]
    pub audit_log: bool,
    #[serde(default)]
    pub cache_ttl_seconds: u64,
}

impl Default for PolicySettings {
    fn default() -> Self {
        PolicySettings {
            prompt: false,
            audit_log: true,
            cache_ttl_seconds: 300,
        }
    }
}

impl PolicyConfig {
    pub fn from_file(path: &std::path::Path) -> Result<Self, PolicyError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PolicyError::IoError(e.to_string()))?;
        let config: PolicyConfig = serde_json::from_str(&content)
            .or_else(|_| {
                serde_yaml::from_str(&content)
                    .map_err(|e| PolicyError::ParseError(e.to_string()))
            })?;
        Ok(config)
    }

    pub fn to_flags(&self) -> PermissionFlags {
        PermissionFlags {
            allow_read: self.allowed.read.clone(),
            deny_read: self.denied.read.clone(),
            allow_write: self.allowed.write.clone(),
            deny_write: self.denied.write.clone(),
            allow_env: self.allowed.env.clone(),
            deny_env: self.denied.env.clone(),
            allow_net: self.allowed.net.clone(),
            deny_net: self.denied.net.clone(),
            allow_run: self.allowed.run.clone(),
            deny_run: self.denied.run.clone(),
            allow_ffi: self.allowed.ffi,
            allow_all: self.allowed.all,
            prompt: self.settings.prompt,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

pub fn parse_allow_flags(flags: &[String]) -> PermissionFlags {
    let mut pf = PermissionFlags::default();
    let mut iter = flags.iter().peekable();

    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--allow-read" => {
                if let Some(val) = iter.next_if(|s| !s.starts_with("--")) {
                    pf.allow_read.push(val.clone());
                }
            }
            "--allow-write" => {
                if let Some(val) = iter.next_if(|s| !s.starts_with("--")) {
                    pf.allow_write.push(val.clone());
                }
            }
            "--allow-env" => {
                if let Some(val) = iter.next_if(|s| !s.starts_with("--")) {
                    pf.allow_env.push(val.clone());
                }
            }
            "--allow-net" => {
                if let Some(val) = iter.next_if(|s| !s.starts_with("--")) {
                    pf.allow_net.push(val.clone());
                }
            }
            "--allow-run" => {
                if let Some(val) = iter.next_if(|s| !s.starts_with("--")) {
                    pf.allow_run.push(val.clone());
                }
            }
            "--allow-ffi" => pf.allow_ffi = true,
            "--allow-all" => pf.allow_all = true,
            "--deny-read" => {
                if let Some(val) = iter.next_if(|s| !s.starts_with("--")) {
                    pf.deny_read.push(val.clone());
                }
            }
            "--deny-write" => {
                if let Some(val) = iter.next_if(|s| !s.starts_with("--")) {
                    pf.deny_write.push(val.clone());
                }
            }
            "--deny-env" => {
                if let Some(val) = iter.next_if(|s| !s.starts_with("--")) {
                    pf.deny_env.push(val.clone());
                }
            }
            "--deny-net" => {
                if let Some(val) = iter.next_if(|s| !s.starts_with("--")) {
                    pf.deny_net.push(val.clone());
                }
            }
            "--deny-run" => {
                if let Some(val) = iter.next_if(|s| !s.starts_with("--")) {
                    pf.deny_run.push(val.clone());
                }
            }
            "--prompt" => pf.prompt = true,
            _ => {}
        }
    }
    pf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_config_default() {
        let config = PolicyConfig::default();
        assert_eq!(config.version, "1.0");
        assert!(!config.allowed.all);
    }

    #[test]
    fn test_policy_config_to_flags() {
        let mut config = PolicyConfig::default();
        config.allowed.read.push("/app/**".into());
        config.allowed.all = false;
        let flags = config.to_flags();
        assert!(flags.allow_read.contains(&"/app/**".to_string()));
    }

    #[test]
    fn test_parse_allow_flags_empty() {
        let flags = parse_allow_flags(&[]);
        assert!(!flags.allow_all);
    }

    #[test]
    fn test_parse_allow_flags_allow_all() {
        let flags = parse_allow_flags(&["--allow-all".into()]);
        assert!(flags.allow_all);
    }

    #[test]
    fn test_parse_allow_flags_with_values() {
        let flags = parse_allow_flags(&[
            "--allow-read".into(),
            "/app/**".into(),
            "--allow-write".into(),
            "/tmp/**".into(),
            "--allow-ffi".into(),
        ]);
        assert!(flags.allow_read.contains(&"/app/**".to_string()));
        assert!(flags.allow_write.contains(&"/tmp/**".to_string()));
        assert!(flags.allow_ffi);
    }

    #[test]
    fn test_parse_deny_flags() {
        let flags = parse_allow_flags(&[
            "--deny-read".into(),
            "/etc/**".into(),
            "--deny-net".into(),
            "*.evil.com".into(),
        ]);
        assert!(flags.deny_read.contains(&"/etc/**".to_string()));
        assert!(flags.deny_net.contains(&"*.evil.com".to_string()));
    }

    #[test]
    fn test_parse_prompt_flag() {
        let flags = parse_allow_flags(&["--prompt".into()]);
        assert!(flags.prompt);
    }

    #[test]
    fn test_policy_config_from_file_nonexistent() {
        let result = PolicyConfig::from_file(std::path::Path::new("/nonexistent/policy.json"));
        assert!(result.is_err());
    }
}
