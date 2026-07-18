use crate::error::V8Error;

#[cfg(feature = "native")]
use crate::ffi;

pub struct RuntimeConfig {
    pub icu_data_path: Option<String>,
    pub snapshot_blob_path: Option<String>,
    pub max_heap_size_mb: usize,
    pub initial_heap_size_mb: usize,
    pub array_buffer_allocator_pool_size: u32,
    pub use_shared_memory: bool,
    pub expose_gc: bool,
    pub single_threaded: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            icu_data_path: None,
            snapshot_blob_path: None,
            max_heap_size_mb: 0,
            initial_heap_size_mb: 0,
            array_buffer_allocator_pool_size: 0,
            use_shared_memory: false,
            expose_gc: true,
            single_threaded: true,
        }
    }
}

pub struct Runtime {
    #[cfg(feature = "native")]
    inner: ffi::V8EnginePtr,
}

impl Runtime {
    #[cfg(feature = "native")]
    pub fn new(_config: Option<RuntimeConfig>) -> Result<Self, V8Error> {
        ffi::V8EnginePtr::init().map(|inner| Self { inner }).map_err(|e| V8Error::InitFailed(e))
    }

    #[cfg(not(feature = "native"))]
    pub fn new(_config: Option<RuntimeConfig>) -> Result<Self, V8Error> {
        Err(V8Error::NotInitialized)
    }

    #[cfg(feature = "native")]
    pub fn engine(&self) -> &ffi::V8EnginePtr {
        &self.inner
    }

    pub fn version() -> String {
        #[cfg(feature = "native")]
        { ffi::V8EnginePtr::version() }
        #[cfg(not(feature = "native"))]
        { "V8 (not available)".into() }
    }
}
