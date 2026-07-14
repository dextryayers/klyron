//! Types for klyron_template
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub version: String,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self { version: "0.1.0".into() }
    }
}
