use crate::error::V8Error;

#[cfg(feature = "native")]
use crate::ffi;

pub struct V8Isolate {
    #[cfg(feature = "native")]
    handle: *mut ffi::V8IsolateHandle,
    #[cfg(feature = "native")]
    owns: bool,
}

impl V8Isolate {
    #[cfg(feature = "native")]
    pub fn new() -> Result<Self, V8Error> {
        let iso = unsafe { ffi::klyron_v8_isolate_new() };
        if iso.is_null() {
            Err(V8Error::InitFailed("isolate creation failed".into()))
        } else {
            Ok(Self { handle: iso, owns: true })
        }
    }

    #[cfg(not(feature = "native"))]
    pub fn new() -> Result<Self, V8Error> {
        Err(V8Error::NotInitialized)
    }

    #[cfg(feature = "native")]
    pub fn from_raw(handle: *mut ffi::V8IsolateHandle) -> Self {
        Self { handle, owns: false }
    }

    #[cfg(feature = "native")]
    pub fn handle(&self) -> *mut ffi::V8IsolateHandle {
        self.handle
    }

    #[cfg(feature = "native")]
    pub fn enter(&self) {
        unsafe { ffi::klyron_v8_isolate_enter(self.handle) }
    }

    #[cfg(feature = "native")]
    pub fn exit(&self) {
        unsafe { ffi::klyron_v8_isolate_exit(self.handle) }
    }

    #[cfg(feature = "native")]
    pub fn low_memory_notification(&self) {
        unsafe { ffi::klyron_v8_low_memory_notification(self.handle) }
    }

    #[cfg(feature = "native")]
    pub fn idle_notification(&self, deadline: f64) {
        unsafe { ffi::klyron_v8_idle_notification(self.handle, deadline) }
    }

    #[cfg(feature = "native")]
    pub fn request_gc(&self) {
        unsafe { ffi::klyron_v8_request_gc(self.handle) }
    }

    #[cfg(feature = "native")]
    pub fn get_heap_stats(&self) -> Result<ffi::V8HeapStats, V8Error> {
        let mut stats = ffi_types::V8HeapStats::zeroed();
        let r = unsafe { ffi::klyron_v8_get_heap_stats(self.handle, &mut stats) };
        if r.success { Ok(stats) } else { Err(V8Error::Internal("get_heap_stats failed".into())) }
    }
}

#[cfg(feature = "native")]
impl Drop for V8Isolate {
    fn drop(&mut self) {
        if self.owns && !self.handle.is_null() {
            unsafe { ffi::klyron_v8_isolate_dispose(self.handle) }
        }
    }
}
