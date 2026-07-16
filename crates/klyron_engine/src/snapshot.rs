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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_snapshot_new() {
        let data = b"snapshot_data".to_vec();
        let snapshot = EngineSnapshot::new(JsEngineKind::V8, data.clone());
        assert_eq!(snapshot.engine_kind, JsEngineKind::V8);
        assert_eq!(snapshot.data, data);
        assert_eq!(snapshot.version, 1);
        assert!(snapshot.created_at > 0);
    }

    #[test]
    fn test_engine_snapshot_serialization_roundtrip() {
        let data = b"test bytecode".to_vec();
        let snapshot = EngineSnapshot::new(JsEngineKind::QuickJS, data);
        let serialized = snapshot.serialize().unwrap();
        let deserialized = EngineSnapshot::deserialize(&serialized).unwrap();
        assert_eq!(snapshot.engine_kind, deserialized.engine_kind);
        assert_eq!(snapshot.data, deserialized.data);
        assert_eq!(snapshot.version, deserialized.version);
    }

    #[test]
    fn test_engine_snapshot_age() {
        let snapshot = EngineSnapshot::new(JsEngineKind::Boa, vec![1, 2, 3]);
        let age = snapshot.age_seconds();
        assert!(age < 10);
    }

    #[test]
    fn test_engine_snapshot_different_kinds() {
        for kind in JsEngineKind::all() {
            let snapshot = EngineSnapshot::new(kind, vec![]);
            assert_eq!(snapshot.engine_kind, kind);
        }
    }

    #[test]
    fn test_engine_snapshot_empty_data() {
        let snapshot = EngineSnapshot::new(JsEngineKind::JSC, vec![]);
        assert!(snapshot.data.is_empty());
    }

    #[test]
    fn test_warmup_snapshot_new() {
        let warmup = WarmupSnapshot::new(JsEngineKind::V8);
        assert_eq!(warmup.engine_kind, JsEngineKind::V8);
        assert!(warmup.warmup_scripts.is_empty());
        assert!(warmup.created_at > 0);
    }

    #[test]
    fn test_warmup_snapshot_add_script() {
        let mut warmup = WarmupSnapshot::new(JsEngineKind::Boa);
        warmup.add_script("console.log('hello');");
        assert_eq!(warmup.warmup_scripts.len(), 1);
        assert_eq!(warmup.warmup_scripts[0], "console.log('hello');");
    }

    #[test]
    fn test_warmup_snapshot_multiple_scripts() {
        let mut warmup = WarmupSnapshot::new(JsEngineKind::QuickJS);
        warmup.add_script("script1");
        warmup.add_script("script2");
        warmup.add_script("script3");
        assert_eq!(warmup.warmup_scripts.len(), 3);
    }

    #[test]
    fn test_snapshot_deserialize_invalid() {
        let result = EngineSnapshot::deserialize(b"invalid data");
        assert!(result.is_err());
    }
}
