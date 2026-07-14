use std::fmt;

#[derive(Debug, Clone)]
pub enum CommonErrorKind {
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
}

impl fmt::Display for CommonErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "Engine not initialized"),
            Self::InitFailed(msg) => write!(f, "Engine initialization failed: {}", msg),
            Self::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            Self::CompileError(msg) => write!(f, "Compile error: {}", msg),
            Self::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            Self::TypeError(msg) => write!(f, "Type error: {}", msg),
            Self::RangeError(msg) => write!(f, "Range error: {}", msg),
            Self::ReferenceError(msg) => write!(f, "Reference error: {}", msg),
            Self::Timeout => write!(f, "Script execution timed out"),
            Self::OutOfMemory => write!(f, "Out of memory"),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Self::ModuleNotFound(msg) => write!(f, "Module not found: {}", msg),
        }
    }
}

pub trait CommonError: fmt::Display + fmt::Debug {
    fn kind(&self) -> CommonErrorKind;
    fn format_stack_trace(&self) -> Option<String> {
        None
    }
    fn to_formatted_string(&self) -> String {
        let mut msg = self.to_string();
        if let Some(trace) = self.format_stack_trace() {
            msg.push_str("\nStack trace:\n");
            msg.push_str(&trace);
        }
        msg
    }
}

#[derive(Debug)]
pub struct DefaultEngineError {
    kind: CommonErrorKind,
    stack_trace: Option<String>,
}

impl DefaultEngineError {
    pub fn new(kind: CommonErrorKind) -> Self {
        Self {
            kind,
            stack_trace: None,
        }
    }

    pub fn with_trace(mut self, trace: String) -> Self {
        self.stack_trace = Some(trace);
        self
    }
}

impl fmt::Display for DefaultEngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl CommonError for DefaultEngineError {
    fn kind(&self) -> CommonErrorKind {
        self.kind.clone()
    }
    fn format_stack_trace(&self) -> Option<String> {
        self.stack_trace.clone()
    }
}

impl std::error::Error for DefaultEngineError {}
