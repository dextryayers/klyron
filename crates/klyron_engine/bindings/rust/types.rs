//! Type definitions for klyron_engine

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Klyron::EngineConfig {
    pub enabled: bool,
}

impl Default for Klyron::EngineConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Klyron::EngineResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> Klyron::EngineResult<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self { success: false, data: None, error: Some(msg.into()) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Klyron::EngineStatus {
    Active,
    Inactive,
    Error(String),
}
