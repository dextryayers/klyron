use crate::error::V8Error;

#[cfg(feature = "native")]
use crate::ffi;
#[cfg(feature = "native")]
use std::ffi::CString;

#[cfg(feature = "native")]
pub type NativeCallback = unsafe extern "C" fn(
    ctx: *mut ffi::V8ContextHandle,
    argc: i32,
    argv: *mut *mut ffi::V8ValueHandle,
    user_data: *mut std::ffi::c_void,
    result: *mut *mut ffi::V8ValueHandle,
);

pub struct V8Function {
    #[cfg(feature = "native")]
    handle: *mut ffi::V8ValueHandle,
    #[cfg(feature = "native")]
    owns: bool,
}

impl V8Function {
    #[cfg(feature = "native")]
    pub fn new(ctx: *mut ffi::V8ContextHandle, name: Option<&str>, callback: Option<NativeCallback>, user_data: *mut std::ffi::c_void) -> Result<Self, V8Error> {
        let c_name = name.map(|n| CString::new(n)).transpose().map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let handle = unsafe {
            ffi::klyron_v8_function_new(
                ctx,
                c_name.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
                callback,
                user_data,
            )
        };
        if handle.is_null() { Err(V8Error::EvalFailed("function_new failed".into())) }
        else { Ok(Self { handle, owns: true }) }
    }

    #[cfg(not(feature = "native"))]
    pub fn new(_ctx: *mut std::ffi::c_void, _name: Option<&str>, _callback: Option<extern "C" fn(*mut std::ffi::c_void, i32, *mut *mut std::ffi::c_void, *mut std::ffi::c_void, *mut *mut std::ffi::c_void)>, _user_data: *mut std::ffi::c_void) -> Result<Self, V8Error> {
        Err(V8Error::NotInitialized)
    }

    #[cfg(feature = "native")]
    pub fn handle(&self) -> *mut ffi::V8ValueHandle {
        self.handle
    }
}

#[cfg(feature = "native")]
impl Drop for V8Function {
    fn drop(&mut self) {
        if self.owns && !self.handle.is_null() {
            unsafe { ffi::klyron_v8_value_dispose(self.handle) }
        }
    }
}
