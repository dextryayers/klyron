use crate::error::V8Error;

#[cfg(feature = "native")]
use crate::ffi;

pub struct V8Snapshot {
    #[cfg(feature = "native")]
    handle: *mut ffi::V8SnapshotHandle,
    #[cfg(feature = "native")]
    data: Option<Vec<u8>>,
}

impl V8Snapshot {
    #[cfg(feature = "native")]
    pub fn create(ctx: *mut ffi::V8ContextHandle) -> Result<Self, V8Error> {
        let ptr = unsafe { ffi::klyron_v8_snapshot_create(ctx) };
        if ptr.is_null() { Err(V8Error::SnapshotFailed("create failed".into())) }
        else { Ok(Self { handle: ptr, data: None }) }
    }

    #[cfg(not(feature = "native"))]
    pub fn create(_ctx: *mut std::ffi::c_void) -> Result<Self, V8Error> {
        Err(V8Error::NotInitialized)
    }

    #[cfg(feature = "native")]
    pub fn load(blob: &[u8]) -> Result<Self, V8Error> {
        let ptr = unsafe { ffi::klyron_v8_snapshot_load(blob.as_ptr() as *const std::os::raw::c_char, blob.len()) };
        if ptr.is_null() { Err(V8Error::SnapshotFailed("load failed".into())) }
        else { Ok(Self { handle: ptr, data: Some(blob.to_vec()) }) }
    }

    #[cfg(not(feature = "native"))]
    pub fn load(_blob: &[u8]) -> Result<Self, V8Error> {
        Err(V8Error::NotInitialized)
    }

    #[cfg(feature = "native")]
    pub fn handle(&self) -> *mut ffi::V8SnapshotHandle {
        self.handle
    }

    #[cfg(feature = "native")]
    pub fn as_bytes(&self) -> Option<&[u8]> {
        self.data.as_deref()
    }
}

#[cfg(feature = "native")]
impl Drop for V8Snapshot {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { ffi::klyron_v8_snapshot_dispose(self.handle) }
        }
    }
}
