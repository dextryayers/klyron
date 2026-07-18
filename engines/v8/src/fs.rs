
#[cfg(feature = "native")]
use crate::error::V8Error;
#[cfg(feature = "native")]
use crate::ffi;
#[cfg(feature = "native")]
use std::ffi::CString;

#[derive(Debug, Clone)]
pub struct FSStat {
    pub dev: u64,
    pub ino: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
    pub blksize: u64,
    pub blocks: u64,
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
    pub file_type: i32,
}

pub struct V8FS {
    #[cfg(feature = "native")]
    ctx: *mut ffi::V8ContextHandle,
}

impl V8FS {
    #[cfg(feature = "native")]
    pub fn new(ctx: *mut ffi::V8ContextHandle) -> Self {
        Self { ctx }
    }

    #[cfg(not(feature = "native"))]
    pub fn new(_ctx: *mut std::ffi::c_void) -> Self {
        Self {}
    }

    #[cfg(feature = "native")]
    pub fn read_file(&self, path: &str) -> Result<*mut ffi::V8ValueHandle, V8Error> {
        let c = CString::new(path).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let mut result: *mut ffi::V8ValueHandle = std::ptr::null_mut();
        let r = unsafe { ffi::klyron_v8_fs_read_file(self.ctx, c.as_ptr(), &mut result) };
        if r.success { Ok(result) }
        else { Err(V8Error::EvalFailed("read_file failed".into())) }
    }

    #[cfg(feature = "native")]
    pub fn write_file(&self, path: &str, data: &[u8]) -> Result<(), V8Error> {
        let c = CString::new(path).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let r = unsafe { ffi::klyron_v8_fs_write_file(self.ctx, c.as_ptr(), data.as_ptr(), data.len()) };
        if r.success { Ok(()) }
        else { Err(V8Error::EvalFailed("write_file failed".into())) }
    }

    #[cfg(feature = "native")]
    pub fn append_file(&self, path: &str, data: &[u8]) -> Result<(), V8Error> {
        let c = CString::new(path).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let r = unsafe { ffi::klyron_v8_fs_append_file(self.ctx, c.as_ptr(), data.as_ptr(), data.len()) };
        if r.success { Ok(()) }
        else { Err(V8Error::EvalFailed("append_file failed".into())) }
    }

    #[cfg(feature = "native")]
    pub fn stat(&self, path: &str) -> Result<FSStat, V8Error> {
        let c = CString::new(path).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let mut st = ffi::V8Stat {
            dev: 0, ino: 0, mode: 0, uid: 0, gid: 0,
            size: 0, blksize: 0, blocks: 0,
            atime: 0, mtime: 0, ctime: 0, file_type: 0,
        };
        let r = unsafe { ffi::klyron_v8_fs_stat(self.ctx, c.as_ptr(), &mut st) };
        if r.success {
            Ok(FSStat {
                dev: st.dev, ino: st.ino, mode: st.mode,
                uid: st.uid, gid: st.gid, size: st.size,
                blksize: st.blksize, blocks: st.blocks,
                atime: st.atime, mtime: st.mtime, ctime: st.ctime,
                file_type: st.file_type,
            })
        } else {
            Err(V8Error::EvalFailed("stat failed".into()))
        }
    }

    #[cfg(feature = "native")]
    pub fn mkdir(&self, path: &str, mode: i32) -> Result<(), V8Error> {
        let c = CString::new(path).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let r = unsafe { ffi::klyron_v8_fs_mkdir(self.ctx, c.as_ptr(), mode) };
        if r.success { Ok(()) }
        else { Err(V8Error::EvalFailed("mkdir failed".into())) }
    }

    #[cfg(feature = "native")]
    pub fn rmdir(&self, path: &str) -> Result<(), V8Error> {
        let c = CString::new(path).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let r = unsafe { ffi::klyron_v8_fs_rmdir(self.ctx, c.as_ptr()) };
        if r.success { Ok(()) }
        else { Err(V8Error::EvalFailed("rmdir failed".into())) }
    }

    #[cfg(feature = "native")]
    pub fn unlink(&self, path: &str) -> Result<(), V8Error> {
        let c = CString::new(path).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let r = unsafe { ffi::klyron_v8_fs_unlink(self.ctx, c.as_ptr()) };
        if r.success { Ok(()) }
        else { Err(V8Error::EvalFailed("unlink failed".into())) }
    }

    #[cfg(feature = "native")]
    pub fn rename(&self, old: &str, new: &str) -> Result<(), V8Error> {
        let c_old = CString::new(old).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let c_new = CString::new(new).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let r = unsafe { ffi::klyron_v8_fs_rename(self.ctx, c_old.as_ptr(), c_new.as_ptr()) };
        if r.success { Ok(()) }
        else { Err(V8Error::EvalFailed("rename failed".into())) }
    }

    #[cfg(feature = "native")]
    pub fn exists(&self, path: &str) -> Result<bool, V8Error> {
        let c = CString::new(path).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let mut exists = false;
        let r = unsafe { ffi::klyron_v8_fs_exists(self.ctx, c.as_ptr(), &mut exists) };
        if r.success { Ok(exists) }
        else { Err(V8Error::EvalFailed("exists failed".into())) }
    }

    #[cfg(feature = "native")]
    pub fn read_dir(&self, path: &str) -> Result<*mut ffi::V8ValueHandle, V8Error> {
        let c = CString::new(path).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let ptr = unsafe { ffi::klyron_v8_fs_read_dir(self.ctx, c.as_ptr()) };
        if ptr.is_null() { Err(V8Error::EvalFailed("read_dir failed".into())) }
        else { Ok(ptr) }
    }
}
