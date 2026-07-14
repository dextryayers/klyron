//! WASM bindings for klyron_compat
use wasm_bindgen::prelude::*;
use crate::types::CompatConfig;

#[wasm_bindgen]
pub struct CompatHandle;

#[wasm_bindgen]
impl CompatHandle {
    pub fn new() -> Self { Self }
    pub fn version(&self) -> String { "klyron_compat 0.1.0".into() }
}
