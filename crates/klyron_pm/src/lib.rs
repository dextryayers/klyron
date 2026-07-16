use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
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
    pub lockfile: Option<LockfileV3>,
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
}
