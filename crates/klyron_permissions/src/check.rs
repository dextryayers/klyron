use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionKind {
    FsRead,
    FsWrite,
    EnvRead,
    EnvWrite,
    NetConnect,
    NetListen,
    Run,
    FFI,
}

impl PermissionKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::FsRead => "fs-read",
            Self::FsWrite => "fs-write",
            Self::EnvRead => "env-read",
            Self::EnvWrite => "env-write",
            Self::NetConnect => "net-connect",
            Self::NetListen => "net-listen",
            Self::Run => "run",
            Self::FFI => "ffi",
        }
    }
}

impl fmt::Display for PermissionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionState {
    Granted,
    Denied,
    Prompt,
}

#[derive(Debug, Clone, Default)]
pub struct PermissionFlags {
    pub allow_read: Vec<String>,
    pub deny_read: Vec<String>,
    pub allow_write: Vec<String>,
    pub deny_write: Vec<String>,
    pub allow_env: Vec<String>,
    pub deny_env: Vec<String>,
    pub allow_net: Vec<String>,
    pub deny_net: Vec<String>,
    pub allow_run: Vec<String>,
    pub deny_run: Vec<String>,
    pub allow_ffi: bool,
    pub allow_all: bool,
    pub prompt: bool,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PermissionOrigin {
    pub module: String,
    pub kind: PermissionKind,
    pub resource: String,
}

impl PermissionOrigin {
    pub fn new(module: &str, kind: PermissionKind, resource: &str) -> Self {
        Self {
            module: module.to_string(),
            kind,
            resource: resource.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedDecision {
    pub state: PermissionState,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub origin: PermissionOrigin,
}

static PERMISSION_CACHE: Lazy<Mutex<HashMap<PermissionOrigin, CachedDecision>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone, thiserror::Error)]
pub enum PermissionError {
    #[error("{kind} access denied to \"{resource}\": {reason}")]
    Denied {
        kind: PermissionKind,
        resource: String,
        reason: String,
    },
}

pub fn canonicalize_path(path: &str) -> String {
    let p = std::path::Path::new(path);
    p.canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| {
            let p = PathBuf::from(path);
            if p.is_relative() {
                if let Ok(cwd) = std::env::current_dir() {
                    cwd.join(&p).to_string_lossy().to_string()
                } else {
                    path.to_string()
                }
            } else {
                path.to_string()
            }
        })
}

#[derive(Debug, Clone)]
pub struct PermissionManager {
    flags: Arc<Mutex<PermissionFlags>>,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self {
            flags: Arc::new(Mutex::new(PermissionFlags::default())),
        }
    }

    pub fn with_flags(flags: PermissionFlags) -> Self {
        Self {
            flags: Arc::new(Mutex::new(flags)),
        }
    }

    pub fn set_flags(&self, flags: PermissionFlags) {
        let mut current = self.flags.lock();
        *current = flags;
    }

    pub fn flags(&self) -> PermissionFlags {
        self.flags.lock().clone()
    }

    pub fn check(
        &self,
        kind: PermissionKind,
        resource: &str,
        module: &str,
    ) -> Result<(), PermissionError> {
        let flags = self.flags.lock();

        let deny_list: &[String] = match kind {
            PermissionKind::FsRead => &flags.deny_read,
            PermissionKind::FsWrite => &flags.deny_write,
            PermissionKind::NetConnect | PermissionKind::NetListen => &flags.deny_net,
            PermissionKind::EnvRead | PermissionKind::EnvWrite => &flags.deny_env,
            PermissionKind::Run => &flags.deny_run,
            PermissionKind::FFI => &[],
        };
        if Self::matches_any(resource, deny_list) {
            return Err(PermissionError::Denied {
                kind,
                resource: resource.to_string(),
                reason: format!("Explicitly denied by --deny-{}", kind.name()),
            });
        }

        if flags.allow_all {
            self.cache_decision(kind, resource, module, PermissionState::Granted);
            return Ok(());
        }

        let allow_list: &[String] = match kind {
            PermissionKind::FsRead => &flags.allow_read,
            PermissionKind::FsWrite => &flags.allow_write,
            PermissionKind::NetConnect | PermissionKind::NetListen => &flags.allow_net,
            PermissionKind::EnvRead | PermissionKind::EnvWrite => &flags.allow_env,
            PermissionKind::Run => &flags.allow_run,
            PermissionKind::FFI => &[],
        };
        if Self::matches_any(resource, allow_list) {
            self.cache_decision(kind, resource, module, PermissionState::Granted);
            return Ok(());
        }

        if kind == PermissionKind::FFI && flags.allow_ffi {
            self.cache_decision(kind, resource, module, PermissionState::Granted);
            return Ok(());
        }

        if flags.prompt {
            self.prompt_user(kind, resource, module)
        } else {
            Err(PermissionError::Denied {
                kind,
                resource: resource.to_string(),
                reason: "Not allowed by permission flags; use --prompt for interactive mode"
                    .to_string(),
            })
        }
    }

    fn matches_any(resource: &str, patterns: &[String]) -> bool {
        if patterns.iter().any(|p| p == resource || p == "*") {
            return true;
        }
        patterns.iter().any(|p| {
            if let Ok(pat) = glob::Pattern::new(p) {
                pat.matches(resource)
            } else {
                false
            }
        })
    }

    #[allow(dead_code)]
    fn glob_match(resource: &str, pattern: &str) -> bool {
        if pattern == "*" || pattern == resource {
            return true;
        }
        if let Ok(pat) = glob::Pattern::new(pattern) {
            pat.matches(resource)
        } else {
            false
        }
    }

    fn cache_decision(
        &self,
        kind: PermissionKind,
        resource: &str,
        module: &str,
        state: PermissionState,
    ) {
        let origin = PermissionOrigin::new(module, kind, resource);
        let decision = CachedDecision {
            state,
            timestamp: chrono::Utc::now(),
            origin: origin.clone(),
        };
        PERMISSION_CACHE.lock().insert(origin, decision);
    }

    pub fn get_cached(
        &self,
        kind: PermissionKind,
        resource: &str,
        module: &str,
    ) -> Option<PermissionState> {
        let origin = PermissionOrigin::new(module, kind, resource);
        PERMISSION_CACHE.lock().get(&origin).map(|d| d.state)
    }

    pub fn clear_cache(&self) {
        PERMISSION_CACHE.lock().clear();
    }

    fn prompt_user(
        &self,
        kind: PermissionKind,
        resource: &str,
        module: &str,
    ) -> Result<(), PermissionError> {
        eprintln!(
            "Klyron requires {kind} access to \"{resource}\" (requested by {module}). Allow? [y/N] "
        );
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            if input.trim().eq_ignore_ascii_case("y")
                || input.trim().eq_ignore_ascii_case("yes")
            {
                self.cache_decision(kind, resource, module, PermissionState::Granted);
                return Ok(());
            }
        }
        self.cache_decision(kind, resource, module, PermissionState::Denied);
        Err(PermissionError::Denied {
            kind,
            resource: resource.to_string(),
            reason: "Denied by user prompt".to_string(),
        })
    }

    pub fn log_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".klyron")
            .join("logs")
    }

    pub fn audit_log_path() -> PathBuf {
        Self::log_dir().join("audit.log")
    }

    pub fn ensure_log_dir() -> std::io::Result<()> {
        std::fs::create_dir_all(Self::log_dir())
    }

    pub fn cache_size(&self) -> usize {
        PERMISSION_CACHE.lock().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_kind_name() {
        assert_eq!(PermissionKind::FsRead.name(), "fs-read");
        assert_eq!(PermissionKind::FFI.name(), "ffi");
    }

    #[test]
    fn test_allow_all() {
        let pm = PermissionManager::with_flags(PermissionFlags {
            allow_all: true,
            ..Default::default()
        });
        assert!(pm
            .check(PermissionKind::FsRead, "/etc/passwd", "test")
            .is_ok());
        assert!(pm
            .check(PermissionKind::NetConnect, "evil.com", "test")
            .is_ok());
    }

    #[test]
    fn test_deny_overrides() {
        let pm = PermissionManager::with_flags(PermissionFlags {
            allow_all: true,
            deny_read: vec!["/etc/**".to_string()],
            ..Default::default()
        });
        assert!(pm
            .check(PermissionKind::FsRead, "/home/user/file", "test")
            .is_ok());
        assert!(pm
            .check(PermissionKind::FsRead, "/etc/passwd", "test")
            .is_err());
    }

    #[test]
    fn test_explicit_allow() {
        let pm = PermissionManager::with_flags(PermissionFlags {
            allow_read: vec!["/app/**".to_string()],
            ..Default::default()
        });
        assert!(pm
            .check(PermissionKind::FsRead, "/app/src/main.ts", "test")
            .is_ok());
        assert!(pm
            .check(PermissionKind::FsRead, "/etc/passwd", "test")
            .is_err());
    }

    #[test]
    fn test_caching() {
        let pm = PermissionManager::with_flags(PermissionFlags {
            allow_read: vec!["/tmp/**".to_string()],
            ..Default::default()
        });
        assert!(pm
            .check(PermissionKind::FsRead, "/tmp/test.txt", "mod")
            .is_ok());
        assert_eq!(
            pm.get_cached(PermissionKind::FsRead, "/tmp/test.txt", "mod"),
            Some(PermissionState::Granted)
        );
    }

    #[test]
    fn test_ffi_denied_by_default() {
        let pm = PermissionManager::new();
        assert!(pm.check(PermissionKind::FFI, "libc.dylib", "test").is_err());
    }

    #[test]
    fn test_ffi_allowed_with_flag() {
        let pm = PermissionManager::with_flags(PermissionFlags {
            allow_ffi: true,
            ..Default::default()
        });
        assert!(pm
            .check(PermissionKind::FFI, "libc.dylib", "test")
            .is_ok());
    }

    #[test]
    fn test_prompt_flag() {
        let pm = PermissionManager::with_flags(PermissionFlags {
            prompt: true,
            ..Default::default()
        });
        assert!(pm.check(PermissionKind::FsRead, "/test", "mod").is_err());
    }

    #[test]
    fn test_origin_tracking() {
        let origin = PermissionOrigin::new("my_module.js", PermissionKind::FsRead, "/etc/config");
        assert_eq!(origin.module, "my_module.js");
        assert_eq!(origin.kind, PermissionKind::FsRead);
        assert_eq!(origin.resource, "/etc/config");
    }

    #[test]
    fn test_canonicalize_path() {
        let path = canonicalize_path("/");
        assert!(!path.is_empty());
    }

    #[test]
    fn test_cache_size() {
        let pm = PermissionManager::with_flags(PermissionFlags {
            allow_read: vec!["/tmp/**".to_string()],
            ..Default::default()
        });
        let _ = pm.check(PermissionKind::FsRead, "/tmp/a", "mod");
        assert!(pm.cache_size() > 0);
        pm.clear_cache();
        assert_eq!(pm.cache_size(), 0);
    }

    #[test]
    fn test_env_permissions() {
        let pm = PermissionManager::with_flags(PermissionFlags {
            allow_env: vec!["PATH".to_string()],
            ..Default::default()
        });
        assert!(pm
            .check(PermissionKind::EnvRead, "PATH", "mod")
            .is_ok());
        assert!(pm
            .check(PermissionKind::EnvRead, "SECRET", "mod")
            .is_err());
    }

    #[test]
    fn test_glob_match() {
        assert!(PermissionManager::glob_match("/app/src/main.ts", "/app/**"));
        assert!(!PermissionManager::glob_match("/etc/passwd", "/app/**"));
        assert!(PermissionManager::glob_match("*.js", "*.js"));
        assert!(PermissionManager::glob_match("/path/to/file.js", "*.js"));
    }

    #[test]
    fn test_display_permission_kind() {
        assert_eq!(format!("{}", PermissionKind::FsRead), "fs-read");
        assert_eq!(format!("{}", PermissionKind::FFI), "ffi");
    }
}
