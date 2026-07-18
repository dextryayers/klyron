
#[cfg(feature = "native")]
use crate::error::V8Error;
#[cfg(feature = "native")]
use crate::ffi;

pub struct V8Crypto {
    #[cfg(feature = "native")]
    ctx: *mut ffi::V8ContextHandle,
}

impl V8Crypto {
    #[cfg(feature = "native")]
    pub fn new(ctx: *mut ffi::V8ContextHandle) -> Self {
        Self { ctx }
    }

    #[cfg(not(feature = "native"))]
    pub fn new(_ctx: *mut std::ffi::c_void) -> Self {
        Self {}
    }

    #[cfg(feature = "native")]
    pub fn random_bytes(&self, size: usize) -> Result<*mut ffi::V8ValueHandle, V8Error> {
        let ptr = unsafe { ffi::klyron_v8_crypto_random_bytes(self.ctx, size) };
        if ptr.is_null() { Err(V8Error::Internal("random_bytes failed".into())) }
        else { Ok(ptr) }
    }

    #[cfg(feature = "native")]
    pub fn random_uuid(&self) -> Result<String, V8Error> {
        let r = unsafe { ffi::klyron_v8_crypto_random_uuid(self.ctx) };
        if r.success {
            let s = if r.data.is_null() { String::new() }
            else {
                let s = unsafe { std::ffi::CStr::from_ptr(r.data).to_string_lossy().into() };
                unsafe { ffi::klyron_v8_free_string(r.data) };
                s
            };
            Ok(s)
        } else {
            Err(V8Error::Internal("random_uuid failed".into()))
        }
    }
}
