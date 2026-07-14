//! Types for klyron_telemetry
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub version: String,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self { version: "0.1.0".into() }
    }
}
