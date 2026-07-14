//! WASM bindings for klyron_plugin
use wasm_bindgen::prelude::*;
use crate::types::PluginConfig;

#[wasm_bindgen]
pub struct PluginHandle;

#[wasm_bindgen]
impl PluginHandle {
    pub fn new() -> Self { Self }
    pub fn version(&self) -> String { "klyron_plugin 0.1.0".into() }
}
