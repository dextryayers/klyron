//! Error types for Quickjs

use std::fmt;

#[derive(Debug)]
pub enum QuickjsError {
    NotInitialized,
    ExecutionFailed(String),
    CompileError(String),
    SyntaxError(String),
    TypeError(String),
    RangeError(String),
    ReferenceError(String),
    Timeout,
    OutOfMemory,
}

impl fmt::Display for QuickjsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "Quickjs not initialized"),
            Self::ExecutionFailed(msg) => write!(f, "Execution failed: {msg}"),
            Self::CompileError(msg) => write!(f, "Compile error: {msg}"),
            Self::SyntaxError(msg) => write!(f, "Syntax error: {msg}"),
            Self::TypeError(msg) => write!(f, "Type error: {msg}"),
            Self::RangeError(msg) => write!(f, "Range error: {msg}"),
            Self::ReferenceError(msg) => write!(f, "Reference error: {msg}"),
            Self::Timeout => write!(f, "Script timeout"),
            Self::OutOfMemory => write!(f, "Out of memory"),
        }
    }
}

impl std::error::Error for QuickjsError {}

impl From<anyhow::Error> for QuickjsError {
    fn from(e: anyhow::Error) -> Self {
        Self::ExecutionFailed(e.to_string())
    }
}
