//! WASM bindings for klyron_config
use wasm_bindgen::prelude::*;
use crate::types::ConfigConfig;

#[wasm_bindgen]
pub struct ConfigHandle;

#[wasm_bindgen]
impl ConfigHandle {
    pub fn new() -> Self { Self }
    pub fn version(&self) -> String { "klyron_config 0.1.0".into() }
}
