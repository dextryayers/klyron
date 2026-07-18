use crate::error::V8Error;

#[cfg(feature = "native")]
use crate::ffi;
#[cfg(feature = "native")]
use std::ffi::CString;

pub struct V8Context {
    #[cfg(feature = "native")]
    handle: *mut ffi::V8ContextHandle,
    #[cfg(feature = "native")]
    owns: bool,
}

impl V8Context {
    #[cfg(feature = "native")]
    pub fn new(isolate: *mut ffi::V8IsolateHandle) -> Result<Self, V8Error> {
        let ctx = unsafe { ffi::klyron_v8_context_new(isolate) };
        if ctx.is_null() {
            Err(V8Error::InitFailed("context creation failed".into()))
        } else {
            Ok(Self { handle: ctx, owns: true })
        }
    }

    #[cfg(not(feature = "native"))]
    pub fn new(_isolate: *mut std::ffi::c_void) -> Result<Self, V8Error> {
        Err(V8Error::NotInitialized)
    }

    #[cfg(feature = "native")]
    pub fn from_snapshot(isolate: *mut ffi::V8IsolateHandle, snapshot: *mut ffi::V8SnapshotHandle) -> Result<Self, V8Error> {
        let ctx = unsafe { ffi::klyron_v8_context_new_from_snapshot(isolate, snapshot) };
        if ctx.is_null() {
            Err(V8Error::InitFailed("context from snapshot failed".into()))
        } else {
            Ok(Self { handle: ctx, owns: true })
        }
    }

    #[cfg(feature = "native")]
    pub fn handle(&self) -> *mut ffi::V8ContextHandle {
        self.handle
    }

    #[cfg(feature = "native")]
    pub fn enter(&self) {
        unsafe { ffi::klyron_v8_context_enter(self.handle) }
    }

    #[cfg(feature = "native")]
    pub fn exit(&self) {
        unsafe { ffi::klyron_v8_context_exit(self.handle) }
    }

    #[cfg(feature = "native")]
    pub fn eval(&self, source: &str) -> Result<String, V8Error> {
        let c_source = CString::new(source).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let r = unsafe { ffi::klyron_v8_eval(self.handle, c_source.as_ptr(), std::ptr::null()) };
        Self::check_string(r)
    }

    #[cfg(feature = "native")]
    fn check_string(result: ffi::V8StringResult) -> Result<String, V8Error> {
        use std::ffi::CStr;
        if result.success {
            let s = if result.data.is_null() {
                String::new()
            } else {
                let s = unsafe { CStr::from_ptr(result.data).to_string_lossy().into() };
                unsafe { ffi::klyron_v8_free_string(result.data) };
                s
            };
            Ok(s)
        } else {
            let err = unsafe { CStr::from_ptr(result.error.as_ptr()).to_string_lossy().into() };
            Err(V8Error::EvalFailed(err))
        }
    }
}

#[cfg(feature = "native")]
impl Drop for V8Context {
    fn drop(&mut self) {
        if self.owns && !self.handle.is_null() {
            unsafe { ffi::klyron_v8_context_dispose(self.handle) }
        }
    }
}
