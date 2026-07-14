use thiserror::Error;

#[derive(Error, Debug)]
pub enum TESTError {
    #[error("operation failed: {0}")]
    OperationFailed(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
