#[cfg(feature = "native")]
use crate::ffi;

pub struct JSCContext {
    #[cfg(feature = "native")]
    inner: *mut ffi::JSCEngineHandle,
}

impl JSCContext {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "native")]
            inner: std::ptr::null_mut(),
        }
    }

    #[cfg(feature = "native")]
    pub fn from_handle(handle: *mut ffi::JSCEngineHandle) -> Self {
        Self { inner: handle }
    }

    pub fn enter(&self) {
        #[cfg(feature = "native")]
        if !self.inner.is_null() {
            unsafe { ffi::klyron_jsc_context_enter(self.inner) }
        }
    }

    pub fn exit(&self) {
        #[cfg(feature = "native")]
        if !self.inner.is_null() {
            unsafe { ffi::klyron_jsc_context_exit(self.inner) }
        }
    }
}

impl Default for JSCContext {
    fn default() -> Self {
        Self::new()
    }
}
