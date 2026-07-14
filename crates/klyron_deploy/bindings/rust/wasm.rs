//! WASM bindings for klyron_deploy
use wasm_bindgen::prelude::*;
use crate::types::DeployConfig;

#[wasm_bindgen]
pub struct DeployHandle;

#[wasm_bindgen]
impl DeployHandle {
    pub fn new() -> Self { Self }
    pub fn version(&self) -> String { "klyron_deploy 0.1.0".into() }
}
