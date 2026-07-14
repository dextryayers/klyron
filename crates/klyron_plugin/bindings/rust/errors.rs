//! Errors for klyron_plugin
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("initialization failed: {0}")]
    InitFailed(String),
    #[error("operation failed: {0}")]
    OperationFailed(String),
    #[error("not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, PluginError>;
