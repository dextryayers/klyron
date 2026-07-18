use crate::error::V8Error;

#[cfg(feature = "native")]
use crate::ffi;
#[cfg(feature = "native")]
use std::ffi::CString;

pub struct V8Module {
    #[cfg(feature = "native")]
    handle: *mut ffi::V8ModuleHandle,
}

impl V8Module {
    #[cfg(feature = "native")]
    pub fn compile(ctx: *mut ffi::V8ContextHandle, source: &str, origin: Option<&str>) -> Result<Self, V8Error> {
        let c_source = CString::new(source).map_err(|e| V8Error::ModuleFailed(e.to_string()))?;
        let c_origin = origin.map(|o| CString::new(o)).transpose().map_err(|e| V8Error::ModuleFailed(e.to_string()))?;
        let ptr = unsafe {
            ffi::klyron_v8_module_compile(ctx, c_source.as_ptr(), c_origin.as_ref().map_or(std::ptr::null(), |o| o.as_ptr()))
        };
        if ptr.is_null() { Err(V8Error::ModuleFailed("compile failed".into())) }
        else { Ok(Self { handle: ptr }) }
    }

    #[cfg(not(feature = "native"))]
    pub fn compile(_ctx: *mut std::ffi::c_void, _source: &str, _origin: Option<&str>) -> Result<Self, V8Error> {
        Err(V8Error::NotInitialized)
    }

    #[cfg(feature = "native")]
    pub fn instantiate(&self, ctx: *mut ffi::V8ContextHandle) -> Result<(), V8Error> {
        let r = unsafe { ffi::klyron_v8_module_instantiate(ctx, self.handle) };
        if r.success { Ok(()) }
        else { Err(V8Error::ModuleFailed("instantiate failed".into())) }
    }

    #[cfg(feature = "native")]
    pub fn evaluate(&self, ctx: *mut ffi::V8ContextHandle) -> Result<String, V8Error> {
        let r = unsafe { ffi::klyron_v8_module_evaluate(ctx, self.handle) };
        if r.success {
            let s = if r.data.is_null() { String::new() }
            else {
                let s = unsafe { std::ffi::CStr::from_ptr(r.data).to_string_lossy().into() };
                unsafe { ffi::klyron_v8_free_string(r.data) };
                s
            };
            Ok(s)
        } else {
            Err(V8Error::ModuleFailed("evaluate failed".into()))
        }
    }

    #[cfg(feature = "native")]
    pub fn identity(&self, ctx: *mut ffi::V8ContextHandle) -> i32 {
        unsafe { ffi::klyron_v8_module_get_identity(ctx, self.handle) }
    }
}

#[cfg(feature = "native")]
impl Drop for V8Module {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { ffi::klyron_v8_module_dispose(self.handle) }
        }
    }
}
