pub mod module_loader;
pub mod error;
pub mod permissions;
pub mod traits;

pub use module_loader::CommonModuleLoader;
pub use error::{CommonError, CommonErrorKind};
pub use permissions::{CommonPermission, CommonPermissions};
pub use traits::{EngineCapabilities, EngineConfig, EngineError, EngineResult};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DefaultEngineError;

    // ── EngineCapabilities tests ──────────────────────────────────────────

    #[test]
    fn test_capabilities_default() {
        let cap = EngineCapabilities::default();
        assert!(cap.supports_modules);
        assert!(!cap.supports_jsx);
        assert_eq!(cap.max_heap_size, 512 * 1024 * 1024);
    }

    #[test]
    fn test_capabilities_boa() {
        let cap = EngineCapabilities::boa();
        assert!(!cap.supports_jsx);
        assert!(!cap.supports_ts);
        assert!(cap.supports_snapshots);
        assert_eq!(cap.max_heap_size, 256 * 1024 * 1024);
    }

    #[test]
    fn test_capabilities_quickjs() {
        let cap = EngineCapabilities::quickjs();
        assert!(!cap.supports_jsx);
        assert!(cap.supports_snapshots);
        assert!(!cap.supports_wasm);
    }

    #[test]
    fn test_capabilities_v8() {
        let cap = EngineCapabilities::v8();
        assert!(cap.supports_jsx);
        assert!(cap.supports_ts);
        assert!(cap.supports_snapshots);
        assert!(cap.supports_wasm);
        assert!(cap.supports_debugger);
    }

    // ── EngineConfig tests ─────────────────────────────────────────────────

    #[test]
    fn test_engine_config_default() {
        let cfg = EngineConfig::default();
        assert!(!cfg.enable_debugger);
        assert!(!cfg.enable_wasm);
        assert!(cfg.enable_jit);
        assert_eq!(cfg.cache_ttl_secs, 3600);
    }

    // ── EngineError tests ──────────────────────────────────────────────────

    #[test]
    fn test_engine_error_display() {
        assert_eq!(EngineError::NotInitialized.to_string(), "Engine not initialized");
        assert_eq!(
            EngineError::InitFailed("oom".into()).to_string(),
            "Engine initialization failed: oom"
        );
        assert_eq!(
            EngineError::ExecutionFailed("crash".into()).to_string(),
            "Execution failed: crash"
        );
        assert_eq!(EngineError::Timeout.to_string(), "Script execution timed out");
        assert_eq!(EngineError::OutOfMemory.to_string(), "Out of memory");
        assert_eq!(
            EngineError::PermissionDenied("no access".into()).to_string(),
            "Permission denied: no access"
        );
        assert_eq!(
            EngineError::ModuleNotFound("lodash".into()).to_string(),
            "Module not found: lodash"
        );
        assert_eq!(EngineError::EngineBusy.to_string(), "Engine is busy");
        assert_eq!(
            EngineError::EnginePoolExhausted.to_string(),
            "Engine pool exhausted"
        );
        assert_eq!(
            EngineError::CacheError("disk full".into()).to_string(),
            "Cache error: disk full"
        );
    }

    #[test]
    fn test_engine_error_debug() {
        let err = EngineError::SyntaxError("unexpected token".into());
        assert!(format!("{err:?}").contains("SyntaxError"));
    }

    #[test]
    fn test_engine_error_clone() {
        let a = EngineError::TypeError("number".into());
        let b = a.clone();
        assert_eq!(format!("{a}"), format!("{b}"));
    }

    // ── CommonErrorKind tests ──────────────────────────────────────────────

    #[test]
    fn test_common_error_kind_display() {
        assert_eq!(CommonErrorKind::NotInitialized.to_string(), "Engine not initialized");
        assert_eq!(
            CommonErrorKind::SyntaxError("bad".into()).to_string(),
            "Syntax error: bad"
        );
    }

    // ── DefaultEngineError tests ──────────────────────────────────────────

    #[test]
    fn test_default_engine_error_new() {
        let err = DefaultEngineError::new(CommonErrorKind::Timeout);
        assert!(matches!(err.kind(), CommonErrorKind::Timeout));
        assert!(err.format_stack_trace().is_none());
    }

    #[test]
    fn test_default_engine_error_with_trace() {
        let err = DefaultEngineError::new(CommonErrorKind::ReferenceError("x".into()))
            .with_trace("at <eval>:1:5".into());
        assert!(matches!(err.kind(), CommonErrorKind::ReferenceError(_)));
        assert_eq!(err.format_stack_trace(), Some("at <eval>:1:5".into()));
    }

    #[test]
    fn test_default_engine_error_formatted_string() {
        let err = DefaultEngineError::new(CommonErrorKind::SyntaxError("eof".into()))
            .with_trace("at line 1".into());
        let s = err.to_formatted_string();
        assert!(s.contains("Syntax error: eof"));
        assert!(s.contains("Stack trace"));
        assert!(s.contains("at line 1"));
    }

    #[test]
    fn test_default_engine_error_no_trace_formatted() {
        let err = DefaultEngineError::new(CommonErrorKind::TypeError("string".into()));
        let s = err.to_formatted_string();
        assert!(s.contains("Type error: string"));
        assert!(!s.contains("Stack trace"));
    }

    // ── CommonPermissions tests ───────────────────────────────────────────

    #[test]
    fn test_permissions_default_deny() {
        let p = CommonPermissions::default();
        assert!(!p.check(&CommonPermission::Read, None));
        assert!(!p.check(&CommonPermission::Write, None));
        assert!(!p.check(&CommonPermission::Net, None));
        assert!(!p.check(&CommonPermission::Env, None));
        assert!(!p.check(&CommonPermission::Run, None));
        assert!(!p.check(&CommonPermission::Ffi, None));
    }

    #[test]
    fn test_permissions_allow_all() {
        let p = CommonPermissions::allow_all();
        assert!(p.check(&CommonPermission::Read, None));
        assert!(p.check(&CommonPermission::Write, None));
        assert!(p.check(&CommonPermission::Net, None));
        assert!(p.check(&CommonPermission::Env, None));
        assert!(p.check(&CommonPermission::Run, None));
        assert!(p.check(&CommonPermission::Ffi, None));
    }

    #[test]
    fn test_permissions_read_path() {
        let p = CommonPermissions {
            allow_read: vec!["/tmp".into()],
            ..Default::default()
        };
        assert!(p.check(&CommonPermission::Read, Some("/tmp/file.txt")));
        assert!(!p.check(&CommonPermission::Read, Some("/etc/passwd")));
        assert!(!p.check(&CommonPermission::Read, None));
    }

    #[test]
    fn test_permissions_write_path() {
        let p = CommonPermissions {
            allow_write: vec!["/var/log".into()],
            ..Default::default()
        };
        assert!(p.check(&CommonPermission::Write, Some("/var/log/app.log")));
        assert!(!p.check(&CommonPermission::Write, Some("/etc/hosts")));
    }

    #[test]
    fn test_permissions_net_host() {
        let p = CommonPermissions {
            allow_net: vec!["api.example.com".into()],
            ..Default::default()
        };
        assert!(p.check(&CommonPermission::Net, Some("api.example.com")));
        assert!(!p.check(&CommonPermission::Net, Some("evil.com")));
        assert!(!p.check(&CommonPermission::Net, None));
    }

    #[test]
    fn test_permissions_net_wildcard() {
        let p = CommonPermissions {
            allow_net: vec!["*".into()],
            ..Default::default()
        };
        assert!(p.check(&CommonPermission::Net, None));
        assert!(p.check(&CommonPermission::Net, Some("any.host")));
    }

    #[test]
    fn test_permissions_all_granted() {
        let p = CommonPermissions::allow_all();
        assert!(p.check(&CommonPermission::All, None));
    }

    #[test]
    fn test_permissions_env() {
        let p = CommonPermissions { allow_env: true, ..Default::default() };
        assert!(p.check(&CommonPermission::Env, None));
    }

    // ── CommonModuleLoader / SharedModuleLoader tests ─────────────────────

    #[test]
    fn test_shared_module_loader_resolve_absolute() {
        let loader = module_loader::SharedModuleLoader::new("/tmp");
        let r = loader.resolve("file:///foo/bar.js", "/base");
        assert_eq!(r, Ok("file:///foo/bar.js".to_string()));
    }

    #[test]
    fn test_shared_module_loader_resolve_relative() {
        let loader = module_loader::SharedModuleLoader::new("/tmp");
        let r = loader.resolve("./lib.js", "/base/mod.js");
        assert_eq!(r, Ok("/base/./lib.js".to_string()));
    }

    #[test]
    fn test_shared_module_loader_resolve_dotdot() {
        let loader = module_loader::SharedModuleLoader::new("/tmp");
        let r = loader.resolve("../other.js", "/base/deep/mod.js");
        assert_eq!(r, Ok("/base/deep/../other.js".to_string()));
    }

    #[test]
    fn test_shared_module_loader_resolve_unresolvable() {
        let loader = module_loader::SharedModuleLoader::new("/nonexistent");
        let r = loader.resolve("some-package", "/base");
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("Module not found"));
    }

    #[test]
    fn test_shared_module_loader_register_and_load() {
        let loader = module_loader::SharedModuleLoader::new("/tmp");
        loader.register("my:module", "export const x = 1;");
        let loaded = loader.load("my:module");
        assert_eq!(loaded, Ok("export const x = 1;".to_string()));
    }

    #[test]
    fn test_resolve_path_from_trait_default() {
        let loader = module_loader::SharedModuleLoader::new("/tmp");
        let r = <module_loader::SharedModuleLoader as CommonModuleLoader>::resolve_path(
            &loader, "file:///a/b.js", "/base",
        );
        assert_eq!(r, Ok("/a/b.js".to_string()));
    }

    #[test]
    fn test_shared_module_loader_load_missing_file() {
        let loader = module_loader::SharedModuleLoader::new("/tmp");
        let path = "/nonexistent_path_12345.js";
        let r = loader.load(path);
        assert!(r.is_err());
    }
}
