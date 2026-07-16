use std::fmt;
use klyron_engine_common::CommonError;

#[derive(Debug)]
pub enum QuickJSError {
    RuntimeError(String),
    EvalError(String),
    ValueError(String),
    Internal(String),
}

impl fmt::Display for QuickJSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RuntimeError(msg) => write!(f, "QuickJS runtime error: {}", msg),
            Self::EvalError(msg) => write!(f, "QuickJS eval error: {}", msg),
            Self::ValueError(msg) => write!(f, "QuickJS value error: {}", msg),
            Self::Internal(msg) => write!(f, "QuickJS internal error: {}", msg),
        }
    }
}

impl std::error::Error for QuickJSError {}

impl From<String> for QuickJSError {
    fn from(msg: String) -> Self {
        Self::RuntimeError(msg)
    }
}

impl CommonError for QuickJSError {
    fn kind(&self) -> klyron_engine_common::CommonErrorKind {
        klyron_engine_common::CommonErrorKind::ExecutionFailed(self.to_string())
    }
}