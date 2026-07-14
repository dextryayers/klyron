//! Test utilities for klyron_template
use crate::types::TemplateConfig;

pub fn test_config() -> TemplateConfig {
    TemplateConfig { version: "0.0.0".into() }
}
