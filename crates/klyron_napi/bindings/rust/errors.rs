use thiserror::Error;

#[derive(Error, Debug)]
pub enum NapiError {
    #[error("module '{0}' not found")]
    ModuleNotFound(String),
    #[error("failed to load native module: {0}")]
    LoadFailed(String),
    #[error("symbol '{0}' not exported")]
    SymbolNotFound(String),
    #[error("version mismatch: expected {expected}, got {got}")]
    VersionMismatch { expected: u32, got: u32 },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
