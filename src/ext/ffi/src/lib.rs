use std::collections::HashMap;
use std::ffi::CString;
use std::sync::Mutex;

use deno_core::{extension, op2, Extension};
use deno_error::JsErrorBox;

struct LibHandle(*mut std::ffi::c_void);
unsafe impl Send for LibHandle {}
unsafe impl Sync for LibHandle {}

type LibMap = HashMap<String, (LibHandle, String)>;

fn with_libs<F: FnOnce(&mut LibMap)>(f: F) {
    static LIBS: Mutex<Option<LibMap>> = Mutex::new(None);
    let mut guard = LIBS.lock().unwrap();
    f(guard.get_or_insert_with(HashMap::new));
}

#[cfg(target_os = "linux")]
#[link(name = "dl")]
unsafe extern "C" {
    fn dlopen(filename: *const std::ffi::c_char, flag: std::ffi::c_int) -> *mut std::ffi::c_void;
    fn dlsym(handle: *mut std::ffi::c_void, symbol: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn dlclose(handle: *mut std::ffi::c_void) -> std::ffi::c_int;
    fn dlerror() -> *mut std::ffi::c_char;
}

#[cfg(not(target_os = "linux"))]
unsafe extern "C" {
    fn dlopen(filename: *const std::ffi::c_char, flag: std::ffi::c_int) -> *mut std::ffi::c_void;
    fn dlsym(handle: *mut std::ffi::c_void, symbol: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn dlclose(handle: *mut std::ffi::c_void) -> std::ffi::c_int;
    fn dlerror() -> *mut std::ffi::c_char;
}

fn last_dlerror() -> String {
    unsafe {
        let ptr = dlerror();
        if ptr.is_null() {
            "unknown error".into()
        } else {
            std::ffi::CStr::from_ptr(ptr).to_string_lossy().into()
        }
    }
}

const RTLD_NOW: std::ffi::c_int = 2;

extension!(
  klyron_ffi,
  ops = [op_ffi_open, op_ffi_call],
  esm_entry_point = "ext:klyron_ffi/ffi.js",

  esm = [dir "js", "ffi.js"],
);

pub fn init() -> Extension {
  klyron_ffi::init()
}

#[op2]
#[string]
fn op_ffi_open(#[string] path: String) -> Result<String, JsErrorBox> {
    let cpath = CString::new(path.clone())
        .map_err(|_| JsErrorBox::generic("path contains null byte"))?;
    let handle = unsafe { dlopen(cpath.as_ptr(), RTLD_NOW) };
    if handle.is_null() {
        return Err(JsErrorBox::generic(format!("dlopen failed: {}", last_dlerror())));
    }
    static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
    let id = format!("lib_{}", NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
    let id_clone = id.clone();
    with_libs(|map| { map.insert(id_clone, (LibHandle(handle), path)); });
    Ok(id)
}

#[op2]
#[string]
fn op_ffi_call(#[string] lib_id: String, #[string] fn_name: String, #[string] args_json: String) -> Result<String, JsErrorBox> {
    let handle_ptr = {
        let mut entry: Option<(LibHandle, String)> = None;
        with_libs(|map| { entry = map.remove(&lib_id); });
        let (h, p) = entry.ok_or_else(|| JsErrorBox::generic("library not found"))?;
        with_libs(|map| { map.insert(lib_id, (LibHandle(h.0), p)); });
        h.0
    };
    let cname = CString::new(fn_name.clone())
        .map_err(|_| JsErrorBox::generic("function name contains null byte"))?;
    let fn_ptr = unsafe { dlsym(handle_ptr, cname.as_ptr()) };
    if fn_ptr.is_null() {
        return Err(JsErrorBox::generic(format!("dlsym failed: {}", last_dlerror())));
    }
    let _args: Vec<serde_json::Value> = serde_json::from_str(&args_json)
        .unwrap_or_default();
    let func: extern "C" fn() -> i64 = unsafe { std::mem::transmute(fn_ptr) };
    let result = func();
    Ok(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_returns_extension() {
        let ext = init();
        assert_eq!(ext.name, "klyron_ffi");
    }

    #[test]
    fn test_ffi_open_nonexistent() {
        let result = op_ffi_open("/tmp/nonexistent_lib_xyz.so".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_ffi_call_no_lib() {
        let result = op_ffi_call("lib_99999".to_string(), "foo".to_string(), "[]".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_ffi_open_null_path() {
        let result = op_ffi_open("lib\0test.so".to_string());
        assert!(result.is_err());
    }
}
