//! Errors for klyron_compat
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompatError {
    #[error("initialization failed: {0}")]
    InitFailed(String),
    #[error("operation failed: {0}")]
    OperationFailed(String),
    #[error("not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, CompatError>;
