//! Errors for klyron_docker
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DockerError {
    #[error("initialization failed: {0}")]
    InitFailed(String),
    #[error("operation failed: {0}")]
    OperationFailed(String),
    #[error("not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, DockerError>;
