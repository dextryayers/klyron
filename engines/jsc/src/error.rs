use std::fmt;
use klyron_engine_common::error::{CommonError, CommonErrorKind};

#[derive(Debug)]
pub enum JSCError {
    NotInitialized,
    InitFailed(String),
    EvalFailed(String),
    ModuleFailed(String),
    GlobalFailed(String),
    CallFailed(String),
    SnapshotFailed(String),
    Internal(String),
}

impl fmt::Display for JSCError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "JSC not initialized"),
            Self::InitFailed(msg) => write!(f, "JSC init failed: {msg}"),
            Self::EvalFailed(msg) => write!(f, "JSC eval error: {msg}"),
            Self::ModuleFailed(msg) => write!(f, "JSC module error: {msg}"),
            Self::GlobalFailed(msg) => write!(f, "JSC global error: {msg}"),
            Self::CallFailed(msg) => write!(f, "JSC call error: {msg}"),
            Self::SnapshotFailed(msg) => write!(f, "JSC snapshot error: {msg}"),
            Self::Internal(msg) => write!(f, "JSC internal error: {msg}"),
        }
    }
}

impl std::error::Error for JSCError {}

impl JSCError {
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

impl CommonError for JSCError {
    fn kind(&self) -> CommonErrorKind {
        self.to_common_kind()
    }
}
