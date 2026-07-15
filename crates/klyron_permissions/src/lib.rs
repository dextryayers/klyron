use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

// ── Permission Types ──────────────────────────────────────────────────────

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

// ── Permission State ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionState {
    Granted,
    Denied,
    Prompt,
}

// ── Permission Flags ──────────────────────────────────────────────────────

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

// ── Origin Tracking ───────────────────────────────────────────────────────

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

// ── Permission Cache ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CachedDecision {
    pub state: PermissionState,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub origin: PermissionOrigin,
}

static PERMISSION_CACHE: Lazy<Mutex<HashMap<PermissionOrigin, CachedDecision>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static PERMISSION_FLAGS: Lazy<Mutex<PermissionFlags>> =
    Lazy::new(|| Mutex::new(PermissionFlags::default()));

// ── Permission Manager ────────────────────────────────────────────────────

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

        if flags.allow_all {
            self.cache_decision(kind, resource, module, PermissionState::Granted);
            return Ok(());
        }

        match kind {
            PermissionKind::FsRead => {
                if Self::matches_any(resource, &flags.deny_read) {
                    return Err(PermissionError::Denied {
                        kind,
                        resource: resource.to_string(),
                        reason: "Explicitly denied by --deny-read".to_string(),
                    });
                }
                for pat in &flags.allow_read {
                    if Self::glob_match(resource, pat) {
                        self.cache_decision(kind, resource, module, PermissionState::Granted);
                        return Ok(());
                    }
                }
            }
            PermissionKind::FsWrite => {
                if Self::matches_any(resource, &flags.deny_write) {
                    return Err(PermissionError::Denied {
                        kind,
                        resource: resource.to_string(),
                        reason: "Explicitly denied by --deny-write".to_string(),
                    });
                }
                for pat in &flags.allow_write {
                    if Self::glob_match(resource, pat) {
                        self.cache_decision(kind, resource, module, PermissionState::Granted);
                        return Ok(());
                    }
                }
            }
            PermissionKind::NetConnect | PermissionKind::NetListen => {
                if Self::matches_any(resource, &flags.deny_net) {
                    return Err(PermissionError::Denied {
                        kind,
                        resource: resource.to_string(),
                        reason: "Explicitly denied by --deny-net".to_string(),
                    });
                }
                for pat in &flags.allow_net {
                    if Self::glob_match(resource, pat) {
                        self.cache_decision(kind, resource, module, PermissionState::Granted);
                        return Ok(());
                    }
                }
            }
            PermissionKind::EnvRead | PermissionKind::EnvWrite => {
                if Self::matches_any(resource, &flags.deny_env) {
                    return Err(PermissionError::Denied {
                        kind,
                        resource: resource.to_string(),
                        reason: "Explicitly denied by --deny-env".to_string(),
                    });
                }
            }
            PermissionKind::Run => {
                if Self::matches_any(resource, &flags.deny_run) {
                    return Err(PermissionError::Denied {
                        kind,
                        resource: resource.to_string(),
                        reason: "Explicitly denied by --deny-run".to_string(),
                    });
                }
                for pat in &flags.allow_run {
                    if Self::glob_match(resource, pat) {
                        self.cache_decision(kind, resource, module, PermissionState::Granted);
                        return Ok(());
                    }
                }
            }
            PermissionKind::FFI => {
                if !flags.allow_all && !flags.allow_ffi {
                    return Err(PermissionError::Denied {
                        kind,
                        resource: resource.to_string(),
                        reason: "FFI requires --allow-ffi flag".to_string(),
                    });
                }
            }
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

    pub fn get_cached(&self, kind: PermissionKind, resource: &str, module: &str) -> Option<PermissionState> {
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
            if input.trim().eq_ignore_ascii_case("y") || input.trim().eq_ignore_ascii_case("yes") {
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
}

// ── Errors ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, thiserror::Error)]
pub enum PermissionError {
    #[error("{kind} access denied to \"{resource}\": {reason}")]
    Denied {
        kind: PermissionKind,
        resource: String,
        reason: String,
    },
}

// ── Parse helpers for CLI flags ───────────────────────────────────────────

pub fn parse_allow_flags(flags: &[String]) -> PermissionFlags {
    let mut pf = PermissionFlags::default();
    for flag in flags {
        match flag.as_str() {
            "--allow-read" => {
                // next arg is the path pattern
            }
            "--allow-write" => {}
            "--allow-net" => {}
            "--allow-all" => pf.allow_all = true,
            _ => {}
        }
    }
    pf
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
        assert!(pm.check(PermissionKind::FsRead, "/etc/passwd", "test").is_ok());
        assert!(pm.check(PermissionKind::NetConnect, "evil.com", "test").is_ok());
    }

    #[test]
    fn test_deny_overrides() {
        let pm = PermissionManager::with_flags(PermissionFlags {
            allow_all: true,
            deny_read: vec!["/etc/**".to_string()],
            ..Default::default()
        });
        assert!(pm.check(PermissionKind::FsRead, "/home/user/file", "test").is_ok());
        assert!(pm.check(PermissionKind::FsRead, "/etc/passwd", "test").is_err());
    }

    #[test]
    fn test_explicit_allow() {
        let pm = PermissionManager::with_flags(PermissionFlags {
            allow_read: vec!["/app/**".to_string()],
            ..Default::default()
        });
        assert!(pm.check(PermissionKind::FsRead, "/app/src/main.ts", "test").is_ok());
        assert!(pm.check(PermissionKind::FsRead, "/etc/passwd", "test").is_err());
    }

    #[test]
    fn test_caching() {
        let pm = PermissionManager::with_flags(PermissionFlags {
            allow_read: vec!["/tmp/**".to_string()],
            ..Default::default()
        });
        assert!(pm.check(PermissionKind::FsRead, "/tmp/test.txt", "mod").is_ok());
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
        assert!(pm.check(PermissionKind::FFI, "libc.dylib", "test").is_ok());
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
}
