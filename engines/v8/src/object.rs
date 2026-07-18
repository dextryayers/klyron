
#[cfg(feature = "native")]
use crate::error::V8Error;
#[cfg(feature = "native")]
use crate::ffi;
#[cfg(feature = "native")]
use std::ffi::CString;

pub struct V8Object {
    #[cfg(feature = "native")]
    ctx: *mut ffi::V8ContextHandle,
    #[cfg(feature = "native")]
    handle: *mut ffi::V8ValueHandle,
}

impl V8Object {
    #[cfg(feature = "native")]
    pub fn new(ctx: *mut ffi::V8ContextHandle) -> Self {
        Self { ctx, handle: unsafe { ffi::klyron_v8_value_new_object(ctx) } }
    }

    #[cfg(not(feature = "native"))]
    pub fn new(_ctx: *mut std::ffi::c_void) -> Self {
        Self {}
    }

    #[cfg(feature = "native")]
    pub fn from_value(handle: *mut ffi::V8ValueHandle) -> Self {
        Self { ctx: std::ptr::null_mut(), handle }
    }

    #[cfg(feature = "native")]
    pub fn handle(&self) -> *mut ffi::V8ValueHandle {
        self.handle
    }

    #[cfg(feature = "native")]
    pub fn set(&self, name: &str, value: *mut ffi::V8ValueHandle) -> Result<(), V8Error> {
        let c = CString::new(name).map_err(|e| V8Error::GlobalFailed(e.to_string()))?;
        let r = unsafe { ffi::klyron_v8_object_set_property(self.ctx, self.handle, c.as_ptr(), value) };
        if r.success { Ok(()) }
        else { Err(V8Error::GlobalFailed("set property failed".into())) }
    }

    #[cfg(feature = "native")]
    pub fn get(&self, name: &str) -> Result<*mut ffi::V8ValueHandle, V8Error> {
        let c = CString::new(name).map_err(|e| V8Error::GlobalFailed(e.to_string()))?;
        let ptr = unsafe { ffi::klyron_v8_object_get_property(self.ctx, self.handle, c.as_ptr()) };
        if ptr.is_null() { Err(V8Error::GlobalFailed("get property failed".into())) }
        else { Ok(ptr) }
    }

    #[cfg(feature = "native")]
    pub fn set_bool(&self, name: &str, val: bool) -> Result<(), V8Error> {
        let v = unsafe { ffi::klyron_v8_value_new_bool(self.ctx, val) };
        self.set(name, v)
    }

    #[cfg(feature = "native")]
    pub fn set_number(&self, name: &str, val: f64) -> Result<(), V8Error> {
        let v = unsafe { ffi::klyron_v8_value_new_number(self.ctx, val) };
        self.set(name, v)
    }

    #[cfg(feature = "native")]
    pub fn set_string(&self, name: &str, val: &str) -> Result<(), V8Error> {
        let c = CString::new(val).map_err(|e| V8Error::GlobalFailed(e.to_string()))?;
        let v = unsafe { ffi::klyron_v8_value_new_string(self.ctx, c.as_ptr()) };
        if v.is_null() { return Err(V8Error::GlobalFailed("new_string failed".into())) }
        self.set(name, v)
    }
}

#[cfg(feature = "native")]
impl Drop for V8Object {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { ffi::klyron_v8_value_dispose(self.handle) }
        }
    }
}
