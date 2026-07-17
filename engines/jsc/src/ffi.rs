#![cfg(feature = "native")]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uchar};

#[repr(C)]
pub struct JSCEngineHandle {
    _private: [u8; 0],
}

unsafe extern "C" {
    fn jsc_init() -> *mut JSCEngineHandle;
    fn jsc_destroy(engine: *mut JSCEngineHandle);
    fn jsc_eval(engine: *mut JSCEngineHandle, code: *const c_char) -> *mut c_char;
    fn jsc_execute_script(
        engine: *mut JSCEngineHandle,
        filename: *const c_char,
        source: *const c_char,
    ) -> *mut c_char;
    fn jsc_execute_module(
        engine: *mut JSCEngineHandle,
        filename: *const c_char,
        source: *const c_char,
    ) -> *mut c_char;
    fn jsc_get_global(engine: *mut JSCEngineHandle, key: *const c_char) -> *mut c_char;
    fn jsc_set_global(
        engine: *mut JSCEngineHandle,
        key: *const c_char,
        value: *const c_char,
    ) -> c_int;
    fn jsc_call_function(
        engine: *mut JSCEngineHandle,
        name: *const c_char,
        args: *const *const c_char,
        argc: c_int,
    ) -> *mut c_char;
    fn jsc_create_snapshot(
        engine: *mut JSCEngineHandle,
        out_len: *mut usize,
    ) -> *mut c_uchar;
    fn jsc_load_snapshot(
        engine: *mut JSCEngineHandle,
        data: *const c_uchar,
        len: usize,
    ) -> c_int;
    fn jsc_last_error(engine: *mut JSCEngineHandle) -> *const c_char;
    fn jsc_free_string(s: *mut c_char);
    fn jsc_free_buffer(buf: *mut c_uchar);
}

pub struct JSCEnginePtr {
    ptr: *mut JSCEngineHandle,
}

impl JSCEnginePtr {
    pub fn init() -> Result<Self, String> {
        let ptr = unsafe { jsc_init() };
        if ptr.is_null() {
            return Err("JSC: failed to create engine".into());
        }
        Ok(Self { ptr })
    }

    fn last_error(&self) -> String {
        unsafe {
            let err = jsc_last_error(self.ptr);
            if err.is_null() { "Unknown error".into() }
            else { CStr::from_ptr(err).to_string_lossy().into() }
        }
    }

    fn cvt(ptr: *mut c_char) -> Result<String, String> {
        if ptr.is_null() {
            Err("Null result from JSC".into())
        } else {
            let s = unsafe { CStr::from_ptr(ptr).to_string_lossy().into() };
            unsafe { jsc_free_string(ptr) };
            Ok(s)
        }
    }

    pub fn eval(&self, code: &str) -> Result<String, String> {
        let c_code = CString::new(code).map_err(|e| e.to_string())?;
        let result = unsafe { jsc_eval(self.ptr, c_code.as_ptr()) };
        if result.is_null() { Err(self.last_error()) } else { Self::cvt(result) }
    }

    pub fn execute_script(&self, filename: &str, source: &str) -> Result<String, String> {
        let c_filename = CString::new(filename).map_err(|e| e.to_string())?;
        let c_source = CString::new(source).map_err(|e| e.to_string())?;
        let result = unsafe { jsc_execute_script(self.ptr, c_filename.as_ptr(), c_source.as_ptr()) };
        if result.is_null() { Err(self.last_error()) } else { Self::cvt(result) }
    }

    pub fn execute_module(&self, filename: &str, source: &str) -> Result<String, String> {
        let c_filename = CString::new(filename).map_err(|e| e.to_string())?;
        let c_source = CString::new(source).map_err(|e| e.to_string())?;
        let result = unsafe { jsc_execute_module(self.ptr, c_filename.as_ptr(), c_source.as_ptr()) };
        if result.is_null() { Err(self.last_error()) } else { Self::cvt(result) }
    }

    pub fn get_global(&self, key: &str) -> Result<Option<String>, String> {
        let c_key = CString::new(key).map_err(|e| e.to_string())?;
        let result = unsafe { jsc_get_global(self.ptr, c_key.as_ptr()) };
        if result.is_null() { Err(self.last_error()) } else { Self::cvt(result).map(Some) }
    }

    pub fn set_global(&self, key: &str, value: &str) -> Result<(), String> {
        let c_key = CString::new(key).map_err(|e| e.to_string())?;
        let c_value = CString::new(value).map_err(|e| e.to_string())?;
        let ret = unsafe { jsc_set_global(self.ptr, c_key.as_ptr(), c_value.as_ptr()) };
        if ret == 0 { Ok(()) } else { Err(self.last_error()) }
    }

    pub fn call_function(&self, name: &str, args: &[&str]) -> Result<String, String> {
        let c_name = CString::new(name).map_err(|e| e.to_string())?;
        let c_args: Vec<CString> = args.iter()
            .map(|a| CString::new(*a).map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        let c_ptrs: Vec<*const c_char> = c_args.iter().map(|a| a.as_ptr()).collect();
        let result = unsafe {
            jsc_call_function(self.ptr, c_name.as_ptr(), c_ptrs.as_ptr(), c_ptrs.len() as c_int)
        };
        if result.is_null() { Err(self.last_error()) } else { Self::cvt(result) }
    }

    pub fn create_snapshot(&self) -> Result<Vec<u8>, String> {
        let mut len: usize = 0;
        let data = unsafe { jsc_create_snapshot(self.ptr, &mut len) };
        if data.is_null() { Err(self.last_error()) }
        else {
            let buf = unsafe { std::slice::from_raw_parts(data, len) }.to_vec();
            unsafe { jsc_free_buffer(data) };
            Ok(buf)
        }
    }

    pub fn load_snapshot(&self, data: &[u8]) -> Result<(), String> {
        let ret = unsafe { jsc_load_snapshot(self.ptr, data.as_ptr(), data.len()) };
        if ret == 0 { Ok(()) } else { Err(self.last_error()) }
    }
}

impl Drop for JSCEnginePtr {
    fn drop(&mut self) {
        unsafe { jsc_destroy(self.ptr) };
    }
}

unsafe impl Send for JSCEnginePtr {}
unsafe impl Sync for JSCEnginePtr {}
