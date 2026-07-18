use crate::error::V8Error;

#[cfg(feature = "native")]
use crate::ffi;
#[cfg(feature = "native")]
use std::ffi::CString;

pub struct V8Script {
    #[cfg(feature = "native")]
    handle: *mut ffi::V8ScriptHandle,
}

impl V8Script {
    #[cfg(feature = "native")]
    pub fn compile(ctx: *mut ffi::V8ContextHandle, source: &str, filename: Option<&str>) -> Result<Self, V8Error> {
        let c_source = CString::new(source).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let c_file = filename.map(|f| CString::new(f)).transpose().map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let ptr = unsafe {
            ffi::klyron_v8_compile(ctx, c_source.as_ptr(), c_file.as_ref().map_or(std::ptr::null(), |f| f.as_ptr()))
        };
        if ptr.is_null() {
            Err(V8Error::EvalFailed("compile failed".into()))
        } else {
            Ok(Self { handle: ptr })
        }
    }

    #[cfg(not(feature = "native"))]
    pub fn compile(_ctx: *mut std::ffi::c_void, _source: &str, _filename: Option<&str>) -> Result<Self, V8Error> {
        Err(V8Error::NotInitialized)
    }

    #[cfg(feature = "native")]
    pub fn run(&self, ctx: *mut ffi::V8ContextHandle) -> Result<String, V8Error> {
        let r = unsafe { ffi::klyron_v8_run(ctx, self.handle) };
        if r.success {
            let s = if r.data.is_null() {
                String::new()
            } else {
                let s = unsafe { std::ffi::CStr::from_ptr(r.data).to_string_lossy().into() };
                unsafe { ffi::klyron_v8_free_string(r.data) };
                s
            };
            Ok(s)
        } else {
            let err = unsafe { std::ffi::CStr::from_ptr(r.error.as_ptr()).to_string_lossy().into() };
            Err(V8Error::EvalFailed(err))
        }
    }
}

#[cfg(feature = "native")]
impl Drop for V8Script {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { ffi::klyron_v8_script_dispose(self.handle) }
        }
    }
}
