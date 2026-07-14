use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::engine::JsEngineKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineSnapshot {
    pub engine_kind: JsEngineKind,
    pub data: Vec<u8>,
    pub created_at: u64,
    pub version: u32,
}

impl EngineSnapshot {
    pub fn new(kind: JsEngineKind, data: Vec<u8>) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            engine_kind: kind,
            data,
            created_at,
            version: 1,
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self).map_err(|e| format!("Snapshot serialization failed: {}", e))
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, String> {
        bincode::deserialize(bytes).map_err(|e| format!("Snapshot deserialization failed: {}", e))
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), String> {
        let data = self.serialize()?;
        std::fs::write(path, data).map_err(|e| format!("Failed to write snapshot: {}", e))
    }

    pub fn load_from_file(path: &std::path::Path) -> Result<Self, String> {
        let data = std::fs::read(path).map_err(|e| format!("Failed to read snapshot: {}", e))?;
        Self::deserialize(&data)
    }

    pub fn age_seconds(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.created_at)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupSnapshot {
    pub engine_kind: JsEngineKind,
    pub warmup_scripts: Vec<String>,
    pub created_at: u64,
}

impl WarmupSnapshot {
    pub fn new(kind: JsEngineKind) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            engine_kind: kind,
            warmup_scripts: Vec::new(),
            created_at,
        }
    }

    pub fn add_script(&mut self, script: &str) {
        self.warmup_scripts.push(script.to_string());
    }
}
