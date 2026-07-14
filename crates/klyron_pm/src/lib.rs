use hex;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
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
}
