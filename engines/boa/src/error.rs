use std::fmt;
use boa_engine::JsNativeErrorKind;
use klyron_engine_common::error::{CommonError, CommonErrorKind};

#[derive(Debug, Clone)]
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
    PermissionDenied(String),
    ModuleNotFound(String),
    EngineBusy,
    EnginePoolExhausted,
    SnapshotError(String),
    CacheError(String),
    JsError(String),
    StackTrace(String),
}

impl BoaError {
    pub fn from_js_error(err: &boa_engine::JsError) -> Self {
        let msg = err.to_string();
        let kind = err.as_native().map(|n| &n.kind);
        match kind {
            Some(JsNativeErrorKind::Type) => Self::TypeError(msg),
            Some(JsNativeErrorKind::Range) => Self::RangeError(msg),
            Some(JsNativeErrorKind::Reference) => Self::ReferenceError(msg),
            Some(JsNativeErrorKind::Syntax) => Self::SyntaxError(msg),
            _ => Self::ExecutionFailed(msg),
        }
    }

    pub fn from_js_error_with_context(err: &boa_engine::JsError) -> Self {
        Self::from_js_error(err)
    }

    pub fn to_common_kind(&self) -> CommonErrorKind {
        match self {
            Self::NotInitialized => CommonErrorKind::NotInitialized,
            Self::InitFailed(msg) => CommonErrorKind::InitFailed(msg.clone()),
            Self::ExecutionFailed(msg) | Self::JsError(msg) => CommonErrorKind::ExecutionFailed(msg.clone()),
            Self::CompileError(msg) => CommonErrorKind::CompileError(msg.clone()),
            Self::SyntaxError(msg) => CommonErrorKind::SyntaxError(msg.clone()),
            Self::TypeError(msg) => CommonErrorKind::TypeError(msg.clone()),
            Self::RangeError(msg) => CommonErrorKind::RangeError(msg.clone()),
            Self::ReferenceError(msg) => CommonErrorKind::ReferenceError(msg.clone()),
            Self::Timeout => CommonErrorKind::Timeout,
            Self::OutOfMemory => CommonErrorKind::OutOfMemory,
            Self::PermissionDenied(msg) => CommonErrorKind::PermissionDenied(msg.clone()),
            Self::ModuleNotFound(msg) => CommonErrorKind::ModuleNotFound(msg.clone()),
            Self::EngineBusy => CommonErrorKind::ExecutionFailed("engine is busy".into()),
            Self::EnginePoolExhausted => CommonErrorKind::ExecutionFailed("engine pool exhausted".into()),
            Self::SnapshotError(msg) => CommonErrorKind::ExecutionFailed(format!("snapshot error: {}", msg)),
            Self::CacheError(msg) => CommonErrorKind::ExecutionFailed(format!("cache error: {}", msg)),
            Self::StackTrace(msg) => CommonErrorKind::ExecutionFailed(msg.clone()),
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
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {msg}"),
            Self::ModuleNotFound(msg) => write!(f, "Module not found: {msg}"),
            Self::EngineBusy => write!(f, "Engine is busy"),
            Self::EnginePoolExhausted => write!(f, "Engine pool exhausted"),
            Self::SnapshotError(msg) => write!(f, "Snapshot error: {msg}"),
            Self::CacheError(msg) => write!(f, "Cache error: {msg}"),
            Self::JsError(msg) => write!(f, "{msg}"),
            Self::StackTrace(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for BoaError {}

impl CommonError for BoaError {
    fn kind(&self) -> CommonErrorKind { self.to_common_kind() }
    fn format_stack_trace(&self) -> Option<String> {
        match self {
            Self::JsError(msg) if msg.contains("\n  at ") => {
                let parts: Vec<&str> = msg.splitn(2, "\n  at ").collect();
                if parts.len() > 1 { Some(format!("  at {}", parts[1])) } else { Some(msg.clone()) }
            }
            Self::StackTrace(s) => Some(s.clone()),
            _ => None,
        }
    }
}

impl From<anyhow::Error> for BoaError {
    fn from(e: anyhow::Error) -> Self { Self::ExecutionFailed(e.to_string()) }
}

impl From<boa_engine::JsError> for BoaError {
    fn from(e: boa_engine::JsError) -> Self { Self::from_js_error(&e) }
}
