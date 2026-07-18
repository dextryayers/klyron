use crate::error::V8Error;

#[cfg(feature = "native")]
use crate::ffi;

pub struct V8Promise {
    #[cfg(feature = "native")]
    handle: *mut ffi::V8PromiseHandle,
}

impl V8Promise {
    #[cfg(feature = "native")]
    pub fn new(ctx: *mut ffi::V8ContextHandle) -> Result<Self, V8Error> {
        let ptr = unsafe { ffi::klyron_v8_promise_new(ctx) };
        if ptr.is_null() { Err(V8Error::CallFailed("promise creation failed".into())) }
        else { Ok(Self { handle: ptr }) }
    }

    #[cfg(not(feature = "native"))]
    pub fn new(_ctx: *mut std::ffi::c_void) -> Result<Self, V8Error> {
        Err(V8Error::NotInitialized)
    }

    #[cfg(feature = "native")]
    pub fn resolve(&self, ctx: *mut ffi::V8ContextHandle, value: *mut ffi::V8ValueHandle) -> Result<(), V8Error> {
        let r = unsafe { ffi::klyron_v8_promise_resolve(ctx, self.handle, value) };
        if r.success { Ok(()) }
        else { Err(V8Error::CallFailed("promise resolve failed".into())) }
    }

    #[cfg(feature = "native")]
    pub fn reject(&self, ctx: *mut ffi::V8ContextHandle, reason: &str) -> Result<(), V8Error> {
        use std::ffi::CString;
        let c = CString::new(reason).map_err(|e| V8Error::CallFailed(e.to_string()))?;
        let r = unsafe { ffi::klyron_v8_promise_reject(ctx, self.handle, c.as_ptr()) };
        if r.success { Ok(()) }
        else { Err(V8Error::CallFailed("promise reject failed".into())) }
    }

    #[cfg(feature = "native")]
    pub fn state(&self, ctx: *mut ffi::V8ContextHandle) -> crate::V8PromiseState {
        let s = unsafe { ffi::klyron_v8_promise_get_state(ctx, self.handle) };
        match s {
            1 => crate::V8PromiseState::Fulfilled,
            2 => crate::V8PromiseState::Rejected,
            _ => crate::V8PromiseState::Pending,
        }
    }

    #[cfg(feature = "native")]
    pub fn has_handler(&self, ctx: *mut ffi::V8ContextHandle) -> bool {
        unsafe { ffi::klyron_v8_promise_has_handler(ctx, self.handle) }
    }

    #[cfg(feature = "native")]
    pub fn mark_as_handled(&self, ctx: *mut ffi::V8ContextHandle) -> Result<(), V8Error> {
        let r = unsafe { ffi::klyron_v8_promise_mark_as_handled(ctx, self.handle) };
        if r.success { Ok(()) }
        else { Err(V8Error::CallFailed("mark_as_handled failed".into())) }
    }

    #[cfg(feature = "native")]
    pub fn handle(&self) -> *mut ffi::V8PromiseHandle {
        self.handle
    }
}
