//! WASM bindings for klyron_shell
use wasm_bindgen::prelude::*;
use crate::types::ShellConfig;

#[wasm_bindgen]
pub struct ShellHandle;

#[wasm_bindgen]
impl ShellHandle {
    pub fn new() -> Self { Self }
    pub fn version(&self) -> String { "klyron_shell 0.1.0".into() }
}
