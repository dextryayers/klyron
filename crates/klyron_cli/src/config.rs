use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum ConfigSource {
    CliArg,
    EnvVar,
    ProjectFile,
    UserFile,
    GlobalDefault,
}

#[derive(Debug, Clone)]
pub enum ConfigValue {
    String(String),
    Number(f64),
    Bool(bool),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
    Null,
}

impl ConfigValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Bool(b) => Some(*b),
            ConfigValue::String(s) => s.parse::<bool>().ok(),
            ConfigValue::Number(n) => Some(*n != 0.0),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ConfigValue::Number(n) => Some(*n),
            ConfigValue::String(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<ConfigValue>> {
        match self {
            ConfigValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, ConfigValue>> {
        match self {
            ConfigValue::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, ConfigValue::Null)
    }
}

impl From<String> for ConfigValue {
    fn from(s: String) -> Self { ConfigValue::String(s) }
}

impl From<&str> for ConfigValue {
    fn from(s: &str) -> Self { ConfigValue::String(s.to_string()) }
}

impl From<bool> for ConfigValue {
    fn from(b: bool) -> Self { ConfigValue::Bool(b) }
}

impl From<f64> for ConfigValue {
    fn from(n: f64) -> Self { ConfigValue::Number(n) }
}

impl From<i64> for ConfigValue {
    fn from(n: i64) -> Self { ConfigValue::Number(n as f64) }
}

pub trait FromConfigValue: Sized {
    fn from_config(value: &ConfigValue) -> Option<Self>;
}

impl FromConfigValue for String {
    fn from_config(value: &ConfigValue) -> Option<Self> {
        value.as_str().map(|s| s.to_string())
    }
}

impl FromConfigValue for bool {
    fn from_config(value: &ConfigValue) -> Option<Self> { value.as_bool() }
}

impl FromConfigValue for f64 {
    fn from_config(value: &ConfigValue) -> Option<Self> { value.as_f64() }
}

impl FromConfigValue for u32 {
    fn from_config(value: &ConfigValue) -> Option<Self> {
        value.as_f64().map(|n| n as u32)
    }
}

impl FromConfigValue for usize {
    fn from_config(value: &ConfigValue) -> Option<Self> {
        value.as_f64().map(|n| n as usize)
    }
}

pub struct ConfigManager {
    config: HashMap<String, (ConfigValue, ConfigSource)>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self { config: HashMap::new() }
    }

    pub fn load_all() -> Self {
        let mut mgr = Self::new();
        mgr.load_global_defaults();
        mgr.load_user_config();
        mgr.load_project_config();
        mgr.load_env_vars();
        mgr
    }

    fn load_global_defaults(&mut self) {
        self.config.insert("npm_registry".into(), (ConfigValue::String("https://registry.npmjs.org".into()), ConfigSource::GlobalDefault));
        self.config.insert("default_engine".into(), (ConfigValue::String("auto".into()), ConfigSource::GlobalDefault));
        self.config.insert("telemetry_enabled".into(), (ConfigValue::Bool(false), ConfigSource::GlobalDefault));
        self.config.insert("cache_dir".into(), (ConfigValue::String(
            dirs::cache_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("klyron").to_string_lossy().to_string()
        ), ConfigSource::GlobalDefault));
        self.config.insert("engine_pool_size".into(), (ConfigValue::Number(4.0), ConfigSource::GlobalDefault));
        self.config.insert("pre_warm".into(), (ConfigValue::Bool(false), ConfigSource::GlobalDefault));
        self.config.insert("json_output".into(), (ConfigValue::Bool(false), ConfigSource::GlobalDefault));
        self.config.insert("max_retries".into(), (ConfigValue::Number(3.0), ConfigSource::GlobalDefault));
        self.config.insert("timeout_seconds".into(), (ConfigValue::Number(30.0), ConfigSource::GlobalDefault));
        self.config.insert("strict_mode".into(), (ConfigValue::Bool(true), ConfigSource::GlobalDefault));
        self.config.insert("verify_integrity".into(), (ConfigValue::Bool(true), ConfigSource::GlobalDefault));
        self.config.insert("verify_signatures".into(), (ConfigValue::Bool(false), ConfigSource::GlobalDefault));
    }

    fn load_user_config(&mut self) {
        let path = Self::user_config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                self.parse_and_merge(&content, ConfigSource::UserFile);
            }
        }
    }

    fn load_project_config(&mut self) {
        let cwd = std::env::current_dir().unwrap_or_default();
        let filenames = ["klyron.json", "klyron.config.json", ".klyronrc", "klyronrc.json"];
        for name in &filenames {
            let path = cwd.join(name);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    self.parse_and_merge(&content, ConfigSource::ProjectFile);
                }
                break;
            }
        }
    }

    fn load_env_vars(&mut self) {
        for (key, val) in std::env::vars() {
            if let Some(config_key) = key.strip_prefix("KLYRON_").or_else(|| key.strip_prefix("KLYRON_")) {
                let config_key = config_key.to_lowercase().replace('_', "-");
                self.config.insert(config_key, (ConfigValue::String(val), ConfigSource::EnvVar));
            }
        }
    }

    fn parse_and_merge(&mut self, content: &str, source: ConfigSource) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
            self.merge_json("", &json, source);
        } else if let Ok(toml) = toml::from_str::<toml::Value>(content) {
            self.merge_toml("", &toml, source);
        }
    }

    fn merge_json(&mut self, prefix: &str, value: &serde_json::Value, source: ConfigSource) {
        match value {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    let key = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
                    self.merge_json(&key, v, source);
                }
            }
            _ => {
                let cv = serde_json_to_config(value);
                self.config.insert(prefix.to_string(), (cv, source));
            }
        }
    }

    fn merge_toml(&mut self, prefix: &str, value: &toml::Value, source: ConfigSource) {
        match value {
            toml::Value::Table(map) => {
                for (k, v) in map {
                    let key = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
                    self.merge_toml(&key, v, source);
                }
            }
            _ => {
                let cv = toml_to_config(value);
                self.config.insert(prefix.to_string(), (cv, source));
            }
        }
    }

    pub fn get<T: FromConfigValue>(&self, key: &str) -> Option<T> {
        self.config.get(key).and_then(|(v, _)| T::from_config(v))
    }

    pub fn set(&mut self, key: &str, value: ConfigValue) -> Result<(), String> {
        self.config.insert(key.to_string(), (value, ConfigSource::CliArg));
        Ok(())
    }

    pub fn unset(&mut self, key: &str) -> Result<(), String> {
        self.config.remove(key);
        Ok(())
    }

    pub fn list(&self) -> Vec<(String, ConfigValue, ConfigSource)> {
        let mut items: Vec<_> = self.config.iter()
            .map(|(k, (v, s))| (k.clone(), v.clone(), s.clone()))
            .collect();
        items.sort_by(|a, b| a.0.cmp(&b.0));
        items
    }

    pub fn get_source(&self, key: &str) -> ConfigSource {
        self.config.get(key).map(|(_, s)| s.clone()).unwrap_or(ConfigSource::GlobalDefault)
    }

    pub fn save_user_config(&self) -> Result<(), String> {
        let path = Self::user_config_path();
        let map: serde_json::Map<String, serde_json::Value> = self.config.iter()
            .filter(|(_, (_, s))| matches!(s, ConfigSource::CliArg | ConfigSource::UserFile))
            .map(|(k, (v, _))| (k.clone(), config_to_serde_json(v)))
            .collect();
        let content = serde_json::to_string_pretty(&map).map_err(|e| e.to_string())?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn save_project_config(&self, dir: &Path) -> Result<(), String> {
        let path = dir.join("klyron.json");
        let map: serde_json::Map<String, serde_json::Value> = self.config.iter()
            .filter(|(_, (_, s))| matches!(s, ConfigSource::CliArg | ConfigSource::ProjectFile))
            .map(|(k, (v, _))| (k.clone(), config_to_serde_json(v)))
            .collect();
        let content = serde_json::to_string_pretty(&map).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_npm_registry(&self) -> String {
        self.get::<String>("npm_registry").unwrap_or_else(|| "https://registry.npmjs.org".into())
    }

    pub fn get_cache_dir(&self) -> PathBuf {
        self.get::<String>("cache_dir").map(PathBuf::from).unwrap_or_else(|| {
            dirs::cache_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("klyron")
        })
    }

    pub fn get_default_engine(&self) -> String {
        self.get::<String>("default_engine").unwrap_or_else(|| "auto".into())
    }

    pub fn get_telemetry_enabled(&self) -> bool {
        self.get::<bool>("telemetry_enabled").unwrap_or(false)
    }

    fn user_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("klyron")
            .join("config.json")
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

fn serde_json_to_config(v: &serde_json::Value) -> ConfigValue {
    match v {
        serde_json::Value::Null => ConfigValue::Null,
        serde_json::Value::Bool(b) => ConfigValue::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() { ConfigValue::Number(f) }
            else { ConfigValue::Number(0.0) }
        }
        serde_json::Value::String(s) => ConfigValue::String(s.clone()),
        serde_json::Value::Array(arr) => ConfigValue::Array(arr.iter().map(serde_json_to_config).collect()),
        serde_json::Value::Object(obj) => ConfigValue::Object(obj.iter().map(|(k, v)| (k.clone(), serde_json_to_config(v))).collect()),
    }
}

fn config_to_serde_json(v: &ConfigValue) -> serde_json::Value {
    match v {
        ConfigValue::Null => serde_json::Value::Null,
        ConfigValue::Bool(b) => serde_json::Value::Bool(*b),
        ConfigValue::Number(n) => serde_json::Value::Number(serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0))),
        ConfigValue::String(s) => serde_json::Value::String(s.clone()),
        ConfigValue::Array(arr) => serde_json::Value::Array(arr.iter().map(config_to_serde_json).collect()),
        ConfigValue::Object(obj) => serde_json::Value::Object(obj.iter().map(|(k, v)| (k.clone(), config_to_serde_json(v))).collect()),
    }
}

fn toml_to_config(v: &toml::Value) -> ConfigValue {
    match v {
        toml::Value::String(s) => ConfigValue::String(s.clone()),
        toml::Value::Integer(i) => ConfigValue::Number(*i as f64),
        toml::Value::Float(f) => ConfigValue::Number(*f),
        toml::Value::Bool(b) => ConfigValue::Bool(*b),
        toml::Value::Array(arr) => ConfigValue::Array(arr.iter().map(toml_to_config).collect()),
        toml::Value::Table(t) => ConfigValue::Object(t.iter().map(|(k, v)| (k.clone(), toml_to_config(v))).collect()),
        toml::Value::Datetime(_) => ConfigValue::String(v.to_string()),
    }
}
