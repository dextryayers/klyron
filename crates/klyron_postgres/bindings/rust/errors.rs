//! Error types for klyron_postgres

use std::fmt;

#[derive(Debug)]
pub enum Klyron::PostgresError {
    NotFound(String),
    InvalidInput(String),
    OperationFailed(String),
    IoError(std::io::Error),
}

impl fmt::Display for Klyron::PostgresError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "NotFound: {msg}"),
            Self::InvalidInput(msg) => write!(f, "InvalidInput: {msg}"),
            Self::OperationFailed(msg) => write!(f, "OperationFailed: {msg}"),
            Self::IoError(e) => write!(f, "IoError: {e}"),
        }
    }
}

impl std::error::Error for Klyron::PostgresError {}

impl From<std::io::Error> for Klyron::PostgresError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<Klyron::PostgresError> for anyhow::Error {
    fn from(e: Klyron::PostgresError) -> Self {
        anyhow::anyhow!("{}", e)
    }
}
