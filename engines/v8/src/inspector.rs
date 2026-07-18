use crate::error::V8Error;

#[cfg(feature = "native")]
use crate::ffi;

pub struct V8Inspector {
    #[cfg(feature = "native")]
    id: i32,
}

impl V8Inspector {
    #[cfg(feature = "native")]
    pub fn new(isolate: *mut ffi::V8IsolateHandle) -> Result<Self, V8Error> {
        let id = unsafe { ffi::klyron_v8_inspector_new(isolate) };
        if id < 0 { Err(V8Error::Internal("inspector creation failed".into())) }
        else { Ok(Self { id }) }
    }

    #[cfg(not(feature = "native"))]
    pub fn new(_isolate: *mut std::ffi::c_void) -> Result<Self, V8Error> {
        Err(V8Error::NotInitialized)
    }

    #[cfg(feature = "native")]
    pub fn connect(&self, url: &str) -> Result<i32, V8Error> {
        use std::ffi::CString;
        let c = CString::new(url).map_err(|e| V8Error::Internal(e.to_string()))?;
        let session = unsafe { ffi::klyron_v8_inspector_connect(self.id, c.as_ptr()) };
        if session < 0 { Err(V8Error::Internal("inspector connect failed".into())) }
        else { Ok(session) }
    }

    #[cfg(feature = "native")]
    pub fn disconnect(session: i32) {
        unsafe { ffi::klyron_v8_inspector_disconnect(session) }
    }

    #[cfg(feature = "native")]
    pub fn dispatch(session: i32, message: &str) -> Result<String, V8Error> {
        use std::ffi::CString;
        let c = CString::new(message).map_err(|e| V8Error::Internal(e.to_string()))?;
        let mut buf = vec![0u8; 65536];
        let written = unsafe { ffi::klyron_v8_inspector_dispatch(session, c.as_ptr(), buf.as_mut_ptr() as *mut std::os::raw::c_char, buf.len()) };
        if written < 0 { Err(V8Error::Internal("inspector dispatch failed".into())) }
        else {
            buf.truncate(written as usize);
            Ok(String::from_utf8_lossy(&buf).to_string())
        }
    }

    #[cfg(feature = "native")]
    pub fn is_active() -> bool {
        unsafe { ffi::klyron_v8_inspector_is_active() }
    }
}

#[cfg(feature = "native")]
impl Drop for V8Inspector {
    fn drop(&mut self) {
        unsafe { ffi::klyron_v8_inspector_dispose(self.id) }
    }
}
