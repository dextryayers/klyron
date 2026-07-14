use std::fmt;

use klyron_engine_common::error::{CommonError, CommonErrorKind};

#[derive(Debug)]
pub enum BoaError {
    NotInitialized,
    InitFailed(String),
    ExecutionFailed(String),
    CompileError(String),
    SyntaxError(String),
    TypeError(String),
    RangeError(String),
    ReferenceError(String),
    Timeout,
    OutOfMemory,
}

impl BoaError {
    pub fn catch_error(_context: &mut boa_engine::Context) -> Option<Self> {
        None
    }

    pub fn format_stack_trace(error: &boa_engine::JsValue, context: &mut boa_engine::Context) -> String {
        error.to_string(context)
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_else(|_| "No stack trace".to_string())
    }

    pub fn to_common_kind(&self) -> CommonErrorKind {
        match self {
            Self::NotInitialized => CommonErrorKind::NotInitialized,
            Self::InitFailed(msg) => CommonErrorKind::InitFailed(msg.clone()),
            Self::ExecutionFailed(msg) => CommonErrorKind::ExecutionFailed(msg.clone()),
            Self::CompileError(msg) => CommonErrorKind::CompileError(msg.clone()),
            Self::SyntaxError(msg) => CommonErrorKind::SyntaxError(msg.clone()),
            Self::TypeError(msg) => CommonErrorKind::TypeError(msg.clone()),
            Self::RangeError(msg) => CommonErrorKind::RangeError(msg.clone()),
            Self::ReferenceError(msg) => CommonErrorKind::ReferenceError(msg.clone()),
            Self::Timeout => CommonErrorKind::Timeout,
            Self::OutOfMemory => CommonErrorKind::OutOfMemory,
        }
    }
}

impl fmt::Display for BoaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "Boa not initialized"),
            Self::InitFailed(msg) => write!(f, "Boa initialization failed: {msg}"),
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

impl std::error::Error for BoaError {}

impl CommonError for BoaError {
    fn kind(&self) -> CommonErrorKind {
        self.to_common_kind()
    }

    fn format_stack_trace(&self) -> Option<String> {
        Some("Stack trace: (Boa)".to_string())
    }
}

impl From<anyhow::Error> for BoaError {
    fn from(e: anyhow::Error) -> Self {
        Self::ExecutionFailed(e.to_string())
    }
}
