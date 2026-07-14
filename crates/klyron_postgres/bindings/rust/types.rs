//! Type definitions for klyron_postgres

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Klyron::PostgresConfig {
    pub enabled: bool,
}

impl Default for Klyron::PostgresConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Klyron::PostgresResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> Klyron::PostgresResult<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self { success: false, data: None, error: Some(msg.into()) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Klyron::PostgresStatus {
    Active,
    Inactive,
    Error(String),
}
