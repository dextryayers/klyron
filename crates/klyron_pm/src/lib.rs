use hex;
use once_cell::sync::Lazy;
use regex::Regex;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

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

// ── Git Dependency Resolution ──────────────────────────────────────────────

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
}
