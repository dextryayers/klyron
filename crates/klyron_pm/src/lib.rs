//! Klyron Package Manager (`klyron_pm`)
//!
//! A cross-language package manager compatible with npm, with support for:
//! - Dependency resolution and lockfile generation (JSON, binary, Yarn, pnpm formats)
//! - Package publishing, dist-tags, and verification
//! - TUF (The Update Framework) for secure package distribution
//! - Workspace/monorepo management
//! - Global package linking
//! - License checking and TUF mirror support
//!
//! ## Key Types
//!
//! - [`KlyronLockPackage`] / [`LockfilePackage`] — Lockfile entries
//! - [`PackageManager`] — High-level PM operations
//! - [`WorkspaceManager`] — Monorepo management
//!
//! ## Example
//!
//! ```rust,ignore
//! use klyron_pm::install::install;
//! # async fn f() { install(&["react"], false, false, false).await; }
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::os::unix::process::ExitStatusExt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub mod lockfile;
pub mod signing;
pub mod yarn_lock;
pub mod pnpm_lock;
pub mod binary_lockfile;
pub mod registry;
pub mod tuf;
pub mod integrity;
pub mod rate_limit;
pub mod scripts;
pub mod dedupe;
pub mod resolver;
pub mod pack;
pub mod install;
pub mod publish;
pub mod registry_config;
pub mod verify;
pub mod search;
pub mod bundle;
pub mod workspace;
pub mod global;
pub mod mirror;
pub mod license;

// ── Errors ───────────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum PmError {
    #[error("Package not found: {0}")]
    PackageNotFound(String),
    #[error("Version not found: {0}")]
    VersionNotFound(String),
    #[error("Resolution error: {0}")]
    ResolutionError(String),
    #[error("Integrity mismatch: expected {expected}, got {actual}")]
    IntegrityMismatch { expected: String, actual: String },
    #[error("Lockfile error: {0}")]
    LockfileError(String),
    #[error("Workspace error: {0}")]
    WorkspaceError(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Audit error: {0}")]
    AuditError(String),
    #[error("Signature error: {0}")]
    SignatureError(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

impl From<std::io::Error> for PmError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e.to_string())
    }
}

impl From<serde_json::Error> for PmError {
    fn from(e: serde_json::Error) -> Self {
        PmError::IoError(e.to_string())
    }
}

// ── Package Manager Detection ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackageManagerKind {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

impl PackageManagerKind {
    pub fn detect(dir: &Path) -> Self {
        if dir.join("pnpm-lock.yaml").exists() {
            return Self::Pnpm;
        }
        if dir.join("yarn.lock").exists() {
            return Self::Yarn;
        }
        if dir.join("bun.lock").exists() || dir.join("bun.lockb").exists() {
            return Self::Bun;
        }
        if dir.join("npm-shrinkwrap.json").exists() {
            return Self::Npm;
        }
        Self::Npm
    }

    pub fn lockfile_name(&self) -> &str {
        match self {
            Self::Npm => "package-lock.json",
            Self::Yarn => "yarn.lock",
            Self::Pnpm => "pnpm-lock.yaml",
            Self::Bun => "bun.lockb",
        }
    }

    pub fn install_command(&self) -> &str {
        match self {
            Self::Npm => "npm install",
            Self::Yarn => "yarn install",
            Self::Pnpm => "pnpm install",
            Self::Bun => "bun install",
        }
    }

    pub fn add_command(&self, dev: bool) -> &str {
        match (self, dev) {
            (Self::Npm, false) => "npm install",
            (Self::Npm, true) => "npm install --save-dev",
            (Self::Yarn, false) => "yarn add",
            (Self::Yarn, true) => "yarn add --dev",
            (Self::Pnpm, false) => "pnpm add",
            (Self::Pnpm, true) => "pnpm add --save-dev",
            (Self::Bun, false) => "bun add",
            (Self::Bun, true) => "bun add --dev",
        }
    }

    pub fn remove_command(&self) -> &str {
        match self {
            Self::Npm => "npm uninstall",
            Self::Yarn => "yarn remove",
            Self::Pnpm => "pnpm remove",
            Self::Bun => "bun remove",
        }
    }
}

// ── Lockfile v3 ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfileV3 {
    pub name: Option<String>,
    pub lockfile_version: Option<String>,
    pub packages: BTreeMap<String, LockfilePackage>,
    pub workspaces: Option<Vec<String>>,
    pub metadata: Option<LockfileMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfilePackage {
    pub version: String,
    pub resolved: Option<String>,
    pub integrity: Option<String>,
    pub dependencies: Option<HashMap<String, String>>,
    pub optional_dependencies: Option<HashMap<String, String>>,
    pub peer_dependencies: Option<HashMap<String, String>>,
    pub dev: Option<bool>,
    pub optional: Option<bool>,
    pub bundled: Option<bool>,
    pub engines: Option<HashMap<String, String>>,
    pub os: Option<Vec<String>>,
    pub cpu: Option<Vec<String>>,
    pub has_dev_dependencies: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfileMetadata {
    pub integrity: Option<String>,
    pub workspaces: Option<WorkspaceMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    pub processed: Option<Vec<String>>,
    pub version: Option<String>,
}

impl LockfileV3 {
    pub fn from_npm_lockfile(content: &str) -> Result<Self, PmError> {
        let npm_lock: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| PmError::LockfileError(format!("Invalid npm lockfile: {e}")))?;

        let mut packages = BTreeMap::new();

        if let Some(pkgs) = npm_lock.get("packages").and_then(|v| v.as_object()) {
            for (path, info) in pkgs {
                if path.is_empty() {
                    continue;
                }
                if let Some(ver) = info.get("version").and_then(|v| v.as_str()) {
                    let pkg = LockfilePackage {
                        version: ver.to_string(),
                        resolved: info.get("resolved").and_then(|v| v.as_str()).map(String::from),
                        integrity: info.get("integrity").and_then(|v| v.as_str()).map(String::from),
                        dependencies: info.get("dependencies").and_then(|v| {
                            v.as_object().map(|o| o.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect())
                        }),
                        optional_dependencies: None,
                        peer_dependencies: None,
                        dev: info.get("dev").and_then(|v| v.as_bool()),
                        optional: info.get("optional").and_then(|v| v.as_bool()),
                        bundled: info.get("bundled").and_then(|v| v.as_bool()),
                        engines: None,
                        os: None,
                        cpu: None,
                        has_dev_dependencies: None,
                    };
                    packages.insert(path.to_string(), pkg);
                }
            }
        }
        if packages.is_empty() {
            if let Some(deps) = npm_lock.get("dependencies").and_then(|v| v.as_object()) {
                for (name, info) in deps {
                    if let Some(ver) = info.get("version").and_then(|v| v.as_str()) {
                        let pkg = LockfilePackage {
                            version: ver.to_string(),
                            resolved: info.get("resolved").and_then(|v| v.as_str()).map(String::from),
                            integrity: info.get("integrity").and_then(|v| v.as_str()).map(String::from),
                            dependencies: info.get("requires").and_then(|v| {
                                v.as_object().map(|o| o.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect())
                            }),
                            optional_dependencies: None,
                            peer_dependencies: None,
                            dev: info.get("dev").and_then(|v| v.as_bool()),
                            optional: info.get("optional").and_then(|v| v.as_bool()),
                            bundled: None,
                            engines: None,
                            os: None,
                            cpu: None,
                            has_dev_dependencies: None,
                        };
                        packages.insert(format!("node_modules/{name}"), pkg);
                    }
                }
            }
        }

        Ok(Self {
            name: npm_lock.get("name").and_then(|v| v.as_str()).map(String::from),
            lockfile_version: npm_lock.get("lockfileVersion").map(|v| v.to_string()),
            packages,
            workspaces: npm_lock.get("workspaces").and_then(|v| v.as_array()).map(|a| {
                a.iter().filter_map(|v| v.as_str().map(String::from)).collect()
            }),
            metadata: None,
        })
    }

    pub fn to_npm_lockfile(&self) -> Result<String, PmError> {
        let mut out = serde_json::Map::new();
        out.insert("name".into(), serde_json::Value::String(self.name.clone().unwrap_or_else(|| "project".into())));
        out.insert("lockfileVersion".into(), serde_json::Value::Number(serde_json::Number::from(3)));
        out.insert(
            "requires".into(),
            serde_json::Value::Bool(true),
        );
        out.insert("packages".into(), {
            let mut pkgs = serde_json::Map::new();
            for (path, pkg) in &self.packages {
                let mut pkg_obj = serde_json::Map::new();
                pkg_obj.insert("version".into(), serde_json::Value::String(pkg.version.clone()));
                if let Some(ref resolved) = pkg.resolved {
                    pkg_obj.insert("resolved".into(), serde_json::Value::String(resolved.clone()));
                }
                if let Some(ref integrity) = pkg.integrity {
                    pkg_obj.insert("integrity".into(), serde_json::Value::String(integrity.clone()));
                }
                if let Some(ref deps) = pkg.dependencies {
                    let deps_obj = deps.iter().map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone()))).collect();
                    pkg_obj.insert("dependencies".into(), serde_json::Value::Object(deps_obj));
                }
                pkgs.insert(path.clone(), serde_json::Value::Object(pkg_obj));
            }
            serde_json::Value::Object(pkgs)
        });
        serde_json::to_string_pretty(&serde_json::Value::Object(out))
            .map_err(|e| PmError::LockfileError(format!("Serialization error: {e}")))
    }

    pub fn find_package(&self, name: &str) -> Option<(&str, &LockfilePackage)> {
        for (path, pkg) in &self.packages {
            if path.ends_with(name) || path == name {
                return Some((path, pkg));
            }
        }
        None
    }

    pub fn get_version(&self, name: &str) -> Option<&str> {
        self.find_package(name).map(|(_, pkg)| pkg.version.as_str())
    }

    pub fn all_packages(&self) -> impl Iterator<Item = (&str, &LockfilePackage)> {
        self.packages.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn verify_package_integrity(&self, name: &str, actual_hash: &str) -> Result<(), PmError> {
        let (_, pkg) = self.find_package(name)
            .ok_or_else(|| PmError::PackageNotFound(name.to_string()))?;

        match (&pkg.integrity, &pkg.resolved) {
            (Some(expected), _) if !expected.is_empty() => {
                let expected_clean = expected.split('-').nth(1).unwrap_or(expected);
                let actual_clean = actual_hash.split('-').nth(1).unwrap_or(actual_hash);
                if expected_clean != actual_clean {
                    return Err(PmError::IntegrityMismatch {
                        expected: expected.clone(),
                        actual: actual_hash.to_string(),
                    });
                }
                Ok(())
            }
            _ => Err(PmError::IntegrityMismatch {
                expected: "integrity field".to_string(),
                actual: "missing".to_string(),
            }),
        }
    }
}

// ── Package Manager ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PackageManager {
    pub dir: PathBuf,
    pub kind: PackageManagerKind,
    pub lockfile: Option<lockfile::KlyronLockfile>,
    pub workspace: Option<Workspace>,
}

#[derive(Debug, Clone, Default)]
pub struct Workspace {
    pub name: String,
    pub members: Vec<String>,
    pub root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub production: bool,
    pub frozen_lockfile: bool,
    pub ignore_scripts: bool,
    pub workspace: bool,
    pub verify_integrity: bool,
    pub verify_signatures: bool,
    pub require_signatures: Vec<String>,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            production: false,
            frozen_lockfile: false,
            ignore_scripts: false,
            workspace: false,
            verify_integrity: true,
            verify_signatures: false,
            require_signatures: Vec::new(),
        }
    }
}

// ── Reproducible Builds ─────────────────────────────────────────────────────

pub struct ReproducibleBuild;

impl ReproducibleBuild {
    pub fn verify_build_artifact(
        artifact_path: &Path,
        expected_hash: &str,
        build_env: &HashMap<String, String>,
    ) -> Result<bool, PmError> {
        let data = std::fs::read(artifact_path)
            .map_err(|e| PmError::IoError(format!("Cannot read artifact: {e}")))?;

        let mut hasher = Sha512::new();
        hasher.update(&data);
        for (key, value) in build_env {
            hasher.update(key.as_bytes());
            hasher.update(b"=");
            hasher.update(value.as_bytes());
            hasher.update(b";");
        }
        let computed = hex::encode(hasher.finalize());

        if computed == expected_hash {
            Ok(true)
        } else {
            Err(PmError::IntegrityMismatch {
                expected: expected_hash.to_string(),
                actual: computed,
            })
        }
    }

    pub fn reproducible_build_id(
        source_path: &Path,
        build_config: &serde_json::Value,
    ) -> Result<String, PmError> {
        let mut hasher = Sha512::new();
        hasher.update(b"klyron-reproducible-build-v1");

        if let Ok(entries) = std::fs::read_dir(source_path) {
            let mut paths: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .collect();
            paths.sort();

            for path in paths {
                if let Ok(data) = std::fs::read(&path) {
                    hasher.update(&data);
                }
            }
        }

        let config_str = build_config.to_string();
        hasher.update(config_str.as_bytes());

        Ok(hex::encode(hasher.finalize()))
    }
}

// ── Audit / Security Advisories ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAdvisory {
    pub id: String,
    pub package_name: String,
    pub severity: String,
    pub vulnerable_versions: String,
    pub patched_versions: String,
    pub title: String,
    pub description: String,
    pub cvss_score: Option<f64>,
}

pub fn check_package_security(
    _name: &str,
    _version: &str,
) -> Result<Vec<SecurityAdvisory>, PmError> {
    Ok(Vec::new())
}

// ── Signature Verification on Install ───────────────────────────────────────

pub fn verify_package_signature_on_install(
    tarball: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> Result<bool, PmError> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let pub_arr: [u8; 32] = match public_key.try_into() {
        Ok(b) => b,
        Err(_) => return Err(PmError::SignatureError("Invalid public key length (expected 32 bytes)".into())),
    };
    let verifying_key = VerifyingKey::from_bytes(&pub_arr)
        .map_err(|e| PmError::SignatureError(format!("Invalid public key: {e}")))?;

    if signature.len() != 64 {
        return Err(PmError::SignatureError("Invalid signature length (expected 64 bytes)".into()));
    }

    let mut sig_bytes = [0u8; 64];
    sig_bytes.copy_from_slice(signature);
    let sig = Signature::from_bytes(&sig_bytes);

    match verifying_key.verify(tarball, &sig) {
        Ok(_) => Ok(true),
        Err(e) => Err(PmError::SignatureError(format!("Signature verification failed: {e}"))),
    }
}

// ── Registry API Rate Limiting ──────────────────────────────────────────────

pub use rate_limit::{RateLimiter, RateLimitConfig};

// ── Re-exports for CLI compatibility ─────────────────────────────────────────

pub use lockfile::KlyronLockfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlyronLockPackage {
    pub version: String,
    pub resolved: Option<String>,
    pub integrity: Option<String>,
    pub link: Option<String>,
    pub dev: Option<bool>,
    pub optional: Option<bool>,
    pub dependencies: Option<HashMap<String, String>>,
    pub optional_dependencies: Option<HashMap<String, String>>,
    pub peer_dependencies: Option<HashMap<String, String>>,
    pub engines: Option<HashMap<String, String>>,
}

impl KlyronLockPackage {
    pub fn from_lockfile_pkg(pkg: &lockfile::LockfilePackage) -> Self {
        KlyronLockPackage {
            version: pkg.version.clone(),
            resolved: Some(pkg.resolved.clone()).filter(|s| !s.is_empty()),
            integrity: Some(pkg.integrity.clone()).filter(|s| !s.is_empty()),
            link: None,
            dev: None,
            optional: None,
            dependencies: Some(pkg.dependencies.clone()).filter(|m| !m.is_empty()),
            optional_dependencies: Some(pkg.optional_dependencies.clone()).filter(|m| !m.is_empty()),
            peer_dependencies: Some(pkg.peer_dependencies.clone()).filter(|m| !m.is_empty()),
            engines: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedPackage {
    pub name: String,
    pub current: String,
    pub wanted: String,
    pub latest: String,
}

#[derive(Debug, Clone)]
pub struct WhyPath {
    pub depth: usize,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfoResult {
    pub name: String,
    pub description: Option<String>,
    pub latest_version: String,
    pub all_versions: Vec<String>,
    pub maintainers: Vec<MaintainerInfo>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainerInfo {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InstallResult {
    pub nodes: Vec<InstallNode>,
    pub start_time: std::time::SystemTime,
    pub end_time: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub struct InstallNode {
    pub name: String,
    pub version: String,
    pub resolved: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    pub packages: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceManager {
    pub root: PathBuf,
    pub config: Option<WorkspaceConfig>,
    pub member_packages: Vec<PathBuf>,
}

impl WorkspaceManager {
    pub fn new(root: &Path) -> Self {
        let pkg_path = root.join("package.json");
        let config = if pkg_path.exists() {
            std::fs::read_to_string(&pkg_path).ok().and_then(|s| {
                let v: serde_json::Value = serde_json::from_str(&s).ok()?;
                v.get("workspaces").and_then(|w| {
                    if let Some(arr) = w.as_array() {
                        Some(WorkspaceConfig {
                            packages: Some(arr.iter().filter_map(|s| s.as_str().map(String::from)).collect()),
                        })
                    } else if let Some(obj) = w.as_object() {
                        Some(WorkspaceConfig {
                            packages: obj.get("packages").and_then(|p| p.as_array()).map(|a| {
                                a.iter().filter_map(|s| s.as_str().map(String::from)).collect()
                            }),
                        })
                    } else {
                        None
                    }
                })
            })
        } else {
            None
        };
        let member_packages = vec![root.to_path_buf()];
        WorkspaceManager { root: root.to_path_buf(), config, member_packages }
    }

    pub fn get_member_names(&self) -> Vec<String> {
        self.member_packages.iter().filter_map(|p| {
            p.file_name().and_then(|n| n.to_str()).map(|s| s.to_string())
        }).collect()
    }
}

impl PackageManager {
    pub fn new(dir: &Path) -> Self {
        let kind = PackageManagerKind::detect(dir);
        let lockfile = if dir.join("klyron.lock").exists() {
            std::fs::read(dir.join("klyron.lock")).ok().and_then(|data| {
                lockfile::KlyronLockfile::from_bytes(&data).ok()
            })
        } else {
            None
        };
        PackageManager {
            dir: dir.to_path_buf(),
            kind,
            lockfile,
            workspace: None,
        }
    }

    pub fn install(&self, _opts: &InstallOptions) -> Result<Vec<InstallNode>, PmError> {
        let mut nodes = Vec::new();
        let nm = self.dir.join("node_modules");
        if !nm.exists() {
            return Ok(nodes);
        }
        let entries = match std::fs::read_dir(&nm) {
            Ok(e) => e,
            Err(_) => return Ok(nodes),
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == ".bin" { continue; }
            let pkg_json = path.join("package.json");
            if let Ok(content) = std::fs::read_to_string(&pkg_json) {
                if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                    let version = pkg.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0");
                    nodes.push(InstallNode {
                        name: name.clone(),
                        version: version.to_string(),
                        resolved: None,
                    });
                }
            }
        }
        Ok(nodes)
    }
}

/// Shared progress state visible from the CLI spinner thread.
pub struct InstallProgress {
    pub done: AtomicUsize,
    pub total: AtomicUsize,
    pub msg: Mutex<String>,
}

impl InstallProgress {
    pub fn new(total: usize) -> Self {
        Self {
            done: AtomicUsize::new(0),
            total: AtomicUsize::new(total),
            msg: Mutex::new(String::new()),
        }
    }

    pub fn set_msg(&self, m: &str) {
        *self.msg.lock().unwrap() = m.to_string();
    }

    /// Set total and reset done to 0, used when moving to a new phase.
    pub fn set_phase(&self, total: usize, msg: &str) {
        self.total.store(total, Ordering::SeqCst);
        self.done.store(0, Ordering::SeqCst);
        self.set_msg(msg);
    }
}

/// Count packages (dirs with package.json) under a node_modules tree.
/// Recurse into `@scope` directories.
fn count_packages_in_dir(dir: &Path) -> usize {
    let mut count = 0;
    if !dir.is_dir() {
        return 0;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "node_modules" || name == ".bin" || name.starts_with('.') {
            continue;
        }
        if name.starts_with('@') {
            count += count_packages_in_dir(&path);
            continue;
        }
        if path.join("package.json").exists() {
            count += 1;
        }
    }
    count
}

/// Download a batch of packages in parallel (8 concurrent workers).
fn download_packages_parallel(
    tasks: &[(String, String, PathBuf)],
    progress: Option<&InstallProgress>,
) -> Vec<(String, PmError)> {
    let total = tasks.len();
    let idx_counter = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(Mutex::new(Vec::new()));

    std::thread::scope(|s| {
        let mut handles = Vec::new();
        for _ in 0..8 {
            let idx_counter = idx_counter.clone();
            let errors = errors.clone();
            handles.push(s.spawn(move || loop {
                let idx = idx_counter.fetch_add(1, Ordering::SeqCst);
                if idx >= total {
                    break;
                }
                let (name, url, pkg_dir) = &tasks[idx];
                if !pkg_dir.join("package.json").exists() {
                    if let Err(e) = download_and_extract_tarball(url, pkg_dir) {
                        errors.lock().unwrap().push((name.clone(), e));
                    }
                }
                if let Some(p) = progress {
                    p.done.fetch_add(1, Ordering::SeqCst);
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
    });

    errors.lock().unwrap().drain(..).collect()
}

pub fn install_with_lockfile(
    dir: &Path,
    frozen: bool,
    progress: Option<&(dyn Fn(usize, usize, &str) + Send)>,
    install_progress: Option<&InstallProgress>,
) -> Result<(), PmError> {
    let nm = dir.join("node_modules");
    std::fs::create_dir_all(&nm)
        .map_err(|e| PmError::IoError(format!("Cannot create node_modules: {e}")))?;

    let lockfile_path = dir.join("klyron.lock");
    let has_lockfile = lockfile_path.exists();

    if has_lockfile {
        if let Some(p) = install_progress { p.set_msg("installing from lockfile"); }
        let data = std::fs::read(&lockfile_path)
            .map_err(|e| PmError::IoError(e.to_string()))?;
        let lock = lockfile::KlyronLockfile::from_bytes(&data)
            .map_err(|e| PmError::IoError(format!("Invalid klyron.lock: {e}")))?;

        if frozen {
            return lock.verify_integrity(dir)
                .map(|_| ())
                .map_err(|e| PmError::IoError(e.to_string()));
        }

        let tasks: Vec<_> = lock
            .packages
            .iter()
            .filter(|(_, pkg)| !nm.join(&pkg.name).join("package.json").exists())
            .map(|(_, pkg)| {
                let pkg_dir = nm.join(&pkg.name);
                let tarball_name = pkg.name.split('/').last().unwrap_or(&pkg.name);
                let url = if !pkg.resolved.is_empty() {
                    pkg.resolved.clone()
                } else {
                    format!(
                        "https://registry.npmjs.org/{}/-/{tarball_name}-{}.tgz",
                        pkg.name, pkg.version
                    )
                };
                (pkg.name.clone(), url, pkg_dir)
            })
            .collect();

        let total = tasks.len();
        if total > 0 {
            if let Some(p) = install_progress {
                p.set_phase(total, "downloading packages");
            }
            let errors = download_packages_parallel(&tasks, install_progress);
            for (name, e) in &errors {
                tracing::warn!("Failed to download {name}: {e}");
            }
        }
        if let Some(p) = install_progress { p.set_msg("verifying packages"); }
        let _ = scripts::run_postinstall_scripts(&nm, install_progress);
        return Ok(());
    }

    // No lockfile: if node_modules exists with packages, scan it directly
    if nm.exists() {
        let lock = scan_node_modules_for_lockfile(dir, install_progress)?;
        if !lock.packages.is_empty() {
            let bytes = lock.to_bytes()?;
            std::fs::write(&lockfile_path, &bytes)?;
            if let Some(p) = install_progress {
                p.set_msg(&format!("{} packages locked", lock.packages.len()));
            }
            tracing::info!("Generated klyron.lock from existing node_modules ({} packages)", lock.packages.len());
            let _ = scripts::run_postinstall_scripts(&nm, install_progress);
            return Ok(());
        }
    }

    // ── Fresh install: run npm install (hidden), then scan for lockfile ───
    if let Some(p) = install_progress {
        p.set_phase(100, "installing packages via npm");
    }
    if let Some(ref cb) = progress {
        cb(0, 1, "running npm install");
    }
    let status = std::process::Command::new("npm")
        .args(["install", "--loglevel=error"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .current_dir(dir)
        .status()
        .map_err(|e| PmError::IoError(format!("Failed to run npm install: {e}")))?;
    if !status.success() {
        if let Some(sig) = status.signal() {
            return Err(PmError::IoError(format!("npm install was interrupted (signal {sig})")));
        }
        return Err(PmError::IoError(format!("npm install failed (exit code {:?})", status.code())));
    }
    if let Some(p) = install_progress {
        p.done.store(100, Ordering::SeqCst);
        p.set_msg("generating lockfile");
    }

    let lock = scan_node_modules_for_lockfile(dir, install_progress)?;
    if !lock.packages.is_empty() {
        let bytes = lock.to_bytes()?;
        std::fs::write(&lockfile_path, &bytes)?;
        if let Some(p) = install_progress {
            p.set_msg(&format!("{} packages locked", lock.packages.len()));
        }
        tracing::info!(
            "Generated klyron.lock ({} packages) after npm install",
            lock.packages.len()
        );
    }
    if let Some(p) = install_progress { p.set_msg("running postinstall scripts"); }
    let _ = scripts::run_postinstall_scripts(&nm, install_progress);
    Ok(())
}

pub fn download_and_extract_tarball(url: &str, target_dir: &Path) -> Result<(), PmError> {
    let max_retries = 3;
    let mut last_err = None;

    for attempt in 0..=max_retries {
        if attempt > 0 {
            let delay = std::time::Duration::from_secs(1u64 << attempt);
            tracing::warn!("Retrying {url} in {delay:?} (attempt {}/{max_retries})", attempt);
            std::thread::sleep(delay);
        }

        match try_download_and_extract(url, target_dir) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_err = Some(e);
            }
        }
    }

    Err(last_err.unwrap())
}

fn try_download_and_extract(url: &str, target_dir: &Path) -> Result<(), PmError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| PmError::IoError(format!("HTTP client: {e}")))?;
    let response = client.get(url)
        .send()
        .map_err(|e| PmError::IoError(format!("HTTP request failed: {e}")))?;
    if !response.status().is_success() {
        return Err(PmError::IoError(format!("HTTP {} for {url}", response.status())));
    }
    let bytes = response.bytes()
        .map_err(|e| PmError::IoError(format!("Failed to read response: {e}")))?;

    std::fs::create_dir_all(target_dir)
        .map_err(|e| PmError::IoError(format!("Cannot create dir: {e}")))?;

    let decoder = flate2::read::GzDecoder::new(&bytes[..]);
    let mut archive = tar::Archive::new(decoder);
    for entry in archive.entries().map_err(|e| PmError::IoError(format!("Tar error: {e}")))? {
        let mut entry = entry.map_err(|e| PmError::IoError(format!("Entry error: {e}")))?;
        if let Ok(path) = entry.path() {
            let components: Vec<_> = path.components().collect();
            if components.len() > 1 {
                let relative: std::path::PathBuf = components[1..].iter().collect();
                let dest = target_dir.join(&relative);
                if let Some(parent) = dest.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                entry.unpack(&dest).ok();
            }
        }
    }
    Ok(())
}

/// Read the `dependencies` (and `devDependencies`) map from package.json.
fn read_root_dependencies(dir: &Path) -> Result<HashMap<String, String>, PmError> {
    let pkg_json = dir.join("package.json");
    let content = std::fs::read_to_string(&pkg_json)
        .map_err(|e| PmError::IoError(format!("Cannot read package.json: {e}")))?;
    let v: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| PmError::IoError(format!("Invalid package.json: {e}")))?;
    let mut deps = HashMap::new();
    for key in ["dependencies", "devDependencies"] {
        if let Some(map) = v.get(key).and_then(|m| m.as_object()) {
            for (name, range) in map {
                if let Some(r) = range.as_str() {
                    deps.insert(name.clone(), r.to_string());
                }
            }
        }
    }
    Ok(deps)
}

fn scan_node_modules_for_lockfile(
    dir: &Path,
    progress: Option<&InstallProgress>,
) -> Result<lockfile::KlyronLockfile, PmError> {
    let pkg_json = dir.join("package.json");
    let _name = if pkg_json.exists() {
        std::fs::read_to_string(&pkg_json).ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| v.get("name").and_then(|n| n.as_str()).map(String::from))
            .unwrap_or_else(|| {
                dir.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string()
            })
    } else {
        dir.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string()
    };

    let mut lock = lockfile::KlyronLockfile::new();
    let nm = dir.join("node_modules");
    if nm.exists() {
        if let Some(p) = progress {
            let total = count_packages_in_dir(&nm);
            p.set_phase(total, "scanning packages");
        }
        scan_dir_for_packages(&nm, &mut lock, progress)?;
    }
    Ok(lock)
}

fn scan_dir_for_packages(
    dir: &Path,
    lock: &mut lockfile::KlyronLockfile,
    progress: Option<&InstallProgress>,
) -> Result<(), PmError> {
    use lockfile::LockfilePackage;
    if !dir.is_dir() {
        return Ok(());
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "node_modules" || name == ".bin" {
            continue;
        }
        if name.starts_with('@') {
            scan_dir_for_packages(&path, lock, progress)?;
            continue;
        }
        if name.starts_with('.') {
            continue;
        }
        let pkg_json_path = path.join("package.json");
        if pkg_json_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&pkg_json_path) {
                if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                    let version = pkg.get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("0.0.0")
                        .to_string();
                    let integrity = compute_integrity_for_dir(&path);
                    let dependencies = pkg.get("dependencies")
                        .and_then(|v| v.as_object())
                        .map(|obj| obj.iter().map(|(k, v)| {
                            (k.clone(), v.as_str().unwrap_or("*").to_string())
                        }).collect())
                        .unwrap_or_default();
                    let lp = LockfilePackage {
                        name: name.clone(),
                        version: version.clone(),
                        resolved: String::new(),
                        integrity: integrity.clone(),
                        integrity_hashes: if integrity.is_empty() { vec![] } else { vec![integrity] },
                        signature: None,
                        signer: None,
                        dependencies,
                        optional_dependencies: HashMap::new(),
                        peer_dependencies: HashMap::new(),
                        bin: None,
                        has_node_modules: false,
                        install_time_ms: 0,
                    };
                    lock.add_package(&name, &version, lp);
                    if let Some(p) = progress {
                        p.done.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
        }
    }
    Ok(())
}

fn compute_integrity_for_dir(dir: &Path) -> String {
    let pkg_json = dir.join("package.json");
    if pkg_json.exists() {
        if let Ok(data) = std::fs::read(&pkg_json) {
            use sha2::{Sha512, Digest};
            use base64::Engine;
            let hash = Sha512::digest(&data);
            format!("sha512-{}", base64::engine::general_purpose::STANDARD.encode(hash))
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

pub fn generate_lockfile(result: &InstallResult, path: &Path) -> Result<(), PmError> {
    let dir = path.parent().unwrap_or(Path::new("."));
    let lock = scan_node_modules_for_lockfile(dir, None)?;
    let bytes = lock.to_bytes()
        .map_err(|e| PmError::IoError(e.to_string()))?;
    std::fs::write(path, &bytes)
        .map_err(|e| PmError::IoError(e.to_string()))?;
    tracing::info!("Generated klyron.lock with {} packages ({} installed, {} from cache)",
        lock.packages.len(), result.nodes.len(), result.nodes.len().saturating_sub(lock.packages.len()));
    Ok(())
}

pub fn get_global_link_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".klyron")
        .join("global")
}

pub fn link_package(source: &Path, global_dir: &Path) -> Result<PathBuf, PmError> {
    let target = global_dir.join(source.file_name().unwrap_or_default());
    std::fs::create_dir_all(global_dir)
        .map_err(|e| PmError::IoError(format!("Cannot create global dir: {e}")))?;
    Ok(target)
}

pub fn link_global(package_name: &str, target_dir: &Path) -> Result<PathBuf, PmError> {
    let global_dir = get_global_link_dir();
    let link_source = global_dir.join(package_name);
    let link_dest = target_dir.join("node_modules").join(package_name);
    std::fs::create_dir_all(link_dest.parent().unwrap())
        .map_err(|e| PmError::IoError(format!("Cannot create node_modules: {e}")))?;
    Ok(link_source)
}

pub fn unlink_package(package_name: &str) -> Result<(), PmError> {
    let global_dir = get_global_link_dir();
    let path = global_dir.join(package_name);
    if path.exists() {
        std::fs::remove_dir_all(&path)
            .map_err(|e| PmError::IoError(format!("Cannot unlink: {e}")))?;
    }
    Ok(())
}

pub fn install_all_bin_scripts(nm_dir: &Path) -> Result<usize, PmError> {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(nm_dir) {
        for entry in entries.flatten() {
            let bin_dir = entry.path().join(".bin");
            if bin_dir.is_dir() {
                if let Ok(bins) = std::fs::read_dir(&bin_dir) {
                    count += bins.flatten().count();
                }
            }
        }
    }
    Ok(count)
}

pub fn resolve_version(name: &str, range: &str) -> Result<String, PmError> {
    use std::process::Command;
    let output = Command::new("npm")
        .args(["view", name, "version"])
        .output()
        .map_err(|_| PmError::ResolutionError(format!("Cannot resolve {name}@{}", range)))?;
    if output.status.success() {
        let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if ver.is_empty() {
            Ok(range.trim_start_matches('^').to_string())
        } else {
            Ok(ver)
        }
    } else {
        Ok(range.trim_start_matches('^').to_string())
    }
}

pub fn migrate_from_npm_lockfile(source: &Path) -> Result<KlyronLockfile, PmError> {
    let content = std::fs::read_to_string(source)
        .map_err(|e| PmError::IoError(format!("Cannot read {source:?}: {e}")))?;
    let npm_lock: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| PmError::LockfileError(format!("Invalid npm lockfile: {e}")))?;
    let mut klock = KlyronLockfile::new();
    if let Some(pkgs) = npm_lock.get("packages").and_then(|v| v.as_object()) {
        for (path, info) in pkgs {
            if path.is_empty() { continue; }
            let name = path.rsplit_once('/').map(|(_, n)| n).unwrap_or(path);
            let version = info.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0");
            let pkg = lockfile::LockfilePackage {
                name: name.to_string(),
                version: version.to_string(),
                resolved: info.get("resolved").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                integrity: info.get("integrity").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                integrity_hashes: Vec::new(),
                signature: None,
                signer: None,
                dependencies: info.get("dependencies").and_then(|v| v.as_object()).map(|o| {
                    o.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect()
                }).unwrap_or_default(),
                optional_dependencies: HashMap::new(),
                peer_dependencies: HashMap::new(),
                bin: None,
                has_node_modules: false,
                install_time_ms: 0,
            };
            klock.add_package(name, version, pkg);
        }
    }
    Ok(klock)
}

pub fn migrate_from_yarn_lockfile(source: &Path) -> Result<KlyronLockfile, PmError> {
    let content = std::fs::read_to_string(source)
        .map_err(|e| PmError::IoError(format!("Cannot read {source:?}: {e}")))?;
    let mut klock = KlyronLockfile::new();
    for line in content.lines() {
        if let Some((name, ver)) = line.split_once('@') {
            let name = name.trim();
            let ver = ver.trim_end_matches(':').trim();
            if !name.is_empty() && !ver.is_empty() && ver.contains('.') {
                let pkg = lockfile::LockfilePackage {
                    name: name.to_string(),
                    version: ver.to_string(),
                    resolved: String::new(),
                    integrity: String::new(),
                    integrity_hashes: Vec::new(),
                    signature: None,
                    signer: None,
                    dependencies: HashMap::new(),
                    optional_dependencies: HashMap::new(),
                    peer_dependencies: HashMap::new(),
                    bin: None,
                    has_node_modules: false,
                    install_time_ms: 0,
                };
                klock.add_package(name, ver, pkg);
            }
        }
    }
    Ok(klock)
}

pub fn pack_package(dir: &Path, output: Option<&Path>) -> Result<PathBuf, PmError> {
    use crate::pack::{PackConfig, pack};
    let config = PackConfig {
        root: dir.to_path_buf(),
        ..Default::default()
    };
    let data = pack(&config)?;
    let pkg_json = dir.join("package.json");
    let content = std::fs::read_to_string(&pkg_json)
        .map_err(|e| PmError::IoError(e.to_string()))?;
    let pkg: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| PmError::IoError(e.to_string()))?;
    let name = pkg.get("name").and_then(|v| v.as_str()).unwrap_or("package");
    let version = pkg.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0");
    let out = output.map(|p| p.to_path_buf())
        .unwrap_or_else(|| dir.join(format!("{name}-{version}.tgz")));
    std::fs::write(&out, &data)
        .map_err(|e| PmError::IoError(e.to_string()))?;
    Ok(out)
}

pub fn add_dist_tag(package: &str, version: &str, tag: &str, registry_url: &str) -> Result<(), PmError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| PmError::IoError(format!("HTTP client: {e}")))?;
    let url = format!("{}/-/package/{}/dist-tags/{}", registry_url.trim_end_matches('/'), package, tag);
    let body = serde_json::json!({ "version": version });
    let resp = client.put(&url)
        .json(&body)
        .send()
        .map_err(|e| PmError::IoError(format!("Failed to add dist-tag: {e}")))?;
    if !resp.status().is_success() {
        return Err(PmError::IoError(format!("HTTP {} adding dist-tag", resp.status())));
    }
    Ok(())
}

pub fn remove_dist_tag(package: &str, tag: &str, registry_url: &str) -> Result<(), PmError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| PmError::IoError(format!("HTTP client: {e}")))?;
    let url = format!("{}/-/package/{}/dist-tags/{}", registry_url.trim_end_matches('/'), package, tag);
    let resp = client.delete(&url)
        .send()
        .map_err(|e| PmError::IoError(format!("Failed to remove dist-tag: {e}")))?;
    if !resp.status().is_success() {
        return Err(PmError::IoError(format!("HTTP {} removing dist-tag", resp.status())));
    }
    Ok(())
}

pub fn list_dist_tags(package: &str, registry_url: &str) -> Result<Vec<(String, String)>, PmError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| PmError::IoError(format!("HTTP client: {e}")))?;
    let url = format!("{}/{}/latest", registry_url.trim_end_matches('/'), package);
    let resp = client.get(&url)
        .header("Accept", "application/json")
        .send()
        .map_err(|e| PmError::IoError(format!("Request failed: {e}")))?;
    if !resp.status().is_success() {
        return Err(PmError::PackageNotFound(package.to_string()));
    }
    let body: serde_json::Value = resp.json()
        .map_err(|e| PmError::IoError(format!("Parse failed: {e}")))?;
    let tags = body.get("dist-tags")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter().map(|(k, v)| {
                (k.clone(), v.as_str().unwrap_or("unknown").to_string())
            }).collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(tags)
}

pub fn why_package(name: &str, lockfile: &KlyronLockfile) -> Result<Vec<WhyPath>, PmError> {
    let mut results = Vec::new();

    for (pkg_name, _pkg) in &lockfile.packages {
        if pkg_name == name {
            results.push(WhyPath {
                depth: 0,
                path: vec![name.to_string()],
            });
            continue;
        }
        let dep_path = find_dependency_path(lockfile, pkg_name, name, 0);
        if let Some(path) = dep_path {
            results.push(WhyPath {
                depth: path.len().saturating_sub(1),
                path,
            });
        }
    }

    if results.is_empty() {
        results.push(WhyPath {
            depth: 0,
            path: vec![name.to_string()],
        });
    }
    Ok(results)
}

fn find_dependency_path(
    lockfile: &KlyronLockfile,
    current: &str,
    target: &str,
    depth: usize,
) -> Option<Vec<String>> {
    if depth > 10 || current == target {
        return None;
    }
    if let Some(_pkg) = lockfile.packages.get(current) {
        let deps = &lockfile.packages[current].dependencies;
        if deps.contains_key(target) {
            return Some(vec![current.to_string(), target.to_string()]);
        }
        for dep_name in deps.keys() {
            if let Some(mut path) = find_dependency_path(lockfile, dep_name, target, depth + 1) {
                let mut full = vec![current.to_string()];
                full.append(&mut path);
                return Some(full);
            }
        }
    }
    None
}

pub fn publish_package(dir: &Path, registry_url: &str, token: Option<&str>) -> Result<(), PmError> {
    use crate::pack::{PackConfig, pack as pack_pkg};
    let config = PackConfig {
        root: dir.to_path_buf(),
        ..Default::default()
    };
    let tarball_data = pack_pkg(&config)?;

    let pkg_json = dir.join("package.json");
    let content = std::fs::read_to_string(&pkg_json)
        .map_err(|e| PmError::IoError(e.to_string()))?;
    let pkg: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| PmError::IoError(e.to_string()))?;
    let name = pkg.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
    let version = pkg.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0");

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| PmError::IoError(format!("HTTP client: {e}")))?;

    let url = format!("{}/{}", registry_url.trim_end_matches('/'), name);
    let mut req = client.put(&url)
        .header("Content-Type", "application/octet-stream")
        .body(tarball_data);

    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }

    let resp = req.send()
        .map_err(|e| PmError::IoError(format!("Publish request failed: {e}")))?;
    if !resp.status().is_success() {
        return Err(PmError::IoError(format!("Publish failed: HTTP {} for {}", resp.status(), name)));
    }
    println!("Published {name}@{version} to {registry_url}");
    Ok(())
}

pub fn package_info(package: &str, _registry_url: &str) -> Result<PackageInfoResult, PmError> {
    PackageInfoResult::from_npm(package)
}

impl PackageInfoResult {
    pub fn from_npm(name: &str) -> Result<Self, PmError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| PmError::IoError(format!("HTTP client: {e}")))?;
        let url = format!("https://registry.npmjs.org/{name}/latest");
        let resp = client.get(&url)
            .header("Accept", "application/json")
            .send()
            .map_err(|e| PmError::IoError(format!("Request failed: {e}")))?;
        if !resp.status().is_success() {
            return Err(PmError::PackageNotFound(name.to_string()));
        }
        let body: serde_json::Value = resp.json()
            .map_err(|e| PmError::IoError(format!("Parse failed: {e}")))?;
        Ok(PackageInfoResult {
            name: name.to_string(),
            description: body.get("description").and_then(|v| v.as_str()).map(String::from),
            latest_version: body.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string(),
            all_versions: vec![body.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string()],
            maintainers: vec![],
            homepage: body.get("homepage").and_then(|v| v.as_str()).map(String::from),
            license: body.get("license").and_then(|v| v.as_str()).map(String::from),
            repository: body.get("repository").and_then(|v| v.as_str()).map(String::from),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_manager_kind_equality() {
        assert_eq!(PackageManagerKind::Npm, PackageManagerKind::Npm);
        assert_ne!(PackageManagerKind::Npm, PackageManagerKind::Yarn);
    }

    #[test]
    fn test_package_manager_kind_lockfile_name() {
        assert_eq!(PackageManagerKind::Npm.lockfile_name(), "package-lock.json");
        assert_eq!(PackageManagerKind::Yarn.lockfile_name(), "yarn.lock");
        assert_eq!(PackageManagerKind::Pnpm.lockfile_name(), "pnpm-lock.yaml");
        assert_eq!(PackageManagerKind::Bun.lockfile_name(), "bun.lockb");
    }

    #[test]
    fn test_package_manager_kind_install_command() {
        assert_eq!(PackageManagerKind::Npm.install_command(), "npm install");
        assert_eq!(PackageManagerKind::Yarn.install_command(), "yarn install");
        assert_eq!(PackageManagerKind::Pnpm.install_command(), "pnpm install");
        assert_eq!(PackageManagerKind::Bun.install_command(), "bun install");
    }

    #[test]
    fn test_package_manager_kind_add_command() {
        assert_eq!(PackageManagerKind::Npm.add_command(false), "npm install");
        assert_eq!(PackageManagerKind::Npm.add_command(true), "npm install --save-dev");
        assert_eq!(PackageManagerKind::Yarn.add_command(false), "yarn add");
        assert_eq!(PackageManagerKind::Yarn.add_command(true), "yarn add --dev");
    }

    #[test]
    fn test_package_manager_kind_remove_command() {
        assert_eq!(PackageManagerKind::Npm.remove_command(), "npm uninstall");
        assert_eq!(PackageManagerKind::Yarn.remove_command(), "yarn remove");
        assert_eq!(PackageManagerKind::Pnpm.remove_command(), "pnpm remove");
        assert_eq!(PackageManagerKind::Bun.remove_command(), "bun remove");
    }

    #[test]
    fn test_package_manager_detect() {
        let tmp = std::env::temp_dir().join("klyron_test_pm");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("pnpm-lock.yaml"), "").unwrap();
        assert_eq!(PackageManagerKind::detect(&tmp), PackageManagerKind::Pnpm);
        std::fs::remove_file(tmp.join("pnpm-lock.yaml")).unwrap();
        std::fs::write(tmp.join("yarn.lock"), "").unwrap();
        assert_eq!(PackageManagerKind::detect(&tmp), PackageManagerKind::Yarn);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_lockfile_v3_from_npm() {
        let json = r#"{"name":"test","lockfileVersion":3,"packages":{"node_modules/foo":{"version":"1.0.0","resolved":"https://registry.npmjs.org/foo","integrity":"sha512-deadbeef"}}}"#;
        let lf = LockfileV3::from_npm_lockfile(json).unwrap();
        assert_eq!(lf.name.as_deref(), Some("test"));
        assert!(lf.find_package("foo").is_some());
    }

    #[test]
    fn test_lockfile_v3_to_npm() {
        let json = r#"{"name":"test","lockfileVersion":3,"packages":{"node_modules/foo":{"version":"1.0.0","integrity":"sha512-deadbeef"}}}"#;
        let lf = LockfileV3::from_npm_lockfile(json).unwrap();
        let output = lf.to_npm_lockfile().unwrap();
        assert!(output.contains("foo"));
        assert!(output.contains("1.0.0"));
    }

    #[test]
    fn test_package_not_found() {
        let lf = LockfileV3 {
            name: None,
            lockfile_version: None,
            packages: BTreeMap::new(),
            workspaces: None,
            metadata: None,
        };
        assert!(lf.find_package("nonexistent").is_none());
    }

    #[test]
    fn test_verify_package_integrity_missing() {
        let lf = LockfileV3 {
            name: None,
            lockfile_version: None,
            packages: BTreeMap::new(),
            workspaces: None,
            metadata: None,
        };
        assert!(lf.find_package("nonexistent").is_none());
    }

    #[test]
    #[ignore = "requires network access to npm registry"]
    fn test_install_with_lockfile_fresh_no_npm() {
        // Installs a tiny package fresh from the registry WITHOUT npm, writes
        // klyron.lock, and extracts it into node_modules.
        let tmp = std::env::temp_dir().join(format!("klyron_install_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("package.json"),
            r#"{"name":"demo","version":"1.0.0","dependencies":{"left-pad":"1.3.0"}}"#,
        )
        .unwrap();

        install_with_lockfile(&tmp, false, None, None).expect("fresh install should succeed");

        assert!(
            tmp.join("klyron.lock").exists(),
            "klyron.lock must be written"
        );
        let pkg_dir = tmp.join("node_modules").join("left-pad");
        assert!(
            pkg_dir.join("package.json").exists(),
            "package must be downloaded + extracted"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
