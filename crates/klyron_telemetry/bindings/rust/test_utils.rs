//! Test utilities for klyron_telemetry
use crate::types::TelemetryConfig;

pub fn test_config() -> TelemetryConfig {
    TelemetryConfig { version: "0.0.0".into() }
}
