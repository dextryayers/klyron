//! Error types for klyron_engine

use std::fmt;

#[derive(Debug)]
pub enum Klyron::EngineError {
    NotFound(String),
    InvalidInput(String),
    OperationFailed(String),
    IoError(std::io::Error),
}

impl fmt::Display for Klyron::EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "NotFound: {msg}"),
            Self::InvalidInput(msg) => write!(f, "InvalidInput: {msg}"),
            Self::OperationFailed(msg) => write!(f, "OperationFailed: {msg}"),
            Self::IoError(e) => write!(f, "IoError: {e}"),
        }
    }
}

impl std::error::Error for Klyron::EngineError {}

impl From<std::io::Error> for Klyron::EngineError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<Klyron::EngineError> for anyhow::Error {
    fn from(e: Klyron::EngineError) -> Self {
        anyhow::anyhow!("{}", e)
    }
}
