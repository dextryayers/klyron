//! WASM bindings for klyron_workspace
use wasm_bindgen::prelude::*;
use crate::types::WorkspaceConfig;

#[wasm_bindgen]
pub struct WorkspaceHandle;

#[wasm_bindgen]
impl WorkspaceHandle {
    pub fn new() -> Self { Self }
    pub fn version(&self) -> String { "klyron_workspace 0.1.0".into() }
}
