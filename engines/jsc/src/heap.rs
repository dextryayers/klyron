#[cfg(feature = "native")]
use crate::ffi;
pub struct JSCHeap {
    #[cfg(feature = "native")]
    engine: Option<crate::ffi::JSCEnginePtr>,
}

impl JSCHeap {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "native")]
            engine: None,
        }
    }

    #[cfg(feature = "native")]
    pub fn with_engine(engine: &crate::ffi::JSCEnginePtr) -> Self {
        Self { engine: Some(unsafe { std::ptr::read(engine as *const _) }) }
    }

    #[cfg(feature = "native")]
    pub fn get_stats(&self) -> Result<crate::HeapStats, String> {
        match &self.engine {
            Some(eng) => {
                let raw = eng.get_heap_stats()?;
                Ok(crate::HeapStats {
                    total_heap_size: raw.total_heap_size as u64,
                    total_heap_size_executable: raw.total_heap_size_executable as u64,
                    total_physical_size: raw.total_physical_size as u64,
                    total_available_size: raw.total_available_size as u64,
                    used_heap_size: raw.used_heap_size as u64,
                    heap_size_limit: raw.heap_size_limit as u64,
                    malloced_memory: raw.malloced_memory as u64,
                    peak_malloced_memory: raw.peak_malloced_memory as u64,
                    number_of_native_contexts: raw.number_of_native_contexts as u64,
                    number_of_detached_contexts: raw.number_of_detached_contexts as u64,
                    total_global_handles_size: raw.total_global_handles_size as u64,
                    used_global_handles_size: raw.used_global_handles_size as u64,
                    external_memory: raw.external_memory as u64,
                })
            }
            None => Err("heap: no engine handle".into()),
        }
    }

    #[cfg(feature = "native")]
    pub fn request_gc(&self) {
        if let Some(eng) = &self.engine {
            eng.request_gc()
        }
    }

    #[cfg(feature = "native")]
    pub fn low_memory_notification(&self) {
        if let Some(eng) = &self.engine {
            eng.low_memory_notification()
        }
    }
}

impl Default for JSCHeap {
    fn default() -> Self {
        Self::new()
    }
}
