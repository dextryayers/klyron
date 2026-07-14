use std::ffi::CString;
use std::fmt;

#[derive(Debug)]
pub enum QuickJSError {
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

impl QuickJSError {
    pub unsafe fn catch_error(_ctx: *mut std::ffi::c_void) -> Option<Self> {
        None
    }

    pub unsafe fn format_stack_trace(_ctx: *mut std::ffi::c_void) -> String {
        "No stack trace (QuickJS native not linked)".to_string()
    }
}

impl fmt::Display for QuickJSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "QuickJS not initialized"),
            Self::InitFailed(msg) => write!(f, "QuickJS initialization failed: {msg}"),
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

impl std::error::Error for QuickJSError {}

impl From<anyhow::Error> for QuickJSError {
    fn from(e: anyhow::Error) -> Self {
        Self::ExecutionFailed(e.to_string())
    }
}

impl From<CString> for QuickJSError {
    fn from(_e: CString) -> Self {
        Self::ExecutionFailed("Nul error in string".into())
    }
}
