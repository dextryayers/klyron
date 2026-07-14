//! WASM bindings for klyron_telemetry
use wasm_bindgen::prelude::*;
use crate::types::TelemetryConfig;

#[wasm_bindgen]
pub struct TelemetryHandle;

#[wasm_bindgen]
impl TelemetryHandle {
    pub fn new() -> Self { Self }
    pub fn version(&self) -> String { "klyron_telemetry 0.1.0".into() }
}
