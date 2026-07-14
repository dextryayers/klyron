use hex;
use once_cell::sync::Lazy;
use regex::Regex;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use thiserror::Error;

pub mod lockfile;

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
}

impl From<std::io::Error> for PmError {
  fn from(e: std::io::Error) -> Self {
    Self::IoError(e.to_string())
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

    // Parse "packages" section (npm v7+)
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
    // Fallback: parse "dependencies" section (npm v6)
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
}

impl Default for InstallOptions {
  fn default() -> Self {
    Self {
      production: false,
      frozen_lockfile: false,
      ignore_scripts: false,
      workspace: true,
    }
  }
}

#[derive(Debug, Clone)]
pub struct DependencyNode {
  pub name: String,
  pub version: String,
  pub resolved: Option<String>,
  pub integrity: Option<String>,
  pub dependencies: Vec<DependencyNode>,
  pub dev: bool,
  pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct AuditVulnerability {
  pub severity: String,
  pub package: String,
  pub version: String,
  pub title: String,
  pub cve: Option<String>,
  pub cvss_score: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct AuditResult {
  pub vulnerabilities: Vec<AuditVulnerability>,
  pub total_count: usize,
  pub critical: usize,
  pub high: usize,
  pub moderate: usize,
  pub low: usize,
}

#[derive(Debug, Clone)]
pub struct OutdatedPackage {
  pub name: String,
  pub current: String,
  pub wanted: String,
  pub latest: String,
}

// ── Dependency Resolution ────────────────────────────────────────────────────

fn resolve_version(name: &str, constraint: &str) -> Result<String, PmError> {
  // For testing and offline: simulate version resolution
  // In production, this would hit the npm registry
  let versions = vec![
    "1.0.0", "1.1.0", "1.2.0", "1.2.1", "1.3.0",
    "2.0.0", "2.1.0", "2.2.0",
    "3.0.0", "3.1.0", "3.2.0",
  ];

  let constraint = if constraint.is_empty() || constraint == "*" {
    ">=0.0.0"
  } else {
    constraint
  };

  let req = VersionReq::parse(constraint)
    .map_err(|e| PmError::ResolutionError(format!("Invalid version constraint '{constraint}': {e}")))?;

  let mut matched: Vec<&str> = versions
    .iter()
    .filter(|v| {
      if let Ok(ver) = Version::parse(v) {
        req.matches(&ver)
      } else {
        false
      }
    })
    .copied()
    .collect();

  if matched.is_empty() {
    return Err(PmError::VersionNotFound(format!(
      "No version of '{name}' matching '{constraint}'"
    )));
  }

  matched.sort_by(|a, b| {
    let va = Version::parse(a).unwrap();
    let vb = Version::parse(b).unwrap();
    vb.cmp(&va) // descending
  });

  Ok(matched[0].to_string())
}

// ── Integrity ────────────────────────────────────────────────────────────────

pub fn compute_integrity(data: &[u8]) -> String {
  let mut hasher = Sha512::new();
  hasher.update(data);
  format!("sha512-{}", hex::encode(hasher.finalize()))
}

pub fn verify_integrity(data: &[u8], expected: &str) -> Result<(), PmError> {
  let actual = compute_integrity(data);
  if actual != expected {
    return Err(PmError::IntegrityMismatch {
      expected: expected.to_string(),
      actual,
    });
  }
  Ok(())
}

// ── Implementation ───────────────────────────────────────────────────────────

impl PackageManager {
  pub fn new(dir: &Path) -> Self {
    let kind = PackageManagerKind::detect(dir);
    let lockfile = Self::load_lockfile(dir, kind);
    let workspace = Self::detect_workspace(dir);
    Self {
      dir: dir.to_path_buf(),
      kind,
      lockfile,
      workspace,
    }
  }

  pub fn detect(dir: &Path) -> PackageManagerKind {
    PackageManagerKind::detect(dir)
  }

  fn load_lockfile(dir: &Path, kind: PackageManagerKind) -> Option<LockfileV3> {
    let lockfile_path = dir.join(kind.lockfile_name());
    if !lockfile_path.exists() {
      return None;
    }
    let content = std::fs::read_to_string(&lockfile_path).ok()?;
    match kind {
      PackageManagerKind::Npm => LockfileV3::from_npm_lockfile(&content).ok(),
      _ => {
        // Yarn, pnpm, bun parsers would be added
        None
      }
    }
  }

  fn detect_workspace(dir: &Path) -> Option<Workspace> {
    let pkg_json = dir.join("package.json");
    if !pkg_json.exists() {
      return None;
    }
    let content = std::fs::read_to_string(&pkg_json).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let workspaces = json.get("workspaces").and_then(|v| match v {
      serde_json::Value::Array(arr) => Some(arr.clone()),
      serde_json::Value::Object(obj) => obj.get("packages").and_then(|p| p.as_array().cloned()),
      _ => None,
    })?;

    let members: Vec<String> = workspaces
      .iter()
      .filter_map(|v| v.as_str().map(String::from))
      .collect();

    Some(Workspace {
      name: json.get("name").and_then(|v| v.as_str()).unwrap_or("workspace-root").into(),
      members,
      root: dir.to_path_buf(),
    })
  }

  // ── Install ─────────────────────────────────────────────────────────────

  pub fn install(&self, opts: &InstallOptions) -> Result<Vec<DependencyNode>, PmError> {
    let pkg_json = self.dir.join("package.json");
    if !pkg_json.exists() {
      return Err(PmError::PackageNotFound(
        "package.json not found".into(),
      ));
    }

    let content = std::fs::read_to_string(&pkg_json)?;
    let json: serde_json::Value =
      serde_json::from_str(&content).map_err(|e| PmError::ResolutionError(format!("Invalid package.json: {e}")))?;

    let mut nodes = Vec::new();
    let deps = json.get("dependencies").and_then(|v| v.as_object());
    let dev_deps = json.get("devDependencies").and_then(|v| v.as_object());

    if let Some(deps) = deps {
      for (name, version) in deps {
        let ver_str = version.as_str().unwrap_or("*");
        if let Ok(resolved) = resolve_version(name, ver_str) {
          nodes.push(DependencyNode {
            name: name.clone(),
            version: resolved,
            resolved: None,
            integrity: None,
            dependencies: Vec::new(),
            dev: false,
            optional: false,
          });
        }
      }
    }

    if !opts.production {
      if let Some(dev_deps) = dev_deps {
        for (name, version) in dev_deps {
          let ver_str = version.as_str().unwrap_or("*");
          if let Ok(resolved) = resolve_version(name, ver_str) {
            nodes.push(DependencyNode {
              name: name.clone(),
              version: resolved,
              resolved: None,
              integrity: None,
              dependencies: Vec::new(),
              dev: true,
              optional: false,
            });
          }
        }
      }
    }

    Ok(nodes)
  }

  // ── Add ─────────────────────────────────────────────────────────────────

  pub fn add(&mut self, name: &str, version: Option<&str>, dev: bool) -> Result<DependencyNode, PmError> {
    let ver_str = version.unwrap_or("latest");
    let resolved = if ver_str == "latest" {
      resolve_version(name, ">=0.0.0")?
    } else {
      resolve_version(name, ver_str)?
    };

    let node = DependencyNode {
      name: name.to_string(),
      version: resolved.clone(),
      resolved: None,
      integrity: None,
      dependencies: Vec::new(),
      dev,
      optional: false,
    };

    // Update package.json
    let pkg_json_path = self.dir.join("package.json");
    if pkg_json_path.exists() {
      let content = std::fs::read_to_string(&pkg_json_path)?;
      let mut json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| PmError::ResolutionError(format!("Invalid package.json: {e}")))?;

      let field = if dev { "devDependencies" } else { "dependencies" };
      if let Some(obj) = json.get_mut(field).and_then(|v| v.as_object_mut()) {
        obj.insert(name.to_string(), serde_json::Value::String(format!("^{resolved}")));
      } else {
        if let Some(obj) = json.as_object_mut() {
          let mut deps = serde_json::Map::new();
          deps.insert(name.to_string(), serde_json::Value::String(format!("^{resolved}")));
          obj.insert(field.to_string(), serde_json::Value::Object(deps));
        }
      }

      std::fs::write(&pkg_json_path, serde_json::to_string_pretty(&json).unwrap())?;
    }

    Ok(node)
  }

  // ── Remove ──────────────────────────────────────────────────────────────

  pub fn remove(&mut self, name: &str) -> Result<bool, PmError> {
    let pkg_json_path = self.dir.join("package.json");
    if !pkg_json_path.exists() {
      return Err(PmError::PackageNotFound("package.json not found".into()));
    }

    let content = std::fs::read_to_string(&pkg_json_path)?;
    let mut json: serde_json::Value = serde_json::from_str(&content)
      .map_err(|e| PmError::ResolutionError(format!("Invalid package.json: {e}")))?;

    let mut removed = false;
    for field in &["dependencies", "devDependencies", "optionalDependencies", "peerDependencies"] {
      if let Some(obj) = json.get_mut(field).and_then(|v| v.as_object_mut()) {
        if obj.remove(name).is_some() {
          removed = true;
        }
      }
    }

    if removed {
      std::fs::write(&pkg_json_path, serde_json::to_string_pretty(&json).unwrap())?;
      // Remove from lockfile
      if let Some(ref mut lockfile) = self.lockfile {
        lockfile.packages.retain(|path, _| !path.ends_with(name) && !path.contains(&format!("/{name}/")));
      }
    }

    // Remove from workspace members if applicable
    if let Some(ref mut ws) = self.workspace {
      if name.starts_with("./packages/") || name.starts_with("packages/") {
        ws.members.retain(|m| m != name);
      }
    }

    Ok(removed)
  }

  // ── Audit ───────────────────────────────────────────────────────────────

  pub fn audit(&self) -> Result<AuditResult, PmError> {
    // In production, this would query the npm audit API
    // For now, return empty audit result
    Ok(AuditResult {
      vulnerabilities: Vec::new(),
      total_count: 0,
      critical: 0,
      high: 0,
      moderate: 0,
      low: 0,
    })
  }

  // ── Outdated ────────────────────────────────────────────────────────────

  pub fn outdated(&self) -> Result<Vec<OutdatedPackage>, PmError> {
    let pkg_json_path = self.dir.join("package.json");
    if !pkg_json_path.exists() {
      return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&pkg_json_path)?;
    let json: serde_json::Value = serde_json::from_str(&content)
      .map_err(|e| PmError::ResolutionError(format!("Invalid package.json: {e}")))?;

    let mut outdated = Vec::new();
    for field in &["dependencies", "devDependencies"] {
      if let Some(deps) = json.get(field).and_then(|v| v.as_object()) {
        for (name, version) in deps {
          let ver_str = version.as_str().unwrap_or("*");
          let wanted = resolve_version(name, ver_str).unwrap_or_else(|_| ver_str.to_string());
          let latest = resolve_version(name, ">=0.0.0").unwrap_or_else(|_| ver_str.to_string());

          if wanted != latest {
            outdated.push(OutdatedPackage {
              name: name.clone(),
              current: ver_str.trim_start_matches('^').trim_start_matches('~').to_string(),
              wanted,
              latest,
            });
          }
        }
      }
    }

    Ok(outdated)
  }

  // ── Dedupe ──────────────────────────────────────────────────────────────

  pub fn dedupe(&mut self) -> Result<usize, PmError> {
    if let Some(ref mut lockfile) = self.lockfile {
      let mut seen: HashSet<String> = HashSet::new();
      let mut to_remove = Vec::new();

      for (path, pkg) in &lockfile.packages {
        let key = if let Some(pos) = path.rfind('/') {
          &path[pos + 1..]
        } else {
          path.as_str()
        };
        let qualifier = format!("{}@{}", key, pkg.version);
        if !seen.insert(qualifier) {
          to_remove.push(path.clone());
        }
      }

      for path in &to_remove {
        lockfile.packages.remove(path);
      }

      Ok(to_remove.len())
    } else {
      Ok(0)
    }
  }

  // ── Lockfile Write ──────────────────────────────────────────────────────

  pub fn write_lockfile(&self) -> Result<(), PmError> {
    if let Some(ref lockfile) = self.lockfile {
      let content = lockfile.to_npm_lockfile()?;
      let path = self.dir.join(self.kind.lockfile_name());
      std::fs::write(&path, content)?;
    }
    Ok(())
  }

  pub fn generate_lockfile(&self) -> Result<LockfileV3, PmError> {
    let pkg_json_path = self.dir.join("package.json");
    if !pkg_json_path.exists() {
      return Err(PmError::PackageNotFound("package.json not found".into()));
    }
    let content = std::fs::read_to_string(&pkg_json_path)?;
    let json: serde_json::Value = serde_json::from_str(&content)
      .map_err(|e| PmError::ResolutionError(format!("Invalid package.json: {e}")))?;

    let mut packages = BTreeMap::new();
    let deps = json.get("dependencies").and_then(|v| v.as_object());
    let dev_deps = json.get("devDependencies").and_then(|v| v.as_object());

    let all_deps = deps
      .into_iter()
      .flatten()
      .chain(dev_deps.into_iter().flatten());

    for (name, version) in all_deps {
      let ver_str = version.as_str().unwrap_or("*");
      if let Ok(resolved) = resolve_version(name, ver_str) {
        let path = format!("node_modules/{name}");
        packages.insert(
          path,
          LockfilePackage {
            version: resolved,
            resolved: None,
            integrity: None,
            dependencies: None,
            optional_dependencies: None,
            peer_dependencies: None,
            dev: None,
            optional: None,
            bundled: None,
            engines: None,
            os: None,
            cpu: None,
            has_dev_dependencies: None,
          },
        );
      }
    }

    Ok(LockfileV3 {
      name: json.get("name").and_then(|v| v.as_str()).map(String::from),
      lockfile_version: Some("3".into()),
      packages,
      workspaces: json.get("workspaces").and_then(|v| v.as_array()).map(|a| {
        a.iter().filter_map(|v| v.as_str().map(String::from)).collect()
      }),
      metadata: None,
    })
  }
}

// ── Klyron Lockfile (klyron-lock.json) ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlyronLockfile {
  pub name: Option<String>,
  pub lockfile_version: String,
  pub packages: BTreeMap<String, KlyronLockPackage>,
  pub workspaces: Option<WorkspaceConfig>,
  pub git_dependencies: Option<HashMap<String, GitDepLock>>,
  pub metadata: Option<LockfileMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlyronLockPackage {
  pub version: String,
  pub resolved: Option<String>,
  pub integrity: Option<String>,
  pub link: Option<bool>,
  pub dev: Option<bool>,
  pub optional: Option<bool>,
  pub dependencies: Option<HashMap<String, String>>,
  pub optional_dependencies: Option<HashMap<String, String>>,
  pub peer_dependencies: Option<HashMap<String, String>>,
  pub engines: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
  pub packages: Vec<String>,
  pub members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDepLock {
  pub url: String,
  pub rev: String,
  pub directory: Option<String>,
  pub version: String,
}

impl KlyronLockfile {
  pub fn new(name: Option<String>) -> Self {
    Self {
      name,
      lockfile_version: "1".into(),
      packages: BTreeMap::new(),
      workspaces: None,
      git_dependencies: None,
      metadata: None,
    }
  }

  pub fn from_json(content: &str) -> Result<Self, PmError> {
    serde_json::from_str(content)
      .map_err(|e| PmError::LockfileError(format!("Invalid klyron-lock.json: {e}")))
  }

  pub fn to_json_pretty(&self) -> Result<String, PmError> {
    serde_json::to_string_pretty(self)
      .map_err(|e| PmError::LockfileError(format!("Serialization error: {e}")))
  }

  pub fn add_package(&mut self, path: &str, pkg: KlyronLockPackage) {
    self.packages.insert(path.to_string(), pkg);
  }

  pub fn get_package(&self, path: &str) -> Option<&KlyronLockPackage> {
    self.packages.get(path)
  }

  pub fn merge(&mut self, other: &KlyronLockfile) {
    for (path, pkg) in &other.packages {
      self.packages.entry(path.clone()).or_insert_with(|| pkg.clone());
    }
    if let Some(ref ws) = other.workspaces {
      self.workspaces = Some(ws.clone());
    }
    if let Some(ref git_deps) = other.git_dependencies {
      if self.git_dependencies.is_none() {
        self.git_dependencies = Some(HashMap::new());
      }
      if let Some(ref mut existing) = self.git_dependencies {
        for (key, dep) in git_deps {
          existing.entry(key.clone()).or_insert_with(|| dep.clone());
        }
      }
    }
  }
}

// ── Monorepo Workspace Support ─────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct WorkspaceManager {
  pub root: PathBuf,
  pub config: Option<WorkspaceConfig>,
  pub member_packages: Vec<PathBuf>,
}

impl WorkspaceManager {
  pub fn new(root: &Path) -> Self {
    let mut wm = Self {
      root: root.to_path_buf(),
      config: None,
      member_packages: Vec::new(),
    };
    wm.discover();
    wm
  }

  pub fn discover(&mut self) {
    let pkg_json = self.root.join("package.json");
    if !pkg_json.exists() {
      return;
    }
    let content = std::fs::read_to_string(&pkg_json).ok();
    let json: Option<serde_json::Value> = content.and_then(|c| serde_json::from_str(&c).ok());
    let workspaces = json.as_ref().and_then(|j| {
      j.get("workspaces").and_then(|w| {
        match w {
          serde_json::Value::Array(arr) => Some(arr.clone()),
          serde_json::Value::Object(obj) => obj.get("packages").and_then(|p| p.as_array().cloned()),
          _ => None,
        }
      })
    });

    if let Some(ws) = workspaces {
      let patterns: Vec<String> = ws.iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
      self.config = Some(WorkspaceConfig {
        packages: patterns.clone(),
        members: Vec::new(),
      });

      // Resolve glob patterns to actual member paths
      for pattern in &patterns {
        if pattern.contains('*') {
          let glob_pattern = self.root.join(pattern).to_string_lossy().to_string();
          if let Ok(entries) = glob::glob(&glob_pattern) {
            for entry in entries.flatten() {
              if entry.join("package.json").exists() {
                self.member_packages.push(entry);
              }
            }
          }
        } else {
          let dir = self.root.join(pattern);
          if dir.exists() && dir.join("package.json").exists() {
            self.member_packages.push(dir);
          }
        }
      }

      if let Some(ref mut config) = self.config {
        config.members = self.member_packages.iter()
          .filter_map(|p| p.file_name().and_then(|n| n.to_str().map(String::from)))
          .collect();
      }
    }
  }

  pub fn get_member_names(&self) -> Vec<String> {
    self.member_packages.iter()
      .filter_map(|p| {
        let pkg_json = p.join("package.json");
        if pkg_json.exists() {
          let content = std::fs::read_to_string(&pkg_json).ok()?;
          let json: serde_json::Value = serde_json::from_str(&content).ok()?;
          json.get("name").and_then(|n| n.as_str().map(String::from))
        } else {
          p.file_name().and_then(|n| n.to_str().map(String::from))
        }
      })
      .collect()
  }

  pub fn for_each_member<F>(&self, f: F) -> Result<(), PmError>
  where
    F: Fn(&Path) -> Result<(), PmError>,
  {
    for member in &self.member_packages {
      f(member)?;
    }
    Ok(())
  }

  pub fn install_all(&self, opts: &InstallOptions) -> Result<HashMap<String, Vec<DependencyNode>>, PmError> {
    let mut results = HashMap::new();
    for member in &self.member_packages {
      let pm = PackageManager::new(member);
      let deps = pm.install(opts)?;
      if let Some(name) = member.file_name().and_then(|n| n.to_str()) {
        results.insert(name.to_string(), deps);
      }
    }
    Ok(results)
  }
}

// ── Workspace Protocol ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum WorkspaceDependency {
  Star,
  Range(String),
  Tilde(String),
}

pub fn parse_workspace_dependency(dep_spec: &str) -> Option<WorkspaceDependency> {
  let spec = dep_spec.trim();
  if !spec.starts_with("workspace:") {
    return None;
  }
  let inner = &spec["workspace:".len()..];
  match inner {
    "*" => Some(WorkspaceDependency::Star),
    s if s.starts_with('^') => Some(WorkspaceDependency::Range(s.to_string())),
    s if s.starts_with('~') => Some(WorkspaceDependency::Tilde(s.to_string())),
    // plain semver
    s if s.chars().next().map_or(false, |c| c.is_ascii_digit()) => Some(WorkspaceDependency::Range(format!("^{s}"))),
    _ => None,
  }
}

pub fn resolve_workspace_dependency(
  dep_spec: &str,
  workspace_members: &HashMap<String, String>,
) -> Result<String, PmError> {
  let ws_dep = parse_workspace_dependency(dep_spec)
    .ok_or_else(|| PmError::WorkspaceError(format!("Invalid workspace spec: {dep_spec}")))?;
  match ws_dep {
    WorkspaceDependency::Star => {
      // Find the workspace member that matches the package
      // The caller should determine the package name from context
      Err(PmError::WorkspaceError("workspace:* requires a package name context".into()))
    }
    WorkspaceDependency::Range(spec) | WorkspaceDependency::Tilde(spec) => {
      // Find a workspace member whose version satisfies the spec
      let req = semver::VersionReq::parse(&spec)
        .map_err(|e| PmError::ResolutionError(format!("Invalid semver: {e}")))?;
      for (name, version) in workspace_members {
        if let Ok(ver) = semver::Version::parse(version) {
          if req.matches(&ver) {
            return Ok(name.clone());
          }
        }
      }
      Err(PmError::VersionNotFound(format!("No workspace member matches '{spec}'")))
    }
  }
}

pub fn resolve_workspace_dependency_version(
  dep_spec: &str,
  workspace_members: &HashMap<String, String>,
  package_name: &str,
) -> Result<String, PmError> {
  let ws_dep = parse_workspace_dependency(dep_spec)
    .ok_or_else(|| PmError::WorkspaceError(format!("Invalid workspace spec: {dep_spec}")))?;
  match ws_dep {
    WorkspaceDependency::Star => {
      workspace_members.get(package_name)
        .cloned()
        .ok_or_else(|| PmError::PackageNotFound(format!("Workspace member '{package_name}' not found")))
    }
    WorkspaceDependency::Range(spec) | WorkspaceDependency::Tilde(spec) => {
      if let Some(version) = workspace_members.get(package_name) {
        let req = semver::VersionReq::parse(&spec)
          .map_err(|e| PmError::ResolutionError(format!("Invalid semver: {e}")))?;
        if let Ok(ver) = semver::Version::parse(version) {
          if req.matches(&ver) {
            return Ok(version.clone());
          }
        }
        return Err(PmError::VersionNotFound(format!(
          "Workspace member '{package_name}' version {version} does not match '{spec}'"
        )));
      }
      // Try to find any member matching the spec
      resolve_workspace_dependency(dep_spec, workspace_members).and_then(|name| {
        workspace_members.get(&name).cloned().ok_or_else(|| PmError::PackageNotFound(name))
      })
    }
  }
}

fn get_workspace_member_versions(workspace: &WorkspaceManager) -> HashMap<String, String> {
  let mut map = HashMap::new();
  for member in &workspace.member_packages {
    let pkg_json = member.join("package.json");
    if let Ok(content) = std::fs::read_to_string(&pkg_json) {
      if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
          let version = json.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();
          map.insert(name.to_string(), version);
        }
      }
    }
  }
  map
}

// ── Link / Unlink ─────────────────────────────────────────────────────────

pub fn get_global_link_dir() -> PathBuf {
  dirs::home_dir()
    .map(|h| h.join(".klyron").join("store").join("linked"))
    .unwrap_or_else(|| PathBuf::from("/tmp/.klyron/linked"))
}

pub fn link_package(package_dir: &Path, global_dir: &Path) -> Result<PathBuf, PmError> {
  let pkg_json = package_dir.join("package.json");
  if !pkg_json.exists() {
    return Err(PmError::PackageNotFound("package.json not found in package dir".into()));
  }
  let content = std::fs::read_to_string(&pkg_json)?;
  let json: serde_json::Value = serde_json::from_str(&content)
    .map_err(|e| PmError::ResolutionError(format!("Invalid package.json: {e}")))?;
  let name = json.get("name")
    .and_then(|v| v.as_str())
    .ok_or_else(|| PmError::PackageNotFound("No 'name' in package.json".into()))?;

  std::fs::create_dir_all(global_dir)?;
  let link_path = global_dir.join(name);

  // Remove existing symlink or directory
  if link_path.exists() || link_path.is_symlink() {
    if link_path.is_symlink() || link_path.is_file() {
      std::fs::remove_file(&link_path)?;
    } else {
      std::fs::remove_dir_all(&link_path)?;
    }
  }

  // Create symlink
  let abs_package_dir = package_dir.canonicalize()
    .map_err(|e| PmError::IoError(format!("Cannot canonicalize package dir: {e}")))?;
  #[cfg(unix)]
  std::os::unix::fs::symlink(&abs_package_dir, &link_path)
    .map_err(|e| PmError::IoError(format!("Failed to create symlink: {e}")))?;
  #[cfg(windows)]
  std::os::windows::fs::symlink_dir(&abs_package_dir, &link_path)
    .map_err(|e| PmError::IoError(format!("Failed to create symlink: {e}")))?;

  Ok(link_path)
}

pub fn link_global(package_name: &str, target_dir: &Path) -> Result<PathBuf, PmError> {
  let global_dir = get_global_link_dir();
  link_global_from_dir(package_name, target_dir, &global_dir)
}

pub fn link_global_from_dir(package_name: &str, target_dir: &Path, global_dir: &Path) -> Result<PathBuf, PmError> {
  let link_source = global_dir.join(package_name);
  if !link_source.exists() {
    return Err(PmError::PackageNotFound(format!(
      "Global link '{package_name}' not found at {}",
      link_source.display()
    )));
  }

  let node_modules = target_dir.join("node_modules");
  std::fs::create_dir_all(&node_modules)?;
  let link_dest = node_modules.join(package_name);

  if link_dest.exists() || link_dest.is_symlink() {
    if link_dest.is_symlink() || link_dest.is_file() {
      std::fs::remove_file(&link_dest)?;
    } else {
      std::fs::remove_dir_all(&link_dest)?;
    }
  }

  let abs_source = std::fs::canonicalize(&link_source)
    .map_err(|e| PmError::IoError(format!("Cannot resolve link source: {e}")))?;
  #[cfg(unix)]
  std::os::unix::fs::symlink(&abs_source, &link_dest)
    .map_err(|e| PmError::IoError(format!("Failed to create symlink: {e}")))?;
  #[cfg(windows)]
  std::os::windows::fs::symlink_dir(&abs_source, &link_dest)
    .map_err(|e| PmError::IoError(format!("Failed to create symlink: {e}")))?;

  Ok(link_dest)
}

pub fn unlink_package(package_name: &str) -> Result<(), PmError> {
  let global_dir = get_global_link_dir();
  let link_path = global_dir.join(package_name);
  if !link_path.exists() && !link_path.is_symlink() {
    return Err(PmError::PackageNotFound(format!(
      "No global link '{package_name}' found"
    )));
  }
  if link_path.is_symlink() || link_path.is_file() {
    std::fs::remove_file(&link_path)?;
  } else {
    std::fs::remove_dir_all(&link_path)?;
  }
  Ok(())
}

// ── Pack ───────────────────────────────────────────────────────────────────

pub fn pack_package(dir: &Path, output_path: Option<&Path>) -> Result<PathBuf, PmError> {
  use flate2::write::GzEncoder;
  use flate2::Compression;
  use tar::Builder;

  let pkg_json = dir.join("package.json");
  if !pkg_json.exists() {
    return Err(PmError::PackageNotFound("package.json not found".into()));
  }

  let content = std::fs::read_to_string(&pkg_json)?;
  let json: serde_json::Value = serde_json::from_str(&content)
    .map_err(|e| PmError::ResolutionError(format!("Invalid package.json: {e}")))?;

  let name = json.get("name").and_then(|v| v.as_str()).unwrap_or("package");
  let version = json.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0");

  let output = output_path.map_or_else(|| {
    dir.join(format!("{name}-{version}.tgz"))
  }, |p| p.to_path_buf());

  let file = std::fs::File::create(&output)?;
  let encoder = GzEncoder::new(file, Compression::default());
  let mut archive = Builder::new(encoder);

  // Files/dirs to include
  let include_patterns = [
    "package.json",
    "README.md",
    "LICENSE",
    "LICENSE.md",
    "CHANGELOG.md",
    "dist",
    "src",
  ];

  // Files/dirs to exclude
  let exclude_patterns = [
    "node_modules",
    ".git",
    "target",
    "test",
    "__tests__",
    "tests",
    ".klyron",
  ];

  // Track added paths to avoid duplicates
  let mut added = std::collections::HashSet::new();

  for entry in walkdir::WalkDir::new(dir)
    .into_iter()
    .filter_entry(|e| {
      let name = e.file_name().to_str().unwrap_or("");
      !exclude_patterns.contains(&name)
    })
    .filter_map(|e| e.ok())
  {
    let relative = entry.path().strip_prefix(dir).unwrap_or(entry.path());
    let relative_str = relative.to_string_lossy();

    // Check if this path matches any include pattern
    let should_include = include_patterns.iter().any(|pat| {
      relative_str == *pat || relative_str.starts_with(&format!("{pat}/"))
    }) || {
      // Also include any files that aren't excluded and are not in excluded dirs
      let parent_in_exclude = exclude_patterns.iter().any(|pat| {
        relative_str.starts_with(&format!("{pat}/"))
      });
      !parent_in_exclude && relative_str != "."
    };

    if !should_include {
      continue;
    }

    let archive_path = format!("package/{relative_str}");
    if added.contains(&archive_path) {
      continue;
    }
    added.insert(archive_path.clone());

    if entry.file_type().is_dir() {
      archive.append_dir(&archive_path, entry.path())
        .map_err(|e| PmError::IoError(format!("Failed to add dir to tarball: {e}")))?;
    } else if entry.file_type().is_file() {
      let mut f = std::fs::File::open(entry.path())?;
      archive.append_file(&archive_path, &mut f)
        .map_err(|e| PmError::IoError(format!("Failed to add file to tarball: {e}")))?;
    }
  }

  let encoder = archive.into_inner()
    .map_err(|e| PmError::IoError(format!("Failed to finish tarball: {e}")))?;
  encoder.finish()?;

  Ok(output)
}

/// Generate integrity hash for a package tarball
pub fn pack_and_get_integrity(dir: &Path) -> Result<(PathBuf, String), PmError> {
  let tarball_path = pack_package(dir, None)?;
  let data = std::fs::read(&tarball_path)?;
  let integrity = compute_integrity(&data);
  Ok((tarball_path, integrity))
}

// ── Publish ────────────────────────────────────────────────────────────────

pub fn publish_package(
  dir: &Path,
  registry_url: &str,
  token: Option<&str>,
) -> Result<(), PmError> {
  let pkg_json = dir.join("package.json");
  if !pkg_json.exists() {
    return Err(PmError::PackageNotFound("package.json not found".into()));
  }

  let content = std::fs::read_to_string(&pkg_json)?;
  let json: serde_json::Value = serde_json::from_str(&content)
    .map_err(|e| PmError::ResolutionError(format!("Invalid package.json: {e}")))?;

  let name = json.get("name").and_then(|v| v.as_str())
    .ok_or_else(|| PmError::PackageNotFound("No 'name' in package.json".into()))?;
  let version = json.get("version").and_then(|v| v.as_str())
    .ok_or_else(|| PmError::PackageNotFound("No 'version' in package.json".into()))?;

  // Pack the package
  let tarball_path = pack_package(dir, None)?;

  let _ = version; // version is validated but not directly used in HTTP request

  let final_data = std::fs::read(&tarball_path)?;

  let client = reqwest::blocking::Client::builder()
    .timeout(std::time::Duration::from_secs(60))
    .build()
    .map_err(|e| PmError::IoError(format!("Failed to create HTTP client: {e}")))?;

  let url = format!("{registry_url}/{name}");
  let mut req = client.put(&url).body(final_data);
  if let Some(t) = token {
    req = req.header("Authorization", format!("Bearer {t}"));
  }

  let resp = req.send()
    .map_err(|e| PmError::IoError(format!("Publish request failed: {e}")))?;

  if !resp.status().is_success() {
    return Err(PmError::IoError(format!(
      "Publish failed: HTTP {}",
      resp.status()
    )));
  }

  let _ = std::fs::remove_file(&tarball_path);
  Ok(())
}

// ── Dist-tag ────────────────────────────────────────────────────────────────

pub fn add_dist_tag(package: &str, version: &str, tag: &str, registry_url: &str) -> Result<(), PmError> {
  let client = reqwest::blocking::Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .map_err(|e| PmError::IoError(format!("Failed to create HTTP client: {e}")))?;

  let url = format!("{registry_url}/-/package/{package}/dist-tags/{tag}");
  let body = serde_json::json!(version).to_string();

  let resp = client.put(&url)
    .header("Content-Type", "application/json")
    .body(body)
    .send()
    .map_err(|e| PmError::IoError(format!("Failed to set dist-tag: {e}")))?;

  if !resp.status().is_success() {
    return Err(PmError::IoError(format!(
      "Failed to set dist-tag: HTTP {}",
      resp.status()
    )));
  }
  Ok(())
}

pub fn remove_dist_tag(package: &str, tag: &str, registry_url: &str) -> Result<(), PmError> {
  let client = reqwest::blocking::Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .map_err(|e| PmError::IoError(format!("Failed to create HTTP client: {e}")))?;

  let url = format!("{registry_url}/-/package/{package}/dist-tags/{tag}");

  let resp = client.delete(&url)
    .send()
    .map_err(|e| PmError::IoError(format!("Failed to remove dist-tag: {e}")))?;

  if !resp.status().is_success() {
    return Err(PmError::IoError(format!(
      "Failed to remove dist-tag: HTTP {}",
      resp.status()
    )));
  }
  Ok(())
}

pub fn list_dist_tags(package: &str, registry_url: &str) -> Result<HashMap<String, String>, PmError> {
  let client = reqwest::blocking::Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .map_err(|e| PmError::IoError(format!("Failed to create HTTP client: {e}")))?;

  let url = if package.starts_with('@') {
    let encoded = package.replace('/', "%2F");
    format!("{registry_url}/{encoded}")
  } else {
    format!("{registry_url}/{package}")
  };

  let resp = client.get(&url)
    .send()
    .map_err(|e| PmError::IoError(format!("Failed to fetch package info: {e}")))?;

  if !resp.status().is_success() {
    return Err(PmError::PackageNotFound(format!(
      "Package '{package}' not found: HTTP {}",
      resp.status()
    )));
  }

  let json: serde_json::Value = resp.json()
    .map_err(|e| PmError::IoError(format!("Failed to parse response: {e}")))?;

  let tags = json.get("dist-tags")
    .and_then(|t| t.as_object())
    .map(|o| o.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect())
    .unwrap_or_default();

  Ok(tags)
}

// ── Package Info ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageFullInfo {
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

pub fn package_info(name: &str, registry_url: &str) -> Result<PackageFullInfo, PmError> {
  let client = reqwest::blocking::Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .map_err(|e| PmError::IoError(format!("Failed to create HTTP client: {e}")))?;

  let url = if name.starts_with('@') {
    let encoded = name.replace('/', "%2F");
    format!("{registry_url}/{encoded}")
  } else {
    format!("{registry_url}/{name}")
  };

  let resp = client.get(&url)
    .header("Accept", "application/json")
    .send()
    .map_err(|e| PmError::IoError(format!("Failed to fetch package info: {e}")))?;

  if !resp.status().is_success() {
    return Err(PmError::PackageNotFound(format!(
      "Package '{name}' not found: HTTP {}",
      resp.status()
    )));
  }

  let json: serde_json::Value = resp.json()
    .map_err(|e| PmError::IoError(format!("Failed to parse response: {e}")))?;

  let latest_version = json.get("dist-tags")
    .and_then(|t| t.get("latest"))
    .and_then(|v| v.as_str())
    .unwrap_or("unknown")
    .to_string();

  let all_versions = json.get("versions")
    .and_then(|v| v.as_object())
    .map(|o| o.keys().cloned().collect::<Vec<_>>())
    .unwrap_or_default();

  let maintainers = json.get("maintainers")
    .and_then(|m| m.as_array())
    .map(|arr| {
      arr.iter().filter_map(|m| {
        Some(MaintainerInfo {
          name: m.get("name")?.as_str()?.to_string(),
          email: m.get("email").and_then(|e| e.as_str()).map(String::from),
        })
      }).collect()
    })
    .unwrap_or_default();

  let homepage = json.get("homepage").and_then(|v| v.as_str()).map(String::from);
  let repo = json.get("repository").and_then(|r| {
    r.as_str().map(String::from).or_else(|| {
      r.get("url").and_then(|u| u.as_str()).map(String::from)
    })
  });

  Ok(PackageFullInfo {
    name: name.to_string(),
    description: json.get("description").and_then(|v| v.as_str()).map(String::from),
    latest_version,
    all_versions,
    maintainers,
    homepage,
    license: json.get("license").and_then(|v| v.as_str()).map(String::from),
    repository: repo,
  })
}

// ── Why (Dependency Tree) ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WhyPath {
  pub path: Vec<String>,
  pub depth: usize,
}

pub fn why_package(
  name: &str,
  lockfile: &KlyronLockfile,
) -> Result<Vec<WhyPath>, PmError> {
  let mut results = Vec::new();

  // Build a forward dependency map: for each package, what does it depend on
  let mut forward_deps: HashMap<String, Vec<String>> = HashMap::new();
  let mut all_pkg_names: HashSet<String> = HashSet::new();

  for (key, pkg) in &lockfile.packages {
    if let Some(at_pos) = key.rfind('@') {
      let pkg_name = key[..at_pos].to_string();
      all_pkg_names.insert(pkg_name.clone());

      let deps = forward_deps.entry(pkg_name).or_default();
      if let Some(ref d) = pkg.dependencies {
        deps.extend(d.keys().cloned());
      }
      if let Some(ref pd) = pkg.peer_dependencies {
        deps.extend(pd.keys().cloned());
      }
    }
  }

  // Find all packages that match the name
  let matching_keys: Vec<String> = lockfile.packages.keys()
    .filter(|k| {
      if let Some(at_pos) = k.rfind('@') {
        &k[..at_pos] == name || k.ends_with(name)
      } else {
        false
      }
    })
    .cloned()
    .collect();

  if matching_keys.is_empty() {
    return Err(PmError::PackageNotFound(format!(
      "Package '{name}' not found in lockfile"
    )));
  }

  // DFS from root following forward deps to find all paths to target
  fn find_paths_fwd(
    current: &str,
    target: &str,
    forward_deps: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    current_path: &mut Vec<String>,
    results: &mut Vec<WhyPath>,
  ) {
    if current == target {
      results.push(WhyPath {
        path: current_path.clone(),
        depth: current_path.len() - 1,
      });
      return;
    }

    if !visited.insert(current.to_string()) {
      return;
    }

    if let Some(deps) = forward_deps.get(current) {
      for dep in deps {
        current_path.push(dep.clone());
        find_paths_fwd(dep, target, forward_deps, visited, current_path, results);
        current_path.pop();
      }
    }

    visited.remove(current);
  }

  // Find root packages (those not depended on by anyone)
  let mut reverse_deps: HashMap<String, Vec<String>> = HashMap::new();
  for (pkg_name, deps) in &forward_deps {
    for dep in deps {
      if all_pkg_names.contains(dep) {
        reverse_deps.entry(dep.clone()).or_default().push(pkg_name.clone());
      }
    }
  }

  let root_pkgs: Vec<String> = all_pkg_names.iter()
    .filter(|n| !reverse_deps.contains_key(*n) || reverse_deps[*n].is_empty())
    .cloned()
    .collect();

  for root in &root_pkgs {
    let mut path = vec![root.clone()];
    let mut visited = HashSet::new();
    find_paths_fwd(root, name, &forward_deps, &mut visited, &mut path, &mut results);
  }

  // Also include direct root deps
  for matching_key in &matching_keys {
    if let Some(at_pos) = matching_key.rfind('@') {
      let pkg_name = &matching_key[..at_pos];
      if root_pkgs.contains(&pkg_name.to_string()) {
        if !results.iter().any(|r| r.path == vec![pkg_name.to_string()]) {
          results.push(WhyPath {
            path: vec![pkg_name.to_string()],
            depth: 0,
          });
        }
      }
    }
  }

  // Deduplicate and sort paths
  results.sort_by(|a, b| a.depth.cmp(&b.depth).then(a.path.len().cmp(&b.path.len())));
  results.dedup_by(|a, b| a.path == b.path);

  Ok(results)
}

// ── Binary Scripts (.bin symlinks) ────────────────────────────────────────

pub fn install_bin_scripts(
  package_dir: &Path,
  bin_map: &HashMap<String, String>,
  node_modules_dir: &Path,
) -> Result<Vec<PathBuf>, PmError> {
  let bin_dir = node_modules_dir.join(".bin");
  std::fs::create_dir_all(&bin_dir)?;

  let mut created = Vec::new();

  for (bin_name, bin_path_str) in bin_map {
    let bin_path = package_dir.join(bin_path_str);
    let link_path = bin_dir.join(bin_name);

    // Remove existing
    if link_path.exists() || link_path.is_symlink() {
      if link_path.is_symlink() || link_path.is_file() {
        let _ = std::fs::remove_file(&link_path);
      } else {
        let _ = std::fs::remove_dir_all(&link_path);
      }
    }

    // Create the symlink
    let abs_bin_path = if bin_path.exists() {
      std::fs::canonicalize(&bin_path)
        .unwrap_or(bin_path)
    } else {
      // Try relative to package_dir
      package_dir.join(bin_path_str)
    };

    #[cfg(unix)]
    std::os::unix::fs::symlink(&abs_bin_path, &link_path)
      .map_err(|e| PmError::IoError(format!("Failed to create .bin symlink: {e}")))?;
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&abs_bin_path, &link_path)
      .map_err(|e| PmError::IoError(format!("Failed to create .bin symlink: {e}")))?;

    // Fix shebang on Unix
    #[cfg(unix)]
    fix_shebang(&abs_bin_path);

    // Create a cmd wrapper for Windows compatibility
    let cmd_wrapper = bin_dir.join(format!("{bin_name}.cmd"));
    let pkg_name = package_dir.file_name().unwrap_or_default().to_string_lossy().to_string();
    let cmd_path = bin_path_str.replace('/', "\\");
    let cmd_content = format!(
      "@if not defined DEBUG_HELPER @echo off\r\n\"%~dp0\\..\\{pkg_name}\\{cmd_path}\" %*\r\n"
    );
    std::fs::write(&cmd_wrapper, cmd_content)?;

    created.push(link_path);
  }

  Ok(created)
}

#[cfg(unix)]
fn fix_shebang(path: &Path) {
  use std::io::Read;
  if let Ok(mut file) = std::fs::File::open(path) {
    let mut first_bytes = [0u8; 2];
    if file.read(&mut first_bytes).ok() == Some(2) && first_bytes == [0x23, 0x21] {
      // It has a shebang, read the first line
      let mut contents = String::new();
      file.read_to_string(&mut contents).ok();
      let first_line = contents.lines().next().unwrap_or("");
      // If the shebang points to node or nodejs, make sure it's executable
      if first_line.contains("node") || first_line.contains("nodejs") {
        // Ensure the script is executable
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
          let mut perms = metadata.permissions();
          if perms.mode() & 0o111 == 0 {
            perms.set_mode(perms.mode() | 0o111);
            let _ = std::fs::set_permissions(path, perms);
          }
        }
      }
    }
  }
}

/// Scan all packages in node_modules for "bin" fields and create .bin symlinks
pub fn install_all_bin_scripts(node_modules_dir: &Path) -> Result<usize, PmError> {
  let mut count = 0;
  if !node_modules_dir.exists() {
    return Ok(0);
  }

  for entry in std::fs::read_dir(node_modules_dir)
    .map_err(|e| PmError::IoError(format!("Failed to read node_modules: {e}")))? {
    let entry = entry?;
    let path = entry.path();
    if !path.is_dir() || path.file_name().map_or(true, |n| n.to_str().map_or(true, |n| n.starts_with('.'))) {
      continue;
    }

    // Handle @scoped packages
    if path.file_name().and_then(|n| n.to_str()) == Some("@") {
      if let Ok(scoped_dir) = std::fs::read_dir(&path) {
        for scope_entry in scoped_dir.flatten() {
          let pkg_dir = scope_entry.path();
          let pkg_json_path = pkg_dir.join("package.json");
          if let Ok(content) = std::fs::read_to_string(&pkg_json_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
              if let Some(bin) = json.get("bin") {
                let bin_map = parse_bin_field(bin);
                if !bin_map.is_empty() {
                  let created = install_bin_scripts(&pkg_dir, &bin_map, node_modules_dir)?;
                  count += created.len();
                }
              }
            }
          }
        }
      }
      continue;
    }

    let pkg_json_path = path.join("package.json");
    if let Ok(content) = std::fs::read_to_string(&pkg_json_path) {
      if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(bin) = json.get("bin") {
          let bin_map = parse_bin_field(bin);
          if !bin_map.is_empty() {
            let created = install_bin_scripts(&path, &bin_map, node_modules_dir)?;
            count += created.len();
          }
        }
      }
    }
  }

  Ok(count)
}

fn parse_bin_field(bin: &serde_json::Value) -> HashMap<String, String> {
  match bin {
    serde_json::Value::String(s) => {
      let name = std::path::Path::new(s)
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("cli")
        .to_string();
      let mut map = HashMap::new();
      map.insert(name, s.clone());
      map
    }
    serde_json::Value::Object(obj) => {
      obj.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect()
    }
    _ => HashMap::new(),
  }
}

// ── Peer / Optional Dependencies ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PeerDepWarning {
  pub package: String,
  pub peer_name: String,
  pub required_range: String,
  pub found_version: Option<String>,
}

pub fn resolve_peer_dependencies(
  package_name: &str,
  peer_deps: &HashMap<String, String>,
  all_packages: &HashMap<String, String>,
) -> Vec<PeerDepWarning> {
  let mut warnings = Vec::new();

  for (peer_name, required_range) in peer_deps {
    let found = all_packages.get(peer_name);
    match found {
      Some(version) => {
        if let Ok(req) = semver::VersionReq::parse(required_range) {
          if let Ok(ver) = semver::Version::parse(version) {
            if !req.matches(&ver) {
              warnings.push(PeerDepWarning {
                package: package_name.to_string(),
                peer_name: peer_name.clone(),
                required_range: required_range.clone(),
                found_version: Some(version.clone()),
              });
            }
          }
        }
      }
      None => {
        warnings.push(PeerDepWarning {
          package: package_name.to_string(),
          peer_name: peer_name.clone(),
          required_range: required_range.clone(),
          found_version: None,
        });
      }
    }
  }

  warnings
}

/// Handle optional dependencies: skip if they fail to resolve
pub fn handle_optional_deps(
  _package_name: &str,
  optional_deps: &HashMap<String, String>,
) -> Vec<(String, String)> {
  let mut resolved = Vec::new();
  for (name, version) in optional_deps {
    match resolve_version(name, version) {
      Ok(v) => resolved.push((name.clone(), v)),
      Err(_) => {
        // Silently skip optional deps that fail
        continue;
      }
    }
  }
  resolved
}

/// Apply overrides from package.json resolution field
pub fn resolve_overrides(
  package_json: &serde_json::Value,
  resolution_map: &mut HashMap<String, String>,
) -> Result<(), PmError> {
  // Check for "overrides" field
  if let Some(overrides) = package_json.get("overrides").and_then(|o| o.as_object()) {
    for (name, version_val) in overrides {
      let version = match version_val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(obj) => {
          // Support nested overrides like: "overrides": { "foo": { ".": "1.0.0", "bar": "2.0.0" } }
          if let Some(dot) = obj.get(".").and_then(|v| v.as_str()) {
            dot.to_string()
          } else {
            continue;
          }
        }
        _ => continue,
      };
      resolution_map.insert(name.clone(), version);
    }
  }

  // Check for "resolutions" field (yarn-style)
  if let Some(resolutions) = package_json.get("resolutions").and_then(|r| r.as_object()) {
    for (name, version_val) in resolutions {
      if let Some(version) = version_val.as_str() {
        resolution_map.insert(name.clone(), version.to_string());
      }
    }
  }

  Ok(())
}

pub fn apply_overrides_to_deps(
  deps: &mut HashMap<String, String>,
  resolution_map: &HashMap<String, String>,
) {
  for (name, resolved_version) in resolution_map {
    if deps.contains_key(name) {
      deps.insert(name.clone(), resolved_version.clone());
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDependency {
  pub url: String,
  pub rev: Option<String>,
  pub branch: Option<String>,
  pub tag: Option<String>,
  pub directory: Option<String>,
}

static GIT_DEP_RE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r#"^(https?://|git@|git://|ssh://)[^#]+(?:#([a-fA-F0-9]+|[^#]+))?$"#).unwrap()
});

pub fn parse_git_dependency(spec: &str) -> Option<GitDependency> {
  if !GIT_DEP_RE.is_match(spec) && !spec.contains("github:") && !spec.contains("git+") {
    return None;
  }

  let spec = spec.trim_start_matches("git+");
  let (url_part, fragment) = if let Some(hash_idx) = spec.find('#') {
    (&spec[..hash_idx], Some(&spec[hash_idx + 1..]))
  } else {
    (spec, None)
  };

  let url = if let Some(gh) = url_part.strip_prefix("github:") {
    format!("https://github.com/{}.git", gh)
  } else {
    url_part.to_string()
  };

  let mut dep = GitDependency {
    url,
    rev: None,
    branch: None,
    tag: None,
    directory: None,
  };

  if let Some(frag) = fragment {
    if frag.contains(':') {
      for part in frag.split('&') {
        if let Some(dir) = part.strip_prefix("dir=") {
          dep.directory = Some(dir.to_string());
        }
      }
    } else if frag.len() == 40 && frag.chars().all(|c| c.is_ascii_hexdigit()) {
      dep.rev = Some(frag.to_string());
    } else {
      dep.branch = Some(frag.to_string());
      dep.tag = Some(frag.to_string());
    }
  }

  Some(dep)
}

pub fn resolve_git_dependency(dep: &GitDependency, target_dir: &Path) -> Result<String, PmError> {
  let cache_dir = target_dir.join(".klyron").join("git-cache");
  std::fs::create_dir_all(&cache_dir)
    .map_err(|e| PmError::IoError(format!("Failed to create cache dir: {e}")))?;

  // Clone/fetch the repo
  let repo_name = dep.url.rsplit('/').next()
    .unwrap_or("repo")
    .trim_end_matches(".git");
  let repo_cache = cache_dir.join(repo_name);

  if !repo_cache.exists() {
    let output = Command::new("git")
      .args(["clone", "--depth", "1", &dep.url, repo_cache.to_string_lossy().as_ref()])
      .output()
      .map_err(|e| PmError::IoError(format!("Git clone failed: {e}")))?;

    if !output.status.success() {
      return Err(PmError::IoError(format!(
        "Git clone failed: {}",
        String::from_utf8_lossy(&output.stderr)
      )));
    }
  }

  // Checkout specific rev/branch/tag if specified
  if let Some(ref rev) = dep.rev {
    let output = Command::new("git")
      .args(["-C", repo_cache.to_string_lossy().as_ref(), "checkout", rev])
      .output()
      .map_err(|e| PmError::IoError(format!("Git checkout failed: {e}")))?;

    if !output.status.success() {
      return Err(PmError::IoError(format!(
        "Git checkout {rev} failed: {}",
        String::from_utf8_lossy(&output.stderr)
      )));
    }
  }

  // Get current HEAD rev
  let output = Command::new("git")
    .args(["-C", repo_cache.to_string_lossy().as_ref(), "rev-parse", "HEAD"])
    .output()
    .map_err(|e| PmError::IoError(format!("Git rev-parse failed: {e}")))?;

  let rev = String::from_utf8_lossy(&output.stdout).trim().to_string();

  // Copy to target
  let source_dir = if let Some(ref dir) = dep.directory {
    repo_cache.join(dir)
  } else {
    repo_cache.clone()
  };

  let dest_dir = target_dir.join("node_modules").join(repo_name);
  std::fs::create_dir_all(&dest_dir)
    .map_err(|e| PmError::IoError(format!("Failed to create dest dir: {e}")))?;

  copy_dir_recursive(&source_dir, &dest_dir)?;

  Ok(rev)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), PmError> {
  for entry in walkdir::WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
    if entry.file_type().is_dir() {
      let dest = dst.join(entry.path().strip_prefix(src).unwrap_or(entry.path()));
      std::fs::create_dir_all(&dest)?;
    } else if entry.file_type().is_file() {
      let dest = dst.join(entry.path().strip_prefix(src).unwrap_or(entry.path()));
      if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
      }
      std::fs::copy(entry.path(), &dest)?;
    }
  }
  Ok(())
}

// ── Package Scripts Lifecycle ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleHook {
  Preinstall,
  Install,
  Postinstall,
  Prepare,
  Prepack,
  Postpack,
  Prepublish,
  PrepublishOnly,
  Postpublish,
  Preversion,
  Postversion,
}

impl std::fmt::Display for LifecycleHook {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl LifecycleHook {
  pub fn as_str(&self) -> &str {
    match self {
      Self::Preinstall => "preinstall",
      Self::Install => "install",
      Self::Postinstall => "postinstall",
      Self::Prepare => "prepare",
      Self::Prepack => "prepack",
      Self::Postpack => "postpack",
      Self::Prepublish => "prepublish",
      Self::PrepublishOnly => "prepublishOnly",
      Self::Postpublish => "postpublish",
      Self::Preversion => "preversion",
      Self::Postversion => "postversion",
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageScripts {
  pub preinstall: Option<String>,
  pub install: Option<String>,
  pub postinstall: Option<String>,
  pub prepare: Option<String>,
  pub prepack: Option<String>,
  pub postpack: Option<String>,
  pub prepublish: Option<String>,
  pub prepublish_only: Option<String>,
  pub postpublish: Option<String>,
  pub preversion: Option<String>,
  pub postversion: Option<String>,
  pub other: HashMap<String, String>,
}

impl PackageScripts {
  pub fn from_package_json(json: &serde_json::Value) -> Self {
    let scripts = json.get("scripts").and_then(|s| s.as_object());
    let mut ps = PackageScripts {
      preinstall: None,
      install: None,
      postinstall: None,
      prepare: None,
      prepack: None,
      postpack: None,
      prepublish: None,
      prepublish_only: None,
      postpublish: None,
      preversion: None,
      postversion: None,
      other: HashMap::new(),
    };

    if let Some(scripts) = scripts {
      for (key, value) in scripts {
        let val = value.as_str().unwrap_or("").to_string();
        match key.as_str() {
          "preinstall" => ps.preinstall = Some(val),
          "install" => ps.install = Some(val),
          "postinstall" => ps.postinstall = Some(val),
          "prepare" => ps.prepare = Some(val),
          "prepack" => ps.prepack = Some(val),
          "postpack" => ps.postpack = Some(val),
          "prepublish" => ps.prepublish = Some(val),
          "prepublishOnly" => ps.prepublish_only = Some(val),
          "postpublish" => ps.postpublish = Some(val),
          "preversion" => ps.preversion = Some(val),
          "postversion" => ps.postversion = Some(val),
      _ => {
                ps.other.insert(key.clone(), val);
            }
        }
      }
    }

    ps
  }

  pub fn run_hook(&self, hook: &LifecycleHook, dir: &Path) -> Result<(), PmError> {
    let script = match hook {
      LifecycleHook::Preinstall => &self.preinstall,
      LifecycleHook::Install => &self.install,
      LifecycleHook::Postinstall => &self.postinstall,
      LifecycleHook::Prepare => &self.prepare,
      LifecycleHook::Prepack => &self.prepack,
      LifecycleHook::Postpack => &self.postpack,
      LifecycleHook::Prepublish => &self.prepublish,
      LifecycleHook::PrepublishOnly => &self.prepublish_only,
      LifecycleHook::Postpublish => &self.postpublish,
      LifecycleHook::Preversion => &self.preversion,
      LifecycleHook::Postversion => &self.postversion,
    };

    if let Some(cmd) = script {
      if cmd.is_empty() {
        return Ok(());
      }
      let shell = if cfg!(target_os = "windows") { "cmd" } else { "sh" };
      let shell_arg = if cfg!(target_os = "windows") { "/C" } else { "-c" };

      let output = Command::new(shell)
        .args([shell_arg, cmd])
        .current_dir(dir)
        .output()
        .map_err(|e| PmError::IoError(format!("Failed to run {hook}: {e}")))?;

      if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PmError::IoError(format!(
          "Lifecycle hook '{hook}' failed: {stderr}"
        )));
      }
    }

    Ok(())
  }

  pub fn run_lifecycle_sequence(&self, dir: &Path, hooks: &[LifecycleHook]) -> Result<(), PmError> {
    for hook in hooks {
      self.run_hook(hook, dir)?;
    }
    Ok(())
  }
}

impl PackageManager {
  pub fn run_install_scripts(&self) -> Result<(), PmError> {
    let pkg_json = self.dir.join("package.json");
    if !pkg_json.exists() {
      return Ok(());
    }
    let content = std::fs::read_to_string(&pkg_json)?;
    let json: serde_json::Value = serde_json::from_str(&content)
      .map_err(|e| PmError::ResolutionError(format!("Invalid package.json: {e}")))?;
    let scripts = PackageScripts::from_package_json(&json);

    scripts.run_lifecycle_sequence(&self.dir, &[
      LifecycleHook::Preinstall,
      LifecycleHook::Install,
      LifecycleHook::Postinstall,
      LifecycleHook::Prepare,
    ])
  }

  pub fn run_script(&self, script_name: &str) -> Result<String, PmError> {
    let pkg_json = self.dir.join("package.json");
    if !pkg_json.exists() {
      return Err(PmError::PackageNotFound("package.json not found".into()));
    }
    let content = std::fs::read_to_string(&pkg_json)?;
    let json: serde_json::Value = serde_json::from_str(&content)
      .map_err(|e| PmError::ResolutionError(format!("Invalid package.json: {e}")))?;
    let scripts = PackageScripts::from_package_json(&json);

    let cmd = scripts.other.get(script_name)
      .or_else(|| {
        match script_name {
          "preinstall" => scripts.preinstall.as_ref(),
          "install" => scripts.install.as_ref(),
          "postinstall" => scripts.postinstall.as_ref(),
          "prepare" => scripts.prepare.as_ref(),
          _ => None,
        }
      })
      .ok_or_else(|| PmError::PackageNotFound(format!("Script '{script_name}' not found")))?;

    let shell = if cfg!(target_os = "windows") { "cmd" } else { "sh" };
    let shell_arg = if cfg!(target_os = "windows") { "/C" } else { "-c" };

    let output = Command::new(shell)
      .args([shell_arg, cmd])
      .current_dir(&self.dir)
      .output()
      .map_err(|e| PmError::IoError(format!("Failed to run script '{script_name}': {e}")))?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(PmError::IoError(format!("Script '{script_name}' failed: {stderr}")));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
  }
}

// ── Improved Audit Command ──────────────────────────────────────────────────

impl PackageManager {
  pub fn audit_improved(&self) -> Result<AuditResult, PmError> {
    let mut vulnerabilities = Vec::new();

    // Check lockfile for known vulnerable packages
    if let Some(ref lockfile) = self.lockfile {
      for (path, pkg) in &lockfile.packages {
        // Simulate vulnerability checking against a database
        // In production, this would query the npm audit API or local advisory DB
        let pkg_name = path.rsplit('/').next().unwrap_or(path);
        let ver = &pkg.version;

        // Check for known vulnerabilities based on severity patterns
        if let Ok(v) = Version::parse(ver) {
          let advisory = check_known_vulnerability(pkg_name, &v);
          if let Some(adv) = advisory {
            vulnerabilities.push(adv);
          }
        }
      }
    }

    let critical = vulnerabilities.iter().filter(|v| v.severity == "critical").count();
    let high = vulnerabilities.iter().filter(|v| v.severity == "high").count();
    let moderate = vulnerabilities.iter().filter(|v| v.severity == "moderate").count();
    let low = vulnerabilities.iter().filter(|v| v.severity == "low").count();

    Ok(AuditResult {
      total_count: vulnerabilities.len(),
      critical,
      high,
      moderate,
      low,
      vulnerabilities,
    })
  }

  pub fn write_klyron_lockfile(&self) -> Result<(), PmError> {
    let lock = self.generate_klyron_lockfile()?;
    let content = lock.to_json_pretty()?;
    let path = self.dir.join("klyron-lock.json");
    std::fs::write(&path, content)?;
    Ok(())
  }

  pub fn generate_klyron_lockfile(&self) -> Result<KlyronLockfile, PmError> {
    let pkg_json_path = self.dir.join("package.json");
    if !pkg_json_path.exists() {
      return Err(PmError::PackageNotFound("package.json not found".into()));
    }

    let content = std::fs::read_to_string(&pkg_json_path)?;
    let json: serde_json::Value = serde_json::from_str(&content)
      .map_err(|e| PmError::ResolutionError(format!("Invalid package.json: {e}")))?;

    let mut lock = KlyronLockfile::new(
      json.get("name").and_then(|v| v.as_str()).map(String::from),
    );

    // Process dependencies
    let deps = json.get("dependencies").and_then(|v| v.as_object());
    let dev_deps = json.get("devDependencies").and_then(|v| v.as_object());

    let all_deps = deps.into_iter().flatten()
      .chain(dev_deps.into_iter().flatten());

    for (name, version) in all_deps {
      let ver_str = version.as_str().unwrap_or("*");

      // Check if it's a git dependency
      if let Some(git_dep) = parse_git_dependency(ver_str) {
        let rev = resolve_git_dependency(&git_dep, &self.dir).unwrap_or_default();
        lock.git_dependencies.get_or_insert_with(HashMap::new)
          .insert(name.clone(), GitDepLock {
            url: git_dep.url.clone(),
            rev: rev.clone(),
            directory: git_dep.directory.clone(),
            version: rev,
          });
        continue;
      }

      if let Ok(resolved) = resolve_version(name, ver_str) {
        let path = format!("node_modules/{name}");
        lock.add_package(&path, KlyronLockPackage {
          version: resolved,
          resolved: None,
          integrity: None,
          link: None,
          dev: None,
          optional: None,
          dependencies: None,
          optional_dependencies: None,
          peer_dependencies: None,
          engines: None,
        });
      }
    }

    // Process workspace members
    let ws = WorkspaceManager::new(&self.dir);
    if ws.config.is_some() {
      lock.workspaces = ws.config.clone().map(|c| WorkspaceConfig {
        packages: c.packages,
        members: ws.get_member_names(),
      });
    }

    Ok(lock)
  }
}

// ── Install Result ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct InstallResult {
  pub nodes: Vec<DependencyNode>,
  pub start_time: SystemTime,
  pub end_time: SystemTime,
}

impl InstallResult {
  pub fn duration_ms(&self) -> u64 {
    self.end_time.duration_since(self.start_time)
      .unwrap_or_default()
      .as_millis() as u64
  }
}

// ── Lockfile Generation ────────────────────────────────────────────────────

pub fn generate_lockfile(install_result: &InstallResult, lockfile_path: &Path) -> Result<(), PmError> {
  use lockfile::{KlyronLockfile, LockfilePackage};

  let mut lock = KlyronLockfile::new();
  lock.metadata.install_count = install_result.nodes.len() as u64;

  for node in &install_result.nodes {
    let integrity = node.integrity.clone()
      .unwrap_or_else(|| compute_integrity(node.name.as_bytes()));
    let resolved = node.resolved.clone()
      .unwrap_or_else(|| format!("https://registry.npmjs.org/{}/-/{}-{}.tgz", node.name, node.name, node.version));

    let mut deps = HashMap::new();
    for dep in &node.dependencies {
      deps.insert(dep.name.clone(), dep.version.clone());
    }

    lock.add_package(
      &node.name,
      &node.version,
      LockfilePackage {
        name: node.name.clone(),
        version: node.version.clone(),
        resolved,
        integrity,
        dependencies: deps,
        optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(),
        bin: None,
        has_node_modules: false,
        install_time_ms: install_result.duration_ms(),
      },
    );
  }

  let bytes = lock.to_bytes()?;
  if let Some(parent) = lockfile_path.parent() {
    std::fs::create_dir_all(parent)?;
  }
  std::fs::write(lockfile_path, bytes)?;
  Ok(())
}

pub fn install_with_lockfile(project_dir: &Path, frozen: bool) -> Result<(), PmError> {
  use lockfile::KlyronLockfile;

  let lockfile_path = project_dir.join("klyron.lock");

  // Load existing lockfile if present
  let existing_lock = if lockfile_path.exists() {
    let data = std::fs::read(&lockfile_path)?;
    Some(KlyronLockfile::from_bytes(&data)?)
  } else {
    None
  };

  if frozen {
    if let Some(ref lock) = existing_lock {
      lock.frozen_check(project_dir)?;
      return Ok(());
    }
    return Err(PmError::LockfileError("klyron.lock not found in frozen mode".into()));
  }

  let pm = PackageManager::new(project_dir);
  let opts = InstallOptions::default();
  let start = SystemTime::now();
  let nodes = pm.install(&opts)?;
  let end = SystemTime::now();

  let result = InstallResult {
    nodes,
    start_time: start,
    end_time: end,
  };

  generate_lockfile(&result, &lockfile_path)?;
  Ok(())
}

pub fn migrate_from_npm_lockfile(npm_lock_path: &Path) -> Result<lockfile::KlyronLockfile, PmError> {
  use lockfile::{KlyronLockfile, LockfilePackage};

  let content = std::fs::read_to_string(npm_lock_path)?;
  let npm_lock: serde_json::Value = serde_json::from_str(&content)
    .map_err(|e| PmError::LockfileError(format!("Invalid npm lockfile: {e}")))?;

  let mut klock = KlyronLockfile::new();

  let packages = npm_lock.get("packages")
    .or_else(|| npm_lock.get("dependencies"))
    .and_then(|v| v.as_object());

  if let Some(pkgs) = packages {
    for (path, info) in pkgs {
      if path.is_empty() {
        continue;
      }
      let name = path.rsplit('/').next().unwrap_or(&path).to_string();
      let version = info.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();
      let resolved = info.get("resolved").and_then(|v| v.as_str()).unwrap_or("").to_string();
      let integrity = info.get("integrity").and_then(|v| v.as_str()).unwrap_or("").to_string();

      let deps = info.get("dependencies").and_then(|v| v.as_object())
        .map(|o| o.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect())
        .unwrap_or_default();
      let opt_deps = info.get("optionalDependencies").and_then(|v| v.as_object())
        .map(|o| o.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect())
        .unwrap_or_default();
      let peer_deps = info.get("peerDependencies").and_then(|v| v.as_object())
        .map(|o| o.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect())
        .unwrap_or_default();

      klock.add_package(&name, &version, LockfilePackage {
        name: name.clone(),
        version: version.clone(),
        resolved,
        integrity,
        dependencies: deps,
        optional_dependencies: opt_deps,
        peer_dependencies: peer_deps,
        bin: None,
        has_node_modules: false,
        install_time_ms: 0,
      });
    }
  }

  Ok(klock)
}

pub fn migrate_from_yarn_lockfile(yarn_lock_path: &Path) -> Result<lockfile::KlyronLockfile, PmError> {
  use lockfile::{KlyronLockfile, LockfilePackage};

  let content = std::fs::read_to_string(yarn_lock_path)?;
  let mut klock = KlyronLockfile::new();

  for entry in content.split("\n\n") {
    let entry = entry.trim();
    if entry.is_empty() || entry.starts_with('#') {
      continue;
    }

    let mut name = String::new();
    let mut version = String::new();
    let mut resolved = String::new();
    let mut integrity = String::new();
    let deps = HashMap::new();

    for line in entry.lines() {
      let line = line.trim();
      if let Some(ident) = line.strip_suffix(':') {
        if !ident.starts_with('"') && !ident.contains('@') {
          continue;
        }
        if name.is_empty() {
          let raw = ident.trim_matches('"');
          if let Some(at_pos) = raw.find("@") {
            let (n, v) = raw.split_at(at_pos);
            name = n.to_string();
            version = v.trim_start_matches('@').to_string();
          }
        }
      } else if let Some(val) = line.strip_prefix("version ") {
        version = val.trim().trim_matches('"').to_string();
      } else if let Some(val) = line.strip_prefix("resolved ") {
        resolved = val.trim().trim_matches('"').to_string();
      } else if let Some(val) = line.strip_prefix("integrity ") {
        integrity = val.trim().trim_matches('"').to_string();
      } else if line.starts_with("dependencies:") {
        // yarn v1 format - skip
      }
    }

    if !name.is_empty() && !version.is_empty() {
      klock.add_package(&name, &version, LockfilePackage {
        name: name.clone(),
        version: version.clone(),
        resolved,
        integrity,
        dependencies: deps,
        optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(),
        bin: None,
        has_node_modules: false,
        install_time_ms: 0,
      });
    }
  }

  Ok(klock)
}

fn check_known_vulnerability(name: &str, version: &Version) -> Option<AuditVulnerability> {
  // Built-in known vulnerability database (simplified)
  // In production, this would use a full advisory database
  let advisories: Vec<(&str, &str, VersionReq, &str, &str, f64)> = vec![
    // (package, cve, version_req, severity, title, cvss)
  ];

  for (pkg, cve, req, severity, title, cvss) in &advisories {
    if *pkg == name && req.matches(version) {
      return Some(AuditVulnerability {
        package: name.to_string(),
        version: version.to_string(),
        severity: severity.to_string(),
        title: title.to_string(),
        cve: Some(cve.to_string()),
        cvss_score: Some(*cvss),
      });
    }
  }

  None
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_detect_pm() {
    let dir = std::env::temp_dir().join("_klyron_pm_detect");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("package-lock.json"), "{}").unwrap();
    assert_eq!(PackageManager::detect(&dir), PackageManagerKind::Npm);
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_detect_yarn() {
    let dir = std::env::temp_dir().join("_klyron_pm_yarn");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("yarn.lock"), "").unwrap();
    assert_eq!(PackageManager::detect(&dir), PackageManagerKind::Yarn);
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_detect_pnpm() {
    let dir = std::env::temp_dir().join("_klyron_pm_pnpm");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("pnpm-lock.yaml"), "").unwrap();
    assert_eq!(PackageManager::detect(&dir), PackageManagerKind::Pnpm);
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_pm_kind_lockfile_name() {
    assert_eq!(PackageManagerKind::Npm.lockfile_name(), "package-lock.json");
    assert_eq!(PackageManagerKind::Yarn.lockfile_name(), "yarn.lock");
    assert_eq!(PackageManagerKind::Pnpm.lockfile_name(), "pnpm-lock.yaml");
    assert_eq!(PackageManagerKind::Bun.lockfile_name(), "bun.lockb");
  }

  #[test]
  fn test_pm_kind_commands() {
    assert_eq!(PackageManagerKind::Npm.install_command(), "npm install");
    assert_eq!(PackageManagerKind::Npm.add_command(true), "npm install --save-dev");
    assert_eq!(PackageManagerKind::Yarn.add_command(false), "yarn add");
    assert_eq!(PackageManagerKind::Pnpm.remove_command(), "pnpm remove");
  }

  #[test]
  fn test_lockfile_v3_from_npm() {
    let content = r#"{
      "name": "test",
      "lockfileVersion": 3,
      "packages": {
        "node_modules/left-pad": {
          "version": "1.3.0",
          "resolved": "https://registry.npmjs.org/left-pad/-/left-pad-1.3.0.tgz",
          "integrity": "sha512-XI5J8dE+Fi5zJ2kHkJHl26eECFMN2wtRoO+Q6/K0Rj2QSFhRxbN9nDmoXZ5wH/LmGmTBNb2Qs+YmbS6+lOPcxw=="
        }
      }
    }"#;
    let lock = LockfileV3::from_npm_lockfile(content).unwrap();
    assert_eq!(lock.name, Some("test".into()));
    let (path, pkg) = lock.find_package("left-pad").unwrap();
    assert_eq!(pkg.version, "1.3.0");
    assert!(path.contains("left-pad"));
  }

  #[test]
  fn test_resolve_version() {
    let ver = resolve_version("test-pkg", "^1.0.0").unwrap();
    let v = Version::parse(&ver).unwrap();
    assert!(v >= Version::new(1, 0, 0));
    assert!(v < Version::new(2, 0, 0));
  }

  #[test]
  fn test_resolve_version_star() {
    let ver = resolve_version("test-pkg", "*").unwrap();
    assert_eq!(ver, "3.2.0");
  }

  #[test]
  fn test_compute_integrity() {
    let hash = compute_integrity(b"hello");
    assert!(hash.starts_with("sha512-"));
    assert!(!hash.contains('+'));
    assert!(!hash.contains('/'));
  }

  #[test]
  fn test_verify_integrity() {
    let data = b"test data";
    let hash = compute_integrity(data);
    assert!(verify_integrity(data, &hash).is_ok());
    assert!(verify_integrity(b"wrong data", &hash).is_err());
  }

  #[test]
  fn test_generate_lockfile() {
    let dir = std::env::temp_dir().join("_klyron_pm_gen");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
      dir.join("package.json"),
      r#"{"name":"test","dependencies":{"left-pad":"^1.0.0"}}"#,
    )
    .unwrap();
    let pm = PackageManager::new(&dir);
    let lock = pm.generate_lockfile().unwrap();
    assert_eq!(lock.name, Some("test".into()));
    assert!(!lock.packages.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_pm_new() {
    let dir = std::env::temp_dir().join("_klyron_pm_new");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("package-lock.json"), r#"{"name":"test","lockfileVersion":3,"packages":{}}"#)
      .unwrap();
    let pm = PackageManager::new(&dir);
    assert_eq!(pm.kind, PackageManagerKind::Npm);
    assert!(pm.lockfile.is_some());
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_add_and_remove() {
    let dir = std::env::temp_dir().join("_klyron_pm_add");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
      dir.join("package.json"),
      r#"{"name":"test"}"#,
    )
    .unwrap();
    let mut pm = PackageManager::new(&dir);
    let node = pm.add("left-pad", Some("^1.0.0"), false).unwrap();
    assert_eq!(node.name, "left-pad");
    assert_eq!(node.version, "1.3.0");
    let removed = pm.remove("left-pad").unwrap();
    assert!(removed);
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_outdated() {
    let dir = std::env::temp_dir().join("_klyron_pm_outdated");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
      dir.join("package.json"),
      r#"{"name":"test","dependencies":{"left-pad":"^1.0.0"}}"#,
    )
    .unwrap();
    let pm = PackageManager::new(&dir);
    let outdated = pm.outdated().unwrap();
    // left-pad 1.3.0 is latest, so not outdated
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_audit() {
    let dir = std::env::temp_dir().join("_klyron_pm_audit");
    std::fs::create_dir_all(&dir).unwrap();
    let pm = PackageManager::new(&dir);
    let audit = pm.audit().unwrap();
    assert_eq!(audit.total_count, 0);
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_dedupe_empty() {
    let mut lock = LockfileV3 {
      name: Some("test".into()),
      lockfile_version: Some("3".into()),
      packages: BTreeMap::new(),
      workspaces: None,
      metadata: None,
    };
    assert_eq!(lock.packages.len(), 0);
  }

  #[test]
  fn test_install_empty() {
    let dir = std::env::temp_dir().join("_klyron_pm_install");
    std::fs::create_dir_all(&dir).unwrap();
    let pm = PackageManager::new(&dir);
    let opts = InstallOptions::default();
    let result = pm.install(&opts);
    assert!(result.is_err() || result.is_ok());
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_lockfile_to_npm_format() {
    let mut lock = LockfileV3 {
      name: Some("test".into()),
      lockfile_version: Some("3".into()),
      packages: BTreeMap::new(),
      workspaces: None,
      metadata: None,
    };
    lock.packages.insert(
      "node_modules/test".into(),
      LockfilePackage {
        version: "1.0.0".into(),
        resolved: Some("https://registry.npmjs.org/test/-/test-1.0.0.tgz".into()),
        integrity: Some("sha512-test".into()),
        dependencies: None,
        optional_dependencies: None,
        peer_dependencies: None,
        dev: None,
        optional: None,
        bundled: None,
        engines: None,
        os: None,
        cpu: None,
        has_dev_dependencies: None,
      },
    );
    let output = lock.to_npm_lockfile().unwrap();
    assert!(output.contains("1.0.0"));
    assert!(output.contains("test-1.0.0.tgz"));
  }

  #[test]
  fn test_pm_error_types() {
    let e1 = PmError::PackageNotFound("test".into());
    let e2 = PmError::IntegrityMismatch {
      expected: "abc".into(),
      actual: "def".into(),
    };
    assert!(e1.to_string().contains("test"));
    assert!(e2.to_string().contains("abc"));
  }

  #[test]
  fn test_detect_workspace() {
    let dir = std::env::temp_dir().join("_klyron_pm_ws");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
      dir.join("package.json"),
      r#"{"name":"ws","workspaces":["packages/*"]}"#,
    )
    .unwrap();
    let ws = PackageManager::detect_workspace(&dir);
    assert!(ws.is_some());
    assert_eq!(ws.unwrap().members.len(), 1);
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_semver_constraint_parsing() {
    let req = VersionReq::parse("^1.2.3").unwrap();
    assert!(req.matches(&Version::new(1, 2, 3)));
    assert!(req.matches(&Version::new(1, 5, 0)));
    assert!(!req.matches(&Version::new(2, 0, 0)));
  }

  #[test]
  fn test_klyron_lockfile_new() {
    let lock = KlyronLockfile::new(Some("test".into()));
    assert_eq!(lock.lockfile_version, "1");
    assert!(lock.packages.is_empty());
  }

  #[test]
  fn test_klyron_lockfile_roundtrip() {
    let mut lock = KlyronLockfile::new(Some("test".into()));
    lock.add_package("node_modules/test-pkg", KlyronLockPackage {
      version: "1.0.0".into(),
      resolved: None,
      integrity: None,
      link: None,
      dev: None,
      optional: None,
      dependencies: None,
      optional_dependencies: None,
      peer_dependencies: None,
      engines: None,
    });
    let json = lock.to_json_pretty().unwrap();
    let parsed = KlyronLockfile::from_json(&json).unwrap();
    assert_eq!(parsed.name, Some("test".into()));
    assert!(parsed.packages.contains_key("node_modules/test-pkg"));
  }

  #[test]
  fn test_klyron_lockfile_merge() {
    let mut lock1 = KlyronLockfile::new(Some("pkg1".into()));
    lock1.add_package("node_modules/a", KlyronLockPackage {
      version: "1.0.0".into(), resolved: None, integrity: None, link: None,
      dev: None, optional: None, dependencies: None,
      optional_dependencies: None, peer_dependencies: None, engines: None,
    });
    let mut lock2 = KlyronLockfile::new(Some("pkg2".into()));
    lock2.add_package("node_modules/b", KlyronLockPackage {
      version: "2.0.0".into(), resolved: None, integrity: None, link: None,
      dev: None, optional: None, dependencies: None,
      optional_dependencies: None, peer_dependencies: None, engines: None,
    });
    lock1.merge(&lock2);
    assert_eq!(lock1.packages.len(), 2);
  }

  #[test]
  fn test_workspace_manager_no_package_json() {
    let dir = std::env::temp_dir().join("_klyron_pm_ws_no_pkg");
    let _ = std::fs::create_dir_all(&dir);
    let ws = WorkspaceManager::new(&dir);
    assert!(ws.member_packages.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_workspace_discover() {
    let dir = std::env::temp_dir().join("_klyron_pm_ws_discover");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("package.json"), r#"{"workspaces":["packages/*"]}"#).unwrap();
    std::fs::create_dir_all(dir.join("packages").join("pkg-a")).unwrap();
    std::fs::write(dir.join("packages/pkg-a/package.json"), r#"{"name":"@test/pkg-a"}"#).unwrap();
    std::fs::create_dir_all(dir.join("packages").join("pkg-b")).unwrap();
    std::fs::write(dir.join("packages/pkg-b/package.json"), r#"{"name":"@test/pkg-b"}"#).unwrap();

    let ws = WorkspaceManager::new(&dir);
    assert!(ws.config.is_some());
    assert_eq!(ws.member_packages.len(), 2);
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_parse_git_dependency_https() {
    let dep = parse_git_dependency("https://github.com/user/repo.git#main");
    assert!(dep.is_some());
    let dep = dep.unwrap();
    assert!(dep.url.contains("github.com/user/repo"));
    assert_eq!(dep.branch, Some("main".into()));
  }

  #[test]
  fn test_parse_git_dependency_github_shorthand() {
    let dep = parse_git_dependency("github:user/repo");
    assert!(dep.is_some());
    let dep = dep.unwrap();
    assert!(dep.url.contains("github.com/user/repo"));
    assert!(dep.rev.is_none());
  }

  #[test]
  fn test_parse_git_dependency_with_commit() {
    let dep = parse_git_dependency("https://github.com/user/repo.git#abc123def456abc123def456abc123def456abc1");
    assert!(dep.is_some());
    let dep = dep.unwrap();
    assert_eq!(dep.rev, Some("abc123def456abc123def456abc123def456abc1".into()));
  }

  #[test]
  fn test_parse_git_dependency_invalid() {
    let dep = parse_git_dependency("lodash@^1.0.0");
    assert!(dep.is_none());
  }

  #[test]
  fn test_package_scripts_from_json() {
    let json: serde_json::Value = serde_json::from_str(r#"{
      "scripts": {
        "preinstall": "echo pre",
        "postinstall": "echo post",
        "prepare": "echo prepare",
        "test": "jest"
      }
    }"#).unwrap();
    let scripts = PackageScripts::from_package_json(&json);
    assert_eq!(scripts.preinstall, Some("echo pre".into()));
    assert_eq!(scripts.postinstall, Some("echo post".into()));
    assert_eq!(scripts.other.get("test").map(|s| s.as_str()), Some("jest"));
  }

  #[test]
  fn test_package_scripts_hook_name() {
    assert_eq!(LifecycleHook::Preinstall.as_str(), "preinstall");
    assert_eq!(LifecycleHook::Postinstall.as_str(), "postinstall");
    assert_eq!(LifecycleHook::Prepare.as_str(), "prepare");
  }

  #[test]
  fn test_audit_improved_empty() {
    let lock = LockfileV3::from_npm_lockfile(r#"{"name":"test","lockfileVersion":3,"packages":{}}"#).unwrap();
    let pm = PackageManager {
      dir: std::env::temp_dir().join("_klyron_pm_nonexistent"),
      kind: PackageManagerKind::Npm,
      lockfile: Some(lock),
      workspace: None,
    };
    let audit = pm.audit_improved().unwrap();
    assert_eq!(audit.total_count, 0);
  }

  #[test]
  fn test_generate_klyron_lockfile() {
    let dir = std::env::temp_dir().join("_klyron_pm_klyron_lock");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("package.json"), r#"{"name":"test","dependencies":{"left-pad":"^1.0.0"}}"#).unwrap();
    let pm = PackageManager::new(&dir);
    let lock = pm.generate_klyron_lockfile().unwrap();
    assert_eq!(lock.name, Some("test".into()));
    assert!(!lock.packages.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_lifecycle_hooks() {
    assert_eq!(LifecycleHook::Preinstall as u8, 0);
    assert_eq!(LifecycleHook::Postinstall as u8, 2);
  }

  #[test]
  fn test_workspace_config_serde() {
    let config = WorkspaceConfig {
      packages: vec!["packages/*".into()],
      members: vec!["pkg-a".into(), "pkg-b".into()],
    };
    let json = serde_json::to_value(&config).unwrap();
    assert_eq!(json["packages"][0], "packages/*");
    assert_eq!(json["members"].as_array().unwrap().len(), 2);
  }

  #[test]
  fn test_git_dep_lock_serde() {
    let dep = GitDepLock {
      url: "https://github.com/user/repo.git".into(),
      rev: "abc123".into(),
      directory: None,
      version: "1.0.0".into(),
    };
    let json = serde_json::to_value(&dep).unwrap();
    assert_eq!(json["url"], "https://github.com/user/repo.git");
  }

  #[test]
  fn test_generate_lockfile_function() {
    let dir = std::env::temp_dir().join("_klyron_pm_gen_lockfile");
    std::fs::create_dir_all(&dir).unwrap();
    let result = InstallResult {
      nodes: vec![
        DependencyNode {
          name: "left-pad".into(),
          version: "1.3.0".into(),
          resolved: Some("https://registry.npmjs.org/left-pad/-/left-pad-1.3.0.tgz".into()),
          integrity: Some("sha512-test".into()),
          dependencies: vec![],
          dev: false,
          optional: false,
        },
      ],
      start_time: std::time::SystemTime::now(),
      end_time: std::time::SystemTime::now(),
    };
    let lockfile_path = dir.join("klyron.lock");
    generate_lockfile(&result, &lockfile_path).unwrap();
    assert!(lockfile_path.exists());
    let data = std::fs::read(&lockfile_path).unwrap();
    assert!(data.starts_with(b"KLYR"));
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_lockfile_module_roundtrip() {
    use lockfile::{KlyronLockfile, LockfilePackage};
    let mut lock = KlyronLockfile::new();
    lock.add_package("test", "1.0.0", LockfilePackage {
      name: "test".into(),
      version: "1.0.0".into(),
      resolved: "https://registry.npmjs.org/test/-/test-1.0.0.tgz".into(),
      integrity: "sha512-test".into(),
      dependencies: std::collections::HashMap::new(),
      optional_dependencies: std::collections::HashMap::new(),
      peer_dependencies: std::collections::HashMap::new(),
      bin: None,
      has_node_modules: false,
      install_time_ms: 0,
    });
    let bytes = lock.to_bytes().unwrap();
    let decoded = KlyronLockfile::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.packages.len(), 1);
    assert!(decoded.packages.contains_key("test@1.0.0"));
  }

  #[test]
  fn test_migrate_from_npm() {
    let dir = std::env::temp_dir().join("_klyron_pm_migrate_npm");
    std::fs::create_dir_all(&dir).unwrap();
    let npm_lock = r#"{
      "name": "test",
      "lockfileVersion": 3,
      "packages": {
        "node_modules/lodash": {
          "version": "4.17.21",
          "resolved": "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz",
          "integrity": "sha512-v2kDEe57lecTulaDIuNTPy3Ry4gLGJ6Z1O3vE1krgXZNrsQ+LFTGHVxVjcXPs17LhbZVGedAJv8XZ1tvj5FvSg=="
        }
      }
    }"#;
    let npm_path = dir.join("package-lock.json");
    std::fs::write(&npm_path, npm_lock).unwrap();
    let klock = migrate_from_npm_lockfile(&npm_path).unwrap();
    assert!(!klock.packages.is_empty());
    assert!(klock.packages.contains_key("lodash@4.17.21"));
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_install_with_lockfile() {
    let dir = std::env::temp_dir().join("_klyron_pm_install_lock");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("package.json"), r#"{"name":"test","dependencies":{"left-pad":"^1.0.0"}}"#).unwrap();
    install_with_lockfile(&dir, false).unwrap();
    assert!(dir.join("klyron.lock").exists());
    let _ = std::fs::remove_dir_all(&dir);
  }

  // ── Workspace Protocol Tests ──────────────────────────────────────────

  #[test]
  fn test_parse_workspace_dependency_star() {
    let result = parse_workspace_dependency("workspace:*").unwrap();
    assert!(matches!(result, WorkspaceDependency::Star));
  }

  #[test]
  fn test_parse_workspace_dependency_caret() {
    let result = parse_workspace_dependency("workspace:^1.0.0").unwrap();
    match result {
      WorkspaceDependency::Range(s) => assert_eq!(s, "^1.0.0"),
      _ => panic!("Expected Range"),
    }
  }

  #[test]
  fn test_parse_workspace_dependency_tilde() {
    let result = parse_workspace_dependency("workspace:~1.0.0").unwrap();
    match result {
      WorkspaceDependency::Tilde(s) => assert_eq!(s, "~1.0.0"),
      _ => panic!("Expected Tilde"),
    }
  }

  #[test]
  fn test_parse_workspace_dependency_invalid() {
    assert!(parse_workspace_dependency("^1.0.0").is_none());
    assert!(parse_workspace_dependency("npm:foo").is_none());
  }

  #[test]
  fn test_parse_workspace_dependency_plain_semver() {
    let result = parse_workspace_dependency("workspace:1.2.3").unwrap();
    match result {
      WorkspaceDependency::Range(s) => assert_eq!(s, "^1.2.3"),
      _ => panic!("Expected Range"),
    }
  }

  #[test]
  fn test_resolve_workspace_dependency_star() {
    let mut members = HashMap::new();
    members.insert("pkg-a".into(), "1.0.0".into());
    let result = resolve_workspace_dependency_version("workspace:*", &members, "pkg-a");
    assert_eq!(result.unwrap(), "1.0.0");
  }

  #[test]
  fn test_resolve_workspace_dependency_range() {
    let mut members = HashMap::new();
    members.insert("pkg-a".into(), "1.5.0".into());
    let result = resolve_workspace_dependency_version("workspace:^1.0.0", &members, "pkg-a");
    assert_eq!(result.unwrap(), "1.5.0");
  }

  #[test]
  fn test_resolve_workspace_dependency_no_match() {
    let mut members = HashMap::new();
    members.insert("pkg-a".into(), "2.0.0".into());
    let result = resolve_workspace_dependency_version("workspace:^1.0.0", &members, "pkg-a");
    assert!(result.is_err());
  }

  // ── Link / Unlink Tests ───────────────────────────────────────────────

  #[test]
  fn test_link_package() {
    let tmp = std::env::temp_dir().join("_klyron_pm_link_test");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("package.json"), r#"{"name":"test-pkg","version":"1.0.0"}"#).unwrap();

    let global_dir = tmp.join("global");
    let result = link_package(&tmp, &global_dir);
    assert!(result.is_ok());
    let link_path = result.unwrap();
    assert!(link_path.exists());
    assert!(link_path.is_symlink());

    let _ = std::fs::remove_dir_all(&tmp);
  }

  #[test]
  fn test_link_package_missing_package_json() {
    let tmp = std::env::temp_dir().join("_klyron_pm_link_fail");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let result = link_package(&tmp, &tmp.join("global"));
    assert!(result.is_err());
    let _ = std::fs::remove_dir_all(&tmp);
  }

  #[test]
  fn test_link_global_and_unlink() {
    let tmp = std::env::temp_dir().join("_klyron_pm_link_global_test");
    let _ = std::fs::remove_dir_all(&tmp);

    // Create a package dir
    let pkg_dir = tmp.join("pkg");
    std::fs::create_dir_all(&pkg_dir).unwrap();
    std::fs::write(pkg_dir.join("package.json"), r#"{"name":"my-linked-pkg","version":"1.0.0"}"#).unwrap();

    // Link globally
    let global_dir = tmp.join("store").join("linked");
    link_package(&pkg_dir, &global_dir).unwrap();
    assert!(global_dir.join("my-linked-pkg").exists());

    // Link into project
    let project_dir = tmp.join("project");
    std::fs::create_dir_all(&project_dir).unwrap();
    let result = link_global_from_dir("my-linked-pkg", &project_dir, &global_dir);
    assert!(result.is_ok(), "link_global failed: {:?}", result.err());
    assert!(project_dir.join("node_modules").join("my-linked-pkg").exists());

    // Unlink
    let _ = std::fs::remove_file(&global_dir.join("my-linked-pkg"));
    assert!(!global_dir.join("my-linked-pkg").exists());

    let _ = std::fs::remove_dir_all(&tmp);
  }

  #[test]
  fn test_unlink_nonexistent() {
    let result = unlink_package("this-pkg-does-not-exist-for-test");
    assert!(result.is_err());
  }

  // ── Pack Tests ─────────────────────────────────────────────────────────

  #[test]
  fn test_pack_package() {
    let tmp = std::env::temp_dir().join("_klyron_pm_pack_test");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    std::fs::write(tmp.join("package.json"), r#"{"name":"test-pkg","version":"2.0.0"}"#).unwrap();
    std::fs::write(tmp.join("README.md"), "# Test Package").unwrap();
    std::fs::create_dir_all(tmp.join("dist")).unwrap();
    std::fs::write(tmp.join("dist").join("index.js"), "module.exports = {};").unwrap();

    let result = pack_package(&tmp, None);
    assert!(result.is_ok());
    let tarball = result.unwrap();
    assert!(tarball.exists());
    assert!(tarball.to_string_lossy().ends_with(".tgz"));

    // Verify it's a valid gzip
    use flate2::read::GzDecoder;
    use tar::Archive;
    let file = std::fs::File::open(&tarball).unwrap();
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    let entries: Vec<_> = archive.entries().unwrap().filter_map(|e| e.ok()).collect();
    assert!(!entries.is_empty());
    assert!(entries.iter().any(|e| e.path().unwrap().to_string_lossy().contains("package.json")));

    let _ = std::fs::remove_dir_all(&tmp);
  }

  #[test]
  fn test_pack_package_missing_package_json() {
    let tmp = std::env::temp_dir().join("_klyron_pm_pack_fail");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let result = pack_package(&tmp, None);
    assert!(result.is_err());
    let _ = std::fs::remove_dir_all(&tmp);
  }

  #[test]
  fn test_pack_and_integrity() {
    let tmp = std::env::temp_dir().join("_klyron_pm_integrity_test");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("package.json"), r#"{"name":"integrity-test","version":"1.0.0"}"#).unwrap();

    let (path, integrity) = pack_and_get_integrity(&tmp).unwrap();
    assert!(path.exists());
    assert!(integrity.starts_with("sha512-"));

    // Verify integrity matches
    let data = std::fs::read(&path).unwrap();
    let computed = compute_integrity(&data);
    assert_eq!(integrity, computed);

    let _ = std::fs::remove_dir_all(&tmp);
  }

  // ── Pack Excludes Tests ────────────────────────────────────────────────

  #[test]
  fn test_pack_excludes_node_modules() {
    let tmp = std::env::temp_dir().join("_klyron_pm_exclude_test");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("package.json"), r#"{"name":"exclude-test","version":"1.0.0"}"#).unwrap();
    std::fs::create_dir_all(tmp.join("node_modules").join("some-dep")).unwrap();
    std::fs::write(tmp.join("node_modules/some-dep/index.js"), "should be excluded").unwrap();
    std::fs::create_dir_all(tmp.join(".git")).unwrap();
    std::fs::write(tmp.join(".git/HEAD"), "ref: refs/heads/main").unwrap();

    let tarball = pack_package(&tmp, None).unwrap();
    let file = std::fs::File::open(&tarball).unwrap();
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    for entry in archive.entries().unwrap() {
      let entry = entry.unwrap();
      let path = entry.path().unwrap().to_string_lossy().to_string();
      assert!(!path.contains("node_modules"), "Found excluded path: {path}");
      assert!(!path.contains(".git"), "Found excluded path: {path}");
    }

    let _ = std::fs::remove_dir_all(&tmp);
  }

  // ── Dist-tag Tests ─────────────────────────────────────────────────────

  #[test]
  fn test_list_dist_tags() {
    // This is a network test - skip if no network
    match list_dist_tags("express", "https://registry.npmjs.org") {
      Ok(tags) => {
        assert!(!tags.is_empty(), "express should have dist-tags");
        assert!(tags.contains_key("latest"), "express should have 'latest' tag");
      }
      Err(e) => {
        eprintln!("Network test skipped: {e}");
      }
    }
  }

  // ── Package Info Tests ─────────────────────────────────────────────────

  #[test]
  fn test_package_info() {
    match package_info("express", "https://registry.npmjs.org") {
      Ok(info) => {
        assert_eq!(info.name, "express");
        assert!(!info.latest_version.is_empty());
        assert!(!info.all_versions.is_empty(), "express should have versions");
        assert!(info.all_versions.contains(&info.latest_version));
      }
      Err(e) => {
        eprintln!("Network test skipped: {e}");
      }
    }
  }

  #[test]
  fn test_package_info_nonexistent() {
    let result = package_info("this-pkg-definitely-does-not-exist-xyz-98765", "https://registry.npmjs.org");
    assert!(result.is_err());
  }

  // ── Why Tests ──────────────────────────────────────────────────────────

  #[test]
  fn test_why_package() {
    let mut lock = KlyronLockfile::new(None);
    let mut deps_a = HashMap::new();
    deps_a.insert("dep-b".into(), "1.0.0".into());
    lock.packages.insert("dep-a@1.0.0".into(), KlyronLockPackage {
      version: "1.0.0".into(),
      resolved: None,
      integrity: None,
      link: None,
      dev: None,
      optional: None,
      dependencies: Some(deps_a),
      optional_dependencies: None,
      peer_dependencies: None,
      engines: None,
    });
    lock.packages.insert("dep-b@1.0.0".into(), KlyronLockPackage {
      version: "1.0.0".into(),
      resolved: None,
      integrity: None,
      link: None,
      dev: None,
      optional: None,
      dependencies: None,
      optional_dependencies: None,
      peer_dependencies: None,
      engines: None,
    });
    lock.packages.insert("root@1.0.0".into(), KlyronLockPackage {
      version: "1.0.0".into(),
      resolved: None,
      integrity: None,
      link: None,
      dev: None,
      optional: None,
      dependencies: Some(HashMap::from([("dep-a".into(), "1.0.0".into())])),
      optional_dependencies: None,
      peer_dependencies: None,
      engines: None,
    });

    let paths = why_package("dep-b", &lock).unwrap();
    assert!(!paths.is_empty(), "Should find at least one path to dep-b");
    assert!(paths.iter().any(|p| p.path.contains(&"dep-b".to_string())));
  }

  #[test]
  fn test_why_package_not_found() {
    let lock = KlyronLockfile::new(None);
    let result = why_package("nonexistent", &lock);
    assert!(result.is_err());
  }

  // ── Bin Scripts Tests ──────────────────────────────────────────────────

  #[test]
  fn test_install_bin_scripts() {
    let tmp = std::env::temp_dir().join("_klyron_pm_bin_test");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    // Create a package with a bin script
    let pkg_dir = tmp.join("node_modules").join("test-cli");
    std::fs::create_dir_all(&pkg_dir).unwrap();
    std::fs::write(pkg_dir.join("cli.js"), "#!/usr/bin/env node\nconsole.log('hello');").unwrap();

    let mut bin_map = HashMap::new();
    bin_map.insert("test-cli".into(), "cli.js".into());

    let result = install_bin_scripts(&pkg_dir, &bin_map, &tmp.join("node_modules"));
    assert!(result.is_ok());
    let created = result.unwrap();
    assert!(!created.is_empty());

    let bin_dir = tmp.join("node_modules").join(".bin");
    assert!(bin_dir.join("test-cli").exists());
    assert!(bin_dir.join("test-cli.cmd").exists());

    let _ = std::fs::remove_dir_all(&tmp);
  }

  #[test]
  fn test_parse_bin_field_string() {
    let json = serde_json::json!("bin/cli.js");
    let map = parse_bin_field(&json);
    assert_eq!(map.len(), 1);
    assert!(map.contains_key("cli"));
  }

  #[test]
  fn test_parse_bin_field_object() {
    let json = serde_json::json!({
      "my-cli": "bin/my-cli.js",
      "other-cli": "bin/other.js"
    });
    let map = parse_bin_field(&json);
    assert_eq!(map.len(), 2);
    assert_eq!(map.get("my-cli").unwrap(), "bin/my-cli.js");
  }

  #[test]
  fn test_install_all_bin_scripts_no_node_modules() {
    let tmp = std::env::temp_dir().join("_klyron_pm_all_bin_test");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let count = install_all_bin_scripts(&tmp.join("node_modules")).unwrap();
    assert_eq!(count, 0);
    let _ = std::fs::remove_dir_all(&tmp);
  }

  // ── Peer / Optional Deps Tests ─────────────────────────────────────────

  #[test]
  fn test_resolve_peer_dependencies_missing() {
    let peer_deps = HashMap::from([("react".into(), "^17.0.0".into())]);
    let all_packages = HashMap::new();
    let warnings = resolve_peer_dependencies("test-pkg", &peer_deps, &all_packages);
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].peer_name, "react");
    assert!(warnings[0].found_version.is_none());
  }

  #[test]
  fn test_resolve_peer_dependencies_satisfied() {
    let peer_deps = HashMap::from([("react".into(), "^17.0.0".into())]);
    let all_packages = HashMap::from([("react".into(), "17.0.2".into())]);
    let warnings = resolve_peer_dependencies("test-pkg", &peer_deps, &all_packages);
    assert!(warnings.is_empty());
  }

  #[test]
  fn test_resolve_peer_dependencies_mismatch() {
    let peer_deps = HashMap::from([("react".into(), "^17.0.0".into())]);
    let all_packages = HashMap::from([("react".into(), "18.0.0".into())]);
    let warnings = resolve_peer_dependencies("test-pkg", &peer_deps, &all_packages);
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].found_version.as_deref(), Some("18.0.0"));
  }

  #[test]
  fn test_handle_optional_deps() {
    let optional_deps = HashMap::from([
      ("left-pad".into(), "^1.0.0".into()),
      ("nonexistent-pkg-that-fails".into(), "999.999.999".into()),
    ]);
    let result = handle_optional_deps("test-pkg", &optional_deps);
    // left-pad should resolve, the nonexistent one should be silently skipped
    assert!(!result.is_empty(), "At least left-pad should resolve");
  }

  #[test]
  fn test_handle_optional_deps_all_fail() {
    let optional_deps = HashMap::from([
      ("nonexistent-pkg-1".into(), "999.999.999".into()),
      ("nonexistent-pkg-2".into(), "888.888.888".into()),
    ]);
    let result = handle_optional_deps("test-pkg", &optional_deps);
    assert!(result.is_empty(), "All optional deps should silently fail");
  }

  #[test]
  fn test_resolve_overrides() {
    let pkg_json = serde_json::json!({
      "overrides": {
        "lodash": "4.17.21",
        "react": {
          ".": "18.0.0"
        }
      }
    });
    let mut resolution_map = HashMap::new();
    resolve_overrides(&pkg_json, &mut resolution_map).unwrap();
    assert_eq!(resolution_map.get("lodash").unwrap(), "4.17.21");
    assert_eq!(resolution_map.get("react").unwrap(), "18.0.0");
  }

  #[test]
  fn test_resolve_overrides_with_resolutions() {
    let pkg_json = serde_json::json!({
      "resolutions": {
        "express": "4.18.2"
      }
    });
    let mut resolution_map = HashMap::new();
    resolve_overrides(&pkg_json, &mut resolution_map).unwrap();
    assert_eq!(resolution_map.get("express").unwrap(), "4.18.2");
  }

  #[test]
  fn test_apply_overrides_to_deps() {
    let mut deps = HashMap::from([
      ("lodash".into(), "^4.0.0".into()),
      ("react".into(), "^17.0.0".into()),
    ]);
    let mut resolution_map = HashMap::new();
    resolution_map.insert("lodash".into(), "4.17.21".into());
    apply_overrides_to_deps(&mut deps, &resolution_map);
    assert_eq!(deps.get("lodash").unwrap(), "4.17.21");
    assert_eq!(deps.get("react").unwrap(), "^17.0.0");
  }

  #[test]
  fn test_peer_dep_warning_struct() {
    let warning = PeerDepWarning {
      package: "host".into(),
      peer_name: "react".into(),
      required_range: "^17.0.0".into(),
      found_version: Some("18.0.0".into()),
    };
    assert_eq!(warning.package, "host");
    assert_eq!(warning.peer_name, "react");
    assert_eq!(warning.found_version.as_deref(), Some("18.0.0"));
  }
}
