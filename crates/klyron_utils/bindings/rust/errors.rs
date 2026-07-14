//! Error types for klyron_utils

use std::fmt;

#[derive(Debug)]
pub enum Klyron::UtilsError {
    NotFound(String),
    InvalidInput(String),
    OperationFailed(String),
    IoError(std::io::Error),
}

impl fmt::Display for Klyron::UtilsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "NotFound: {msg}"),
            Self::InvalidInput(msg) => write!(f, "InvalidInput: {msg}"),
            Self::OperationFailed(msg) => write!(f, "OperationFailed: {msg}"),
            Self::IoError(e) => write!(f, "IoError: {e}"),
        }
    }
}

impl std::error::Error for Klyron::UtilsError {}

impl From<std::io::Error> for Klyron::UtilsError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<Klyron::UtilsError> for anyhow::Error {
    fn from(e: Klyron::UtilsError) -> Self {
        anyhow::anyhow!("{}", e)
    }
}
