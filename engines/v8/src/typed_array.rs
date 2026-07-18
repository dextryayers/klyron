use crate::error::V8Error;

#[cfg(feature = "native")]
use crate::ffi;
#[cfg(feature = "native")]
use std::ffi::CString;

pub struct V8TypedArray {
    #[cfg(feature = "native")]
    ctx: *mut ffi::V8ContextHandle,
    #[cfg(feature = "native")]
    handle: *mut ffi::V8ValueHandle,
}

impl V8TypedArray {
    #[cfg(feature = "native")]
    pub fn new(ctx: *mut ffi::V8ContextHandle, type_name: &str, length: usize) -> Result<Self, V8Error> {
        let c = CString::new(type_name).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let handle = unsafe { ffi::klyron_v8_typed_array_new(ctx, c.as_ptr(), length) };
        if handle.is_null() { Err(V8Error::EvalFailed("typed_array_new failed".into())) }
        else { Ok(Self { ctx, handle }) }
    }

    #[cfg(not(feature = "native"))]
    pub fn new(_ctx: *mut std::ffi::c_void, _type_name: &str, _length: usize) -> Result<Self, V8Error> {
        Err(V8Error::NotInitialized)
    }

    #[cfg(feature = "native")]
    pub fn handle(&self) -> *mut ffi::V8ValueHandle {
        self.handle
    }

    #[cfg(feature = "native")]
    pub fn length(&self) -> usize {
        unsafe { ffi::klyron_v8_typed_array_get_length(self.ctx, self.handle) }
    }

    #[cfg(feature = "native")]
    pub fn buffer(&self) -> Result<*mut ffi::V8ValueHandle, V8Error> {
        let buf = unsafe { ffi::klyron_v8_typed_array_get_buffer(self.ctx, self.handle) };
        if buf.is_null() { Err(V8Error::EvalFailed("typed_array_get_buffer failed".into())) }
        else { Ok(buf) }
    }

    #[cfg(feature = "native")]
    pub fn array_type(&self) -> u32 {
        unsafe { ffi::klyron_v8_get_typed_array_type(self.ctx, self.handle) }
    }
}

#[cfg(feature = "native")]
impl Drop for V8TypedArray {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { ffi::klyron_v8_value_dispose(self.handle) }
        }
    }
}
