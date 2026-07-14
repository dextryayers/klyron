use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmError {
    #[error("package manager not found")]
    NotFound,
    #[error("install failed: {0}")]
    InstallFailed(String),
    #[error("unsupported package manager: {0:?}")]
    Unsupported(PackageManager),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
use crate::types::PackageManager;
