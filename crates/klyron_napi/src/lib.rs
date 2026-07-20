pub mod bindings;
pub mod types;

pub use bindings::*;
pub use types::*;

use std::path::{Path, PathBuf};

pub fn load_native_addon(path: &Path) -> anyhow::Result<*mut libc::c_void> {
    let path_str = path.to_string_lossy();
    let handle = unsafe {
        libc::dlopen(
            path_str.as_ptr() as *const libc::c_char,
            libc::RTLD_NOW | libc::RTLD_LOCAL,
        )
    };
    if handle.is_null() {
        let err = unsafe { std::ffi::CStr::from_ptr(libc::dlerror()) };
        anyhow::bail!("Failed to load native addon {}: {}", path.display(), err.to_string_lossy());
    }
    Ok(handle)
}

pub fn find_napi_addon(name: &str, dir: &Path) -> Option<PathBuf> {
    let candidates = [
        dir.join("node_modules").join(name).join(format!("{}.node", name)),
        dir.join("node_modules").join(name).join("build").join("Release").join(format!("{}.node", name)),
        dir.join("node_modules").join(name).join("build").join("Debug").join(format!("{}.node", name)),
        dir.join("node_modules").join(name).join("prebuilds").join(format!("{}.node", name)),
    ];
    candidates.into_iter().find(|p| p.exists())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::Path;
use std::path::{Path, PathBuf};

    #[test]
    fn test_napi_loader_new() {
        let loader = NapiLoader::new();
        assert!(loader.list_loaded().is_empty());
        assert_eq!(NapiLoader::napi_version(), 9);
    }

    #[test]
    fn test_is_napi_module() {
        assert!(NapiLoader::is_napi_module(Path::new("addon.node")));
        assert!(!NapiLoader::is_napi_module(Path::new("addon.so")));
        assert!(!NapiLoader::is_napi_module(Path::new("addon.dylib")));
    }

    #[test]
    fn test_napi_version_check() {
        assert!(NapiModule::check_version_compatibility(1).is_ok());
        assert!(NapiModule::check_version_compatibility(9).is_ok());
        assert!(NapiModule::check_version_compatibility(0).is_err());
        assert!(NapiModule::check_version_compatibility(10).is_err());
    }

    #[test]
    fn test_buffer_bounds_check() {
        let buf = vec![0u8; 10];
        assert!(check_buffer_bounds(&buf, 0, 10).is_ok());
        assert!(check_buffer_bounds(&buf, 5, 5).is_ok());
        assert!(check_buffer_bounds(&buf, 10, 0).is_ok());
        assert!(check_buffer_bounds(&buf, 0, 11).is_err());
        assert!(check_buffer_bounds(&buf, 11, 0).is_err());
    }

    #[test]
    fn test_typed_array_bounds() {
        let kind = TypedArrayKind::Uint8Array;
        assert!(check_typed_array_bounds(kind, 10, 0, 10).is_ok());
        assert!(check_typed_array_bounds(kind, 10, 5, 5).is_ok());
        assert!(check_typed_array_bounds(kind, 10, 0, 11).is_err());
        assert!(check_typed_array_bounds(kind, 10, 10, 1).is_err());
    }

    #[test]
    fn test_typed_array_float64_bounds() {
        let kind = TypedArrayKind::Float64Array;
        assert!(check_typed_array_bounds(kind, 10, 0, 80).is_ok());
        assert!(check_typed_array_bounds(kind, 10, 0, 81).is_err());
    }

    #[test]
    fn test_typed_array_kind_element_size() {
        assert_eq!(TypedArrayKind::Int8Array.element_size(), 1);
        assert_eq!(TypedArrayKind::Int32Array.element_size(), 4);
        assert_eq!(TypedArrayKind::Float64Array.element_size(), 8);
        assert_eq!(TypedArrayKind::BigInt64Array.element_size(), 8);
    }

    #[test]
    fn test_typed_array_kind_from_str() {
        assert_eq!(
            TypedArrayKind::from_str("Uint8Array"),
            Some(TypedArrayKind::Uint8Array)
        );
        assert_eq!(
            TypedArrayKind::from_str("Float32Array"),
            Some(TypedArrayKind::Float32Array)
        );
        assert_eq!(TypedArrayKind::from_str("Nonexistent"), None);
    }

    #[test]
    fn test_napi_value_types() {
        let _v1 = NapiValue::Undefined;
        let _v2 = NapiValue::Null;
        let _v3 = NapiValue::Bool(true);
        let _v4 = NapiValue::Number(3.14);
        let _v5 = NapiValue::String("hello".into());
        let _v6 = NapiValue::Buffer(vec![1, 2, 3]);
        let _v7 = NapiValue::Symbol("sym".into());
        assert!(matches!(_v3, NapiValue::Bool(true)));
    }

    #[test]
    fn test_napi_module_detect_version() {
        let p = Path::new("addon.napi-v9.node");
        assert_eq!(NapiModule::detect_napi_version(p), 9);
        let p2 = Path::new("addon.napi8.node");
        assert_eq!(NapiModule::detect_napi_version(p2), 8);
        let p3 = Path::new("addon.node");
        assert_eq!(NapiModule::detect_napi_version(p3), 9);
    }

    #[test]
    fn test_async_work_pool() {
        let pool = AsyncWorkPool::new(2);
        assert_eq!(pool.workers, 2);
    }

    #[test]
    fn test_async_work_new() {
        let work = AsyncWork::new(
            "test",
            Box::new(|| Ok(NapiValue::Number(42.0))),
            Box::new(|_val| {}),
        );
        assert_eq!(work.name, "test");
    }

    #[test]
    fn test_napi_loader_add_search_path() {
        let mut loader = NapiLoader::new();
        loader.add_search_path(PathBuf::from("/custom/path"));
        assert!(loader
            .search_paths
            .iter()
            .any(|p| p.ends_with("custom/path")));
    }

    #[test]
    fn test_napi_loader_clear() {
        let mut loader = NapiLoader::new();
        loader.clear();
        assert!(loader.list_loaded().is_empty());
    }

    #[test]
    fn test_napi_module_set_property() {
        let mut module = NapiModule {
            name: "test".into(),
            path: PathBuf::from("test.node"),
            exports: HashMap::new(),
            napi_version: 9,
            library: None,
        };
        module.set_property("key".into(), NapiValue::Number(42.0));
        assert!(module.get_property("key").is_some());
    }

    #[test]
    fn test_check_module_version() {
        assert!(NapiLoader::check_module_version(9).is_ok());
        assert!(NapiLoader::check_module_version(0).is_err());
    }

    #[test]
    fn test_napi_error_types() {
        let e1 = NapiError::TypeError("bad type".into());
        let e2 = NapiError::BufferOverflow("overflow".into());
        let e3 = NapiError::UnsupportedVersion(0);
        assert!(e1.to_string().contains("bad type"));
        assert!(e2.to_string().contains("overflow"));
        assert!(e3.to_string().contains("0"));
    }

    #[test]
    fn test_napi_value_from_conversions() {
        let v1: NapiValue = true.into();
        assert!(matches!(v1, NapiValue::Bool(true)));

        let v2: NapiValue = 42i32.into();
        assert!(matches!(v2, NapiValue::Int(42)));

        let v3: NapiValue = 3.14f64.into();
        assert!(matches!(v3, NapiValue::Number(n) if (n - 3.14).abs() < 1e-10));

        let v4: NapiValue = "hello".into();
        assert!(matches!(v4, NapiValue::String(s) if s == "hello"));

        let v5: NapiValue = vec![1u8, 2, 3].into();
        assert!(matches!(v5, NapiValue::Buffer(_)));
    }

    #[test]
    fn test_napi_value_type_name() {
        assert_eq!(NapiValue::Undefined.type_name(), "undefined");
        assert_eq!(NapiValue::Bool(true).type_name(), "boolean");
        assert_eq!(NapiValue::Number(1.0).type_name(), "number");
        assert_eq!(NapiValue::String("a".into()).type_name(), "string");
    }

    #[test]
    fn test_napi_value_as_methods() {
        let s = NapiValue::String("test".into());
        assert_eq!(s.as_str(), Some("test"));
        assert!(s.is_string());

        let n = NapiValue::Number(42.5);
        assert_eq!(n.as_f64(), Some(42.5));

        let b = NapiValue::Bool(false);
        assert_eq!(b.as_bool(), Some(false));
    }

    #[test]
    fn test_detect_napi_version_unknown() {
        let p = Path::new("random.node");
        assert_eq!(NapiModule::detect_napi_version(p), 9);
    }

    #[test]
    fn test_napi_loader_list_symbols_empty() {
        let loader = NapiLoader::new();
        assert!(loader.list_symbols().is_empty());
    }

    #[test]
    fn test_napi_value_typed_array_to_str() {
        assert_eq!(TypedArrayKind::Uint8Array.to_str(), "Uint8Array");
        assert_eq!(TypedArrayKind::Float64Array.to_str(), "Float64Array");
    }
}
