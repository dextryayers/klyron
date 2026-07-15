use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

static TELEMETRY_INSTANCE: once_cell::sync::Lazy<Mutex<TelemetryManager>> =
    once_cell::sync::Lazy::new(|| Mutex::new(TelemetryManager::new()));

#[derive(Debug, Clone)]
pub enum TelemetryEvent {
    CommandExecuted { command: String, duration_ms: u64, success: bool },
    InstallCompleted { packages: usize, duration_ms: u64 },
    BuildCompleted { duration_ms: u64, output_size: u64 },
    DevStarted { framework: Option<String> },
    Error { error_type: String, message: String },
    Performance { operation: String, duration_ms: u64, data: HashMap<String, f64> },
    SystemInfo { os: String, arch: String, cpu_cores: u32, memory_mb: u64, engine: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacyMode {
    None,
    Basic,
    Full,
}

#[derive(Debug, Clone)]
pub struct TelemetryStatus {
    pub enabled: bool,
    pub privacy_mode: PrivacyMode,
    pub session_id: String,
    pub install_id: String,
    pub event_count: usize,
    pub last_submitted: Option<String>,
}

pub struct TelemetryManager {
    enabled: bool,
    privacy_mode: PrivacyMode,
    event_buffer: Vec<TelemetryEvent>,
    config_path: PathBuf,
    session_id: String,
    install_id: String,
}

impl TelemetryManager {
    pub fn new() -> Self {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("klyron")
            .join("telemetry.json");

        let install_id = Self::load_or_create_id(&config_path);

        Self {
            enabled: false,
            privacy_mode: PrivacyMode::Basic,
            event_buffer: Vec::new(),
            config_path,
            session_id: uuid_v4(),
            install_id,
        }
    }

    fn load_or_create_id(path: &PathBuf) -> String {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(id) = json["install_id"].as_str() {
                        return id.to_string();
                    }
                }
            }
        }
        let id = uuid_v4();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = serde_json::json!({ "install_id": id });
        if let Ok(json) = serde_json::to_string_pretty(&content) {
            let _ = std::fs::write(path, json);
        }
        id
    }

    pub fn enable() -> Result<(), String> {
        let mut mgr = TELEMETRY_INSTANCE.lock().map_err(|e| e.to_string())?;
        mgr.enabled = true;
        mgr.save_pref(true)
    }

    pub fn disable() -> Result<(), String> {
        let mut mgr = TELEMETRY_INSTANCE.lock().map_err(|e| e.to_string())?;
        mgr.enabled = false;
        mgr.save_pref(false)
    }

    pub fn is_enabled() -> bool {
        TELEMETRY_INSTANCE.lock().map(|m| m.enabled).unwrap_or(false)
    }

    pub fn record_event(event: TelemetryEvent) {
        if let Ok(mut mgr) = TELEMETRY_INSTANCE.lock() {
            if mgr.enabled {
                let event = mgr.anonymize(event);
                mgr.event_buffer.push(event);
                if mgr.event_buffer.len() >= 50 {
                    let _ = mgr.submit_events_inner();
                }
            }
        }
    }

    pub fn submit_events() -> Result<(), String> {
        let mut mgr = TELEMETRY_INSTANCE.lock().map_err(|e| e.to_string())?;
        mgr.submit_events_inner()
    }

    fn submit_events_inner(&mut self) -> Result<(), String> {
        if self.event_buffer.is_empty() {
            return Ok(());
        }

        let events: Vec<serde_json::Value> = self.event_buffer.iter().map(|e| self.event_to_json(e)).collect();
        let payload = serde_json::json!({
            "session_id": self.session_id,
            "install_id": self.install_id,
            "events": events,
            "timestamp": chrono_now_iso(),
        });

        let url = "https://telemetry.klyron.dev/v1/events";
        let resp = ureq::post(url)
            .set("Content-Type", "application/json")
            .set("User-Agent", "klyron-telemetry")
            .timeout(std::time::Duration::from_secs(5))
            .send_json(payload);

        match resp {
            Ok(_) => {
                self.event_buffer.clear();
                self.save_last_submitted();
                Ok(())
            }
            Err(e) => Err(format!("Failed to submit telemetry: {e}")),
        }
    }

    pub fn flush() {
        let _ = Self::submit_events();
    }

    pub fn get_status() -> TelemetryStatus {
        let mgr = TELEMETRY_INSTANCE.lock().unwrap_or_else(|e| e.into_inner());
        TelemetryStatus {
            enabled: mgr.enabled,
            privacy_mode: mgr.privacy_mode,
            session_id: mgr.session_id.clone(),
            install_id: mgr.install_id.clone(),
            event_count: mgr.event_buffer.len(),
            last_submitted: None,
        }
    }

    pub fn generate_report() -> String {
        let status = Self::get_status();
        format!(
            "Telemetry Report\n\
             ================\n\
             Enabled: {}\n\
             Privacy Mode: {:?}\n\
             Session ID: {}\n\
             Install ID: {}\n\
             Buffered Events: {}\n",
            if status.enabled { "yes" } else { "no" },
            status.privacy_mode,
            status.session_id,
            status.install_id,
            status.event_count,
        )
    }

    pub fn anonymize(event: TelemetryEvent) -> TelemetryEvent {
        match event {
            TelemetryEvent::Error { error_type, message: _ } => {
                TelemetryEvent::Error {
                    error_type,
                    message: "<redacted>".to_string(),
                }
            }
            TelemetryEvent::CommandExecuted { command, duration_ms, success } => {
                TelemetryEvent::CommandExecuted {
                    command: hash_string(&command),
                    duration_ms,
                    success,
                }
            }
            other => other,
        }
    }

    pub fn set_privacy_mode(mode: PrivacyMode) {
        if let Ok(mut mgr) = TELEMETRY_INSTANCE.lock() {
            mgr.privacy_mode = mode;
        }
    }

    pub fn view_data() -> String {
        let mgr = TELEMETRY_INSTANCE.lock().unwrap_or_else(|e| e.into_inner());
        let events: Vec<serde_json::Value> = mgr.event_buffer.iter().map(|e| mgr.event_to_json(e)).collect();
        serde_json::to_string_pretty(&serde_json::json!({
            "session_id": mgr.session_id,
            "install_id": mgr.install_id,
            "enabled": mgr.enabled,
            "privacy_mode": format!("{:?}", mgr.privacy_mode),
            "events": events,
        })).unwrap_or_default()
    }

    pub fn delete_data() -> Result<(), String> {
        let payload = serde_json::json!({
            "install_id": Self::get_status().install_id,
        });
        let url = "https://telemetry.klyron.dev/v1/delete";
        let resp = ureq::post(url)
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(10))
            .send_json(payload);

        match resp {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to delete telemetry data: {e}")),
        }
    }

    fn save_pref(&self, enabled: bool) -> Result<(), String> {
        let pref_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("klyron")
            .join("config.toml");
        if let Some(parent) = pref_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = format!("[telemetry]\nenabled = {enabled}\n");
        std::fs::write(&pref_path, content).map_err(|e| e.to_string())
    }

    fn save_last_submitted(&self) {
        let path = self.config_path.parent().map(|p| p.join("last_submitted.txt"));
        if let Some(path) = path {
            let _ = std::fs::write(path, chrono_now_iso());
        }
    }

    fn event_to_json(&self, event: &TelemetryEvent) -> serde_json::Value {
        match event {
            TelemetryEvent::CommandExecuted { command, duration_ms, success } => {
                serde_json::json!({
                    "type": "command_executed",
                    "command": command,
                    "duration_ms": duration_ms,
                    "success": success,
                })
            }
            TelemetryEvent::InstallCompleted { packages, duration_ms } => {
                serde_json::json!({
                    "type": "install_completed",
                    "packages": packages,
                    "duration_ms": duration_ms,
                })
            }
            TelemetryEvent::BuildCompleted { duration_ms, output_size } => {
                serde_json::json!({
                    "type": "build_completed",
                    "duration_ms": duration_ms,
                    "output_size": output_size,
                })
            }
            TelemetryEvent::DevStarted { framework } => {
                serde_json::json!({
                    "type": "dev_started",
                    "framework": framework,
                })
            }
            TelemetryEvent::Error { error_type, message } => {
                serde_json::json!({
                    "type": "error",
                    "error_type": error_type,
                    "message": message,
                })
            }
            TelemetryEvent::Performance { operation, duration_ms, data } => {
                serde_json::json!({
                    "type": "performance",
                    "operation": operation,
                    "duration_ms": duration_ms,
                    "data": data,
                })
            }
            TelemetryEvent::SystemInfo { os, arch, cpu_cores, memory_mb, engine } => {
                serde_json::json!({
                    "type": "system_info",
                    "os": os,
                    "arch": arch,
                    "cpu_cores": cpu_cores,
                    "memory_mb": memory_mb,
                    "engine": engine,
                })
            }
        }
    }
}

impl Default for TelemetryManager {
    fn default() -> Self {
        Self::new()
    }
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let nanos = now.as_nanos();
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (nanos >> 32) as u32,
        (nanos >> 16) as u16 & 0xffff,
        (nanos & 0xfff) as u16,
        ((nanos >> 48) as u16 & 0x3fff) | 0x8000,
        nanos as u64 & 0xffffffffffff,
    )
}

fn chrono_now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs();
    let millis = d.subsec_millis();
    let days = secs / 86400;
    let time = secs % 86400;
    let hours = time / 3600;
    let mins = (time % 3600) / 60;
    let secs = time % 60;
    format!("{days:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{millis:03}Z", 1 + (days % 31), 1 + (days % 12), hours, mins, secs)
}

fn hash_string(s: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())[..16].to_string()
}
