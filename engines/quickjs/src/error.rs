use std::fmt;
use klyron_engine_common::error::{CommonError, CommonErrorKind};

#[derive(Debug)]
pub enum QuickJSError {
    NotInitialized,
    InitFailed(String),
    EvalFailed(String),
    ModuleFailed(String),
    GlobalGetFailed(String),
    GlobalSetFailed(String),
    CallFailed(String),
    SnapshotFailed(String),
    Internal(String),
}

impl fmt::Display for QuickJSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "QuickJS not initialized"),
            Self::InitFailed(msg) => write!(f, "QuickJS init failed: {msg}"),
            Self::EvalFailed(msg) => write!(f, "QuickJS eval error: {msg}"),
            Self::ModuleFailed(msg) => write!(f, "QuickJS module error: {msg}"),
            Self::GlobalGetFailed(msg) => write!(f, "QuickJS global get error: {msg}"),
            Self::GlobalSetFailed(msg) => write!(f, "QuickJS global set error: {msg}"),
            Self::CallFailed(msg) => write!(f, "QuickJS call error: {msg}"),
            Self::SnapshotFailed(msg) => write!(f, "QuickJS snapshot error: {msg}"),
            Self::Internal(msg) => write!(f, "QuickJS internal error: {msg}"),
        }
    }
}

impl std::error::Error for QuickJSError {}

impl QuickJSError {
    pub fn to_common_kind(&self) -> CommonErrorKind {
        match self {
            Self::NotInitialized => CommonErrorKind::NotInitialized,
            Self::InitFailed(msg) => CommonErrorKind::InitFailed(msg.clone()),
            Self::EvalFailed(msg) | Self::ModuleFailed(msg) | Self::GlobalGetFailed(msg)
                | Self::GlobalSetFailed(msg) | Self::CallFailed(msg) | Self::Internal(msg) => CommonErrorKind::ExecutionFailed(msg.clone()),
            Self::SnapshotFailed(msg) => CommonErrorKind::ExecutionFailed(msg.clone()),
        }
    }
}

impl CommonError for QuickJSError {
    fn kind(&self) -> CommonErrorKind {
        self.to_common_kind()
    }
}
