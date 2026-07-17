use std::fmt;
use klyron_engine_common::error::{CommonError, CommonErrorKind};

#[derive(Debug)]
pub enum V8Error {
    NotInitialized,
    InitFailed(String),
    EvalFailed(String),
    ModuleFailed(String),
    GlobalFailed(String),
    CallFailed(String),
    SnapshotFailed(String),
    Internal(String),
}

impl fmt::Display for V8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "V8 not initialized"),
            Self::InitFailed(msg) => write!(f, "V8 init failed: {msg}"),
            Self::EvalFailed(msg) => write!(f, "V8 eval error: {msg}"),
            Self::ModuleFailed(msg) => write!(f, "V8 module error: {msg}"),
            Self::GlobalFailed(msg) => write!(f, "V8 global error: {msg}"),
            Self::CallFailed(msg) => write!(f, "V8 call error: {msg}"),
            Self::SnapshotFailed(msg) => write!(f, "V8 snapshot error: {msg}"),
            Self::Internal(msg) => write!(f, "V8 internal error: {msg}"),
        }
    }
}

impl std::error::Error for V8Error {}

impl V8Error {
    pub fn to_common_kind(&self) -> CommonErrorKind {
        match self {
            Self::NotInitialized => CommonErrorKind::NotInitialized,
            Self::InitFailed(msg) => CommonErrorKind::InitFailed(msg.clone()),
            Self::EvalFailed(msg) | Self::ModuleFailed(msg) | Self::GlobalFailed(msg)
                | Self::CallFailed(msg) | Self::Internal(msg) => CommonErrorKind::ExecutionFailed(msg.clone()),
            Self::SnapshotFailed(msg) => CommonErrorKind::ExecutionFailed(msg.clone()),
        }
    }
}

impl CommonError for V8Error {
    fn kind(&self) -> CommonErrorKind {
        self.to_common_kind()
    }
}
