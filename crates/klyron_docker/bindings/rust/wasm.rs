//! WASM bindings for klyron_docker
use wasm_bindgen::prelude::*;
use crate::types::DockerConfig;

#[wasm_bindgen]
pub struct DockerHandle;

#[wasm_bindgen]
impl DockerHandle {
    pub fn new() -> Self { Self }
    pub fn version(&self) -> String { "klyron_docker 0.1.0".into() }
}
