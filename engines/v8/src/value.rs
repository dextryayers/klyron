
#[cfg(feature = "native")]
use crate::error::V8Error;
#[cfg(feature = "native")]
use crate::ffi;
#[cfg(feature = "native")]
use std::ffi::CString;

pub struct V8Value {
    #[cfg(feature = "native")]
    handle: *mut ffi::V8ValueHandle,
}

impl V8Value {
    #[cfg(feature = "native")]
    pub fn from_handle(handle: *mut ffi::V8ValueHandle) -> Self {
        Self { handle }
    }

    #[cfg(feature = "native")]
    pub fn handle(&self) -> *mut ffi::V8ValueHandle {
        self.handle
    }

    #[cfg(feature = "native")]
    pub fn new_string(ctx: *mut ffi::V8ContextHandle, s: &str) -> Result<Self, V8Error> {
        let c = CString::new(s).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let ptr = unsafe { ffi::klyron_v8_value_new_string(ctx, c.as_ptr()) };
        if ptr.is_null() { Err(V8Error::EvalFailed("new_string failed".into())) }
        else { Ok(Self { handle: ptr }) }
    }

    #[cfg(feature = "native")]
    pub fn new_number(ctx: *mut ffi::V8ContextHandle, n: f64) -> Self {
        Self { handle: unsafe { ffi::klyron_v8_value_new_number(ctx, n) } }
    }

    #[cfg(feature = "native")]
    pub fn new_bool(ctx: *mut ffi::V8ContextHandle, v: bool) -> Self {
        Self { handle: unsafe { ffi::klyron_v8_value_new_bool(ctx, v) } }
    }

    #[cfg(feature = "native")]
    pub fn new_null(ctx: *mut ffi::V8ContextHandle) -> Self {
        Self { handle: unsafe { ffi::klyron_v8_value_new_null(ctx) } }
    }

    #[cfg(feature = "native")]
    pub fn new_undefined(ctx: *mut ffi::V8ContextHandle) -> Self {
        Self { handle: unsafe { ffi::klyron_v8_value_new_undefined(ctx) } }
    }

    #[cfg(feature = "native")]
    pub fn new_object(ctx: *mut ffi::V8ContextHandle) -> Self {
        Self { handle: unsafe { ffi::klyron_v8_value_new_object(ctx) } }
    }

    #[cfg(feature = "native")]
    pub fn new_array(ctx: *mut ffi::V8ContextHandle) -> Self {
        Self { handle: unsafe { ffi::klyron_v8_value_new_array(ctx) } }
    }

    #[cfg(feature = "native")]
    pub fn value_type(&self, ctx: *mut ffi::V8ContextHandle) -> ffi::V8TypeResult {
        unsafe { ffi::klyron_v8_value_typeof(ctx, self.handle) }
    }

    #[cfg(feature = "native")]
    pub fn to_string(&self, ctx: *mut ffi::V8ContextHandle) -> Result<String, V8Error> {
        let r = unsafe { ffi::klyron_v8_value_to_string(ctx, self.handle) };
        if r.success {
            let s = if r.data.is_null() { String::new() }
            else {
                let s = unsafe { std::ffi::CStr::from_ptr(r.data).to_string_lossy().into() };
                unsafe { ffi::klyron_v8_free_string(r.data) };
                s
            };
            Ok(s)
        } else {
            Err(V8Error::EvalFailed("to_string failed".into()))
        }
    }

    #[cfg(feature = "native")]
    pub fn to_number(&self, ctx: *mut ffi::V8ContextHandle) -> f64 {
        unsafe { ffi::klyron_v8_value_to_number(ctx, self.handle) }
    }

    #[cfg(feature = "native")]
    pub fn to_bool(&self, ctx: *mut ffi::V8ContextHandle) -> bool {
        unsafe { ffi::klyron_v8_value_to_bool(ctx, self.handle) }
    }

    #[cfg(feature = "native")]
    pub fn is_array(&self, ctx: *mut ffi::V8ContextHandle) -> bool {
        unsafe { ffi::klyron_v8_value_is_array(ctx, self.handle) }
    }
}

#[cfg(feature = "native")]
impl Drop for V8Value {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { ffi::klyron_v8_value_dispose(self.handle) }
        }
    }
}
