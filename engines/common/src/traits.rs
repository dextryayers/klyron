use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCapabilities {
    pub supports_modules: bool,
    pub supports_jsx: bool,
    pub supports_ts: bool,
    pub supports_snapshots: bool,
    pub supports_wasm: bool,
    pub supports_debugger: bool,
    pub max_heap_size: usize,
    pub max_stack_size: usize,
}

impl Default for EngineCapabilities {
    fn default() -> Self {
        Self {
            supports_modules: true,
            supports_jsx: false,
            supports_ts: false,
            supports_snapshots: false,
            supports_wasm: false,
            supports_debugger: false,
            max_heap_size: 512 * 1024 * 1024,
            max_stack_size: 1024 * 1024,
        }
    }
}

impl EngineCapabilities {
    pub fn v8() -> Self {
        Self {
            supports_modules: true,
            supports_jsx: true,
            supports_ts: true,
            supports_snapshots: true,
            supports_wasm: true,
            supports_debugger: true,
            max_heap_size: 2 * 1024 * 1024 * 1024,
            max_stack_size: 4 * 1024 * 1024,
        }
    }

    pub fn quickjs() -> Self {
        Self {
            supports_modules: true,
            supports_jsx: false,
            supports_ts: false,
            supports_snapshots: true,
            supports_wasm: false,
            supports_debugger: false,
            max_heap_size: 512 * 1024 * 1024,
            max_stack_size: 1024 * 1024,
        }
    }

    pub fn jsc() -> Self {
        Self {
            supports_modules: true,
            supports_jsx: true,
            supports_ts: true,
            supports_snapshots: false,
            supports_wasm: true,
            supports_debugger: true,
            max_heap_size: 1024 * 1024 * 1024,
            max_stack_size: 2 * 1024 * 1024,
        }
    }

    pub fn boa() -> Self {
        Self {
            supports_modules: true,
            supports_jsx: false,
            supports_ts: false,
            supports_snapshots: true,
            supports_wasm: false,
            supports_debugger: false,
            max_heap_size: 256 * 1024 * 1024,
            max_stack_size: 512 * 1024,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub memory_limit: Option<usize>,
    pub time_limit: Option<std::time::Duration>,
    pub stack_size: Option<usize>,
    pub snapshot_path: Option<std::path::PathBuf>,
    pub enable_debugger: bool,
    pub enable_wasm: bool,
    pub enable_jit: bool,
    pub cache_enabled: bool,
    pub cache_ttl_secs: u64,
    pub cache_max_size_mb: u64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            memory_limit: None,
            time_limit: None,
            stack_size: None,
            snapshot_path: None,
            enable_debugger: false,
            enable_wasm: false,
            enable_jit: true,
            cache_enabled: true,
            cache_ttl_secs: 3600,
            cache_max_size_mb: 512,
        }
    }
}

pub type EngineResult<T> = Result<T, EngineError>;

#[derive(Debug, Clone)]
pub enum EngineError {
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
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
            Self::EngineBusy => write!(f, "Engine is busy"),
            Self::EnginePoolExhausted => write!(f, "Engine pool exhausted"),
            Self::SnapshotError(msg) => write!(f, "Snapshot error: {}", msg),
            Self::CacheError(msg) => write!(f, "Cache error: {}", msg),
        }
    }
}

impl std::error::Error for EngineError {}
