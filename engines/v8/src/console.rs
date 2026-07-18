
#[cfg(feature = "native")]
use crate::ffi;
#[cfg(feature = "native")]
use std::ffi::CString;

pub struct V8Console {
    #[cfg(feature = "native")]
    ctx: *mut ffi::V8ContextHandle,
    #[cfg(feature = "native")]
    handle: Option<*mut ffi::V8ValueHandle>,
}

impl V8Console {
    #[cfg(feature = "native")]
    pub fn new(ctx: *mut ffi::V8ContextHandle) -> Self {
        let handle = unsafe { ffi::klyron_v8_console_new(ctx) };
        Self { ctx, handle: Some(handle) }
    }

    #[cfg(not(feature = "native"))]
    pub fn new(_ctx: *mut std::ffi::c_void) -> Self {
        Self {}
    }

    #[cfg(feature = "native")]
    pub fn log(&self, msg: &str) {
        if let Ok(c) = CString::new(msg) {
            unsafe { ffi::klyron_v8_console_log(self.ctx, c.as_ptr()) }
        }
    }

    #[cfg(feature = "native")]
    pub fn warn(&self, msg: &str) {
        if let Ok(c) = CString::new(msg) {
            unsafe { ffi::klyron_v8_console_warn(self.ctx, c.as_ptr()) }
        }
    }

    #[cfg(feature = "native")]
    pub fn error(&self, msg: &str) {
        if let Ok(c) = CString::new(msg) {
            unsafe { ffi::klyron_v8_console_error(self.ctx, c.as_ptr()) }
        }
    }

    #[cfg(feature = "native")]
    pub fn info(&self, msg: &str) {
        if let Ok(c) = CString::new(msg) {
            unsafe { ffi::klyron_v8_console_info(self.ctx, c.as_ptr()) }
        }
    }

    #[cfg(feature = "native")]
    pub fn debug(&self, msg: &str) {
        if let Ok(c) = CString::new(msg) {
            unsafe { ffi::klyron_v8_console_debug(self.ctx, c.as_ptr()) }
        }
    }

    #[cfg(feature = "native")]
    pub fn handle(&self) -> Option<*mut ffi::V8ValueHandle> {
        self.handle
    }
}
