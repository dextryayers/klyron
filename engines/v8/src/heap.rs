
#[cfg(feature = "native")]
use crate::error::V8Error;
#[cfg(feature = "native")]
use crate::ffi_types;

#[cfg(feature = "native")]
use crate::ffi;

pub struct HeapManager {
    #[cfg(feature = "native")]
    isolate: *mut ffi::V8IsolateHandle,
}

impl HeapManager {
    #[cfg(feature = "native")]
    pub fn new(isolate: *mut ffi::V8IsolateHandle) -> Self {
        Self { isolate }
    }

    #[cfg(not(feature = "native"))]
    pub fn new(_isolate: *mut std::ffi::c_void) -> Self {
        Self {}
    }

    #[cfg(feature = "native")]
    pub fn get_stats(&self) -> Result<ffi_types::V8HeapStats, V8Error> {
        let mut stats = ffi_types::V8HeapStats::zeroed();
        let r = unsafe { ffi::klyron_v8_get_heap_stats(self.isolate, &mut stats) };
        if r.success { Ok(stats) }
        else { Err(V8Error::Internal("get_heap_stats failed".into())) }
    }

    #[cfg(feature = "native")]
    pub fn low_memory_notification(&self) {
        unsafe { ffi::klyron_v8_low_memory_notification(self.isolate) }
    }

    #[cfg(feature = "native")]
    pub fn idle_notification(&self, deadline: f64) {
        unsafe { ffi::klyron_v8_idle_notification(self.isolate, deadline) }
    }

    #[cfg(feature = "native")]
    pub fn set_memory_pressure(&self, pressure: u32) {
        unsafe { ffi::klyron_v8_set_memory_pressure(self.isolate, pressure) }
    }

    #[cfg(feature = "native")]
    pub fn request_gc(&self) {
        unsafe { ffi::klyron_v8_request_gc(self.isolate) }
    }

    #[cfg(feature = "native")]
    pub fn get_malloced_memory(&self) -> usize {
        unsafe { ffi::klyron_v8_get_malloced_memory(self.isolate) }
    }

    #[cfg(feature = "native")]
    pub fn adjust_external_memory(&self, change: i64) -> usize {
        unsafe { ffi::klyron_v8_adjust_external_memory(self.isolate, change) }
    }
}
