//! WASM bindings for klyron_template
use wasm_bindgen::prelude::*;
use crate::types::TemplateConfig;

#[wasm_bindgen]
pub struct TemplateHandle;

#[wasm_bindgen]
impl TemplateHandle {
    pub fn new() -> Self { Self }
    pub fn version(&self) -> String { "klyron_template 0.1.0".into() }
}
