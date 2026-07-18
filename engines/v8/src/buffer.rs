use crate::error::V8Error;

#[cfg(feature = "native")]
use crate::ffi;
#[cfg(feature = "native")]
use std::ffi::CString;

pub struct V8Buffer {
    #[cfg(feature = "native")]
    ctx: *mut ffi::V8ContextHandle,
    #[cfg(feature = "native")]
    handle: *mut ffi::V8ValueHandle,
    #[cfg(feature = "native")]
    owns: bool,
}

impl V8Buffer {
    #[cfg(feature = "native")]
    pub fn new(ctx: *mut ffi::V8ContextHandle, size: usize) -> Result<Self, V8Error> {
        let handle = unsafe { ffi::klyron_v8_buffer_new(ctx, size) };
        if handle.is_null() { Err(V8Error::EvalFailed("buffer_new failed".into())) }
        else { Ok(Self { ctx, handle, owns: true }) }
    }

    #[cfg(not(feature = "native"))]
    pub fn new(_ctx: *mut std::ffi::c_void, _size: usize) -> Result<Self, V8Error> {
        Err(V8Error::NotInitialized)
    }

    #[cfg(feature = "native")]
    pub fn from_string(ctx: *mut ffi::V8ContextHandle, s: &str, encoding: Option<&str>) -> Result<Self, V8Error> {
        let c_str = CString::new(s).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let c_enc = encoding.map(|e| CString::new(e)).transpose().map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let handle = unsafe {
            ffi::klyron_v8_buffer_from_string(ctx, c_str.as_ptr(),
                                               c_enc.as_ref().map_or(std::ptr::null(), |e| e.as_ptr()))
        };
        if handle.is_null() { Err(V8Error::EvalFailed("buffer_from_string failed".into())) }
        else { Ok(Self { ctx, handle, owns: true }) }
    }

    #[cfg(feature = "native")]
    pub fn from_bytes(ctx: *mut ffi::V8ContextHandle, data: &[u8]) -> Result<Self, V8Error> {
        let handle = unsafe { ffi::klyron_v8_buffer_from_bytes(ctx, data.as_ptr(), data.len()) };
        if handle.is_null() { Err(V8Error::EvalFailed("buffer_from_bytes failed".into())) }
        else { Ok(Self { ctx, handle, owns: true }) }
    }

    #[cfg(feature = "native")]
    pub fn handle(&self) -> *mut ffi::V8ValueHandle {
        self.handle
    }

    #[cfg(feature = "native")]
    pub fn length(&self) -> usize {
        unsafe { ffi::klyron_v8_buffer_get_length(self.ctx, self.handle) }
    }

    #[cfg(feature = "native")]
    pub fn to_string(&self, encoding: Option<&str>, start: Option<usize>, end: Option<usize>) -> Result<String, V8Error> {
        let c_enc = encoding.map(|e| CString::new(e)).transpose().map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let r = unsafe {
            ffi::klyron_v8_buffer_to_string(self.ctx, self.handle,
                                             c_enc.as_ref().map_or(std::ptr::null(), |e| e.as_ptr()),
                                             start.unwrap_or(0), end.unwrap_or(self.length()))
        };
        if r.success {
            let s = if r.data.is_null() { String::new() }
            else {
                let s = unsafe { std::ffi::CStr::from_ptr(r.data).to_string_lossy().into() };
                unsafe { ffi::klyron_v8_free_string(r.data) };
                s
            };
            Ok(s)
        } else {
            Err(V8Error::EvalFailed("buffer_to_string failed".into()))
        }
    }

    #[cfg(feature = "native")]
    pub fn slice(&self, start: usize, end: usize) -> Result<Self, V8Error> {
        let handle = unsafe { ffi::klyron_v8_buffer_slice(self.ctx, self.handle, start, end) };
        if handle.is_null() { Err(V8Error::EvalFailed("buffer_slice failed".into())) }
        else { Ok(Self { ctx: self.ctx, handle, owns: true }) }
    }

    #[cfg(feature = "native")]
    pub fn copy_to(&self, target: &V8Buffer, dst_offset: usize, src_offset: usize, count: usize) -> Result<(), V8Error> {
        let r = unsafe { ffi::klyron_v8_buffer_copy(self.ctx, target.handle, dst_offset, self.handle, src_offset, count) };
        if r.success { Ok(()) }
        else { Err(V8Error::EvalFailed("buffer_copy failed".into())) }
    }

    #[cfg(feature = "native")]
    pub fn write(&self, data: &[u8], offset: usize) -> Result<(), V8Error> {
        let r = unsafe { ffi::klyron_v8_buffer_write(self.ctx, self.handle, data.as_ptr(), offset, data.len()) };
        if r.success { Ok(()) }
        else { Err(V8Error::EvalFailed("buffer_write failed".into())) }
    }
}

#[cfg(feature = "native")]
impl Drop for V8Buffer {
    fn drop(&mut self) {
        if self.owns && !self.handle.is_null() {
            unsafe { ffi::klyron_v8_value_dispose(self.handle) }
        }
    }
}
