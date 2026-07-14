//! WASM bindings for klyron_adapter
use wasm_bindgen::prelude::*;
use crate::types::AdapterConfig;

#[wasm_bindgen]
pub struct AdapterHandle;

#[wasm_bindgen]
impl AdapterHandle {
    pub fn new() -> Self { Self }
    pub fn version(&self) -> String { "klyron_adapter 0.1.0".into() }
}
