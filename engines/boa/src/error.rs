use std::fmt;

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

impl From<anyhow::Error> for BoaError {
    fn from(e: anyhow::Error) -> Self {
        Self::ExecutionFailed(e.to_string())
    }
}
