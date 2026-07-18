
#[cfg(feature = "native")]
use crate::ffi;
#[cfg(feature = "native")]
use std::ffi::CString;

pub struct V8Encoding {
    #[cfg(feature = "native")]
    ctx: *mut ffi::V8ContextHandle,
}

impl V8Encoding {
    #[cfg(feature = "native")]
    pub fn new(ctx: *mut ffi::V8ContextHandle) -> Self {
        Self { ctx }
    }

    #[cfg(not(feature = "native"))]
    pub fn new(_ctx: *mut std::ffi::c_void) -> Self {
        Self {}
    }

    #[cfg(feature = "native")]
    pub fn encode(&self, input: &str) -> Result<*mut ffi::V8ValueHandle, V8Error> {
        let c = CString::new(input).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let ptr = unsafe { ffi::klyron_v8_encoding_encode(self.ctx, c.as_ptr()) };
        if ptr.is_null() { Err(V8Error::EvalFailed("encode failed".into())) }
        else { Ok(ptr) }
    }

    #[cfg(feature = "native")]
    pub fn decode(&self, data: &[u8], encoding: &str) -> Result<String, V8Error> {
        let c_enc = CString::new(encoding).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let r = unsafe { ffi::klyron_v8_encoding_decode(self.ctx, data.as_ptr(), data.len(), c_enc.as_ptr()) };
        if r.success {
            let s = if r.data.is_null() { String::new() }
            else {
                let s = unsafe { std::ffi::CStr::from_ptr(r.data).to_string_lossy().into() };
                unsafe { ffi::klyron_v8_free_string(r.data) };
                s
            };
            Ok(s)
        } else {
            Err(V8Error::EvalFailed("decode failed".into()))
        }
    }

    #[cfg(feature = "native")]
    pub fn base64_encode(&self, data: &[u8]) -> Result<String, V8Error> {
        let r = unsafe { ffi::klyron_v8_encoding_base64_encode(self.ctx, data.as_ptr(), data.len()) };
        if r.success {
            let s = if r.data.is_null() { String::new() }
            else {
                let s = unsafe { std::ffi::CStr::from_ptr(r.data).to_string_lossy().into() };
                unsafe { ffi::klyron_v8_free_string(r.data) };
                s
            };
            Ok(s)
        } else {
            Err(V8Error::EvalFailed("base64_encode failed".into()))
        }
    }

    #[cfg(feature = "native")]
    pub fn base64_decode(&self, input: &str) -> Result<*mut ffi::V8ValueHandle, V8Error> {
        let c = CString::new(input).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let ptr = unsafe { ffi::klyron_v8_encoding_base64_decode(self.ctx, c.as_ptr()) };
        if ptr.is_null() { Err(V8Error::EvalFailed("base64_decode failed".into())) }
        else { Ok(ptr) }
    }

    #[cfg(feature = "native")]
    pub fn hex_encode(&self, data: &[u8]) -> Result<String, V8Error> {
        let r = unsafe { ffi::klyron_v8_encoding_hex_encode(self.ctx, data.as_ptr(), data.len()) };
        if r.success {
            let s = if r.data.is_null() { String::new() }
            else {
                let s = unsafe { std::ffi::CStr::from_ptr(r.data).to_string_lossy().into() };
                unsafe { ffi::klyron_v8_free_string(r.data) };
                s
            };
            Ok(s)
        } else {
            Err(V8Error::EvalFailed("hex_encode failed".into()))
        }
    }

    #[cfg(feature = "native")]
    pub fn hex_decode(&self, input: &str) -> Result<*mut ffi::V8ValueHandle, V8Error> {
        let c = CString::new(input).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let ptr = unsafe { ffi::klyron_v8_encoding_hex_decode(self.ctx, c.as_ptr()) };
        if ptr.is_null() { Err(V8Error::EvalFailed("hex_decode failed".into())) }
        else { Ok(ptr) }
    }
}
