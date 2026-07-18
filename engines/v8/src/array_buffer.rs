use crate::error::V8Error;

#[cfg(feature = "native")]
use crate::ffi;

pub struct V8ArrayBuffer {
    #[cfg(feature = "native")]
    handle: *mut ffi::V8ValueHandle,
}

impl V8ArrayBuffer {
    #[cfg(feature = "native")]
    pub fn new(ctx: *mut ffi::V8ContextHandle, data: &[u8]) -> Result<Self, V8Error> {
        let handle = unsafe { ffi::klyron_v8_array_buffer_new(ctx, data.as_ptr(), data.len()) };
        if handle.is_null() { Err(V8Error::EvalFailed("array_buffer_new failed".into())) }
        else { Ok(Self { handle }) }
    }

    #[cfg(not(feature = "native"))]
    pub fn new(_ctx: *mut std::ffi::c_void, _data: &[u8]) -> Result<Self, V8Error> {
        Err(V8Error::NotInitialized)
    }

    #[cfg(feature = "native")]
    pub fn handle(&self) -> *mut ffi::V8ValueHandle {
        self.handle
    }

    #[cfg(feature = "native")]
    pub fn from_value(value: *mut ffi::V8ValueHandle) -> Self {
        Self { handle: value }
    }
}

#[cfg(feature = "native")]
impl Drop for V8ArrayBuffer {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { ffi::klyron_v8_value_dispose(self.handle) }
        }
    }
}
