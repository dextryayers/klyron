use crate::{PmError, KlyronLockfile as MainKlyronLockfile, KlyronLockPackage, LockfileV3};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

const KLYRON_MAGIC: &[u8] = b"KLYR";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfilePackage {
  pub name: String,
  pub version: String,
  pub resolved: String,
  pub integrity: String,
  pub dependencies: HashMap<String, String>,
  pub optional_dependencies: HashMap<String, String>,
  pub peer_dependencies: HashMap<String, String>,
  pub bin: Option<HashMap<String, String>>,
  pub has_node_modules: bool,
  pub install_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfileMetadata {
  pub created_at: String,
  pub klyron_version: String,
  pub install_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlyronLockfile {
  pub lockfile_version: u32,
  pub metadata: LockfileMetadata,
  pub packages: HashMap<String, LockfilePackage>,
}

impl KlyronLockfile {
  pub fn new() -> Self {
    Self {
      lockfile_version: 1,
      metadata: LockfileMetadata {
        created_at: chrono::Utc::now().to_rfc3339(),
        klyron_version: env!("CARGO_PKG_VERSION").to_string(),
        install_count: 0,
      },
      packages: HashMap::new(),
    }
  }

  pub fn to_bytes(&self) -> Result<Vec<u8>, PmError> {
    let mut buf = Vec::with_capacity(4096);
    buf.extend_from_slice(KLYRON_MAGIC);
    let payload = bincode::serialize(self)
      .map_err(|e| PmError::LockfileError(format!("Bincode serialize: {e}")))?;
    buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    buf.extend_from_slice(&payload);
    Ok(buf)
  }

  pub fn from_bytes(bytes: &[u8]) -> Result<Self, PmError> {
    if bytes.len() < 8 {
      return Err(PmError::LockfileError("Truncated lockfile".into()));
    }
    if &bytes[..4] != KLYRON_MAGIC {
      return Err(PmError::LockfileError("Invalid magic bytes".into()));
    }
    let payload_len = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    if bytes.len() < 8 + payload_len {
      return Err(PmError::LockfileError("Truncated lockfile payload".into()));
    }
    bincode::deserialize(&bytes[8..8 + payload_len])
      .map_err(|e| PmError::LockfileError(format!("Bincode deserialize: {e}")))
  }

  pub fn to_json_pretty(&self) -> Result<String, PmError> {
    serde_json::to_string_pretty(self)
      .map_err(|e| PmError::LockfileError(format!("JSON serialize: {e}")))
  }

  pub fn from_json(json: &str) -> Result<Self, PmError> {
    serde_json::from_str(json)
      .map_err(|e| PmError::LockfileError(format!("JSON deserialize: {e}")))
  }

  pub fn add_package(&mut self, name: &str, version: &str, pkg: LockfilePackage) {
    let key = format!("{name}@{version}");
    self.packages.insert(key, pkg);
    self.metadata.install_count = self.packages.len() as u64;
  }

  pub fn remove_package(&mut self, name: &str) {
    let keys: Vec<String> = self.packages.keys()
      .filter(|k| k.starts_with(&format!("{name}@")))
      .cloned()
      .collect();
    for k in keys {
      self.packages.remove(&k);
    }
    self.metadata.install_count = self.packages.len() as u64;
  }

  pub fn get_package(&self, name: &str) -> Option<&LockfilePackage> {
    let mut candidates: Vec<&String> = self.packages.keys()
      .filter(|k| k.starts_with(&format!("{name}@")))
      .collect();
    candidates.sort();
    candidates.last().and_then(|k| self.packages.get(*k))
  }

  pub fn has_changed(&self, other: &Self) -> bool {
    if self.packages.len() != other.packages.len() {
      return true;
    }
    for (key, pkg) in &self.packages {
      match other.packages.get(key) {
        Some(other_pkg) => {
          if pkg.version != other_pkg.version
            || pkg.resolved != other_pkg.resolved
            || pkg.integrity != other_pkg.integrity
            || pkg.dependencies != other_pkg.dependencies
          {
            return true;
          }
        }
        None => return true,
      }
    }
    false
  }

  pub fn merge(&self, other: &Self) -> Self {
    let mut merged = self.clone();
    for (key, pkg) in &other.packages {
      merged.packages.entry(key.clone()).or_insert_with(|| pkg.clone());
    }
    merged.metadata.install_count = merged.packages.len() as u64;
    if other.metadata.created_at > merged.metadata.created_at {
      merged.metadata.created_at = other.metadata.created_at.clone();
    }
    merged
  }

  pub fn verify_integrity(&self, dir: &Path) -> Result<Vec<String>, PmError> {
    let mut mismatches = Vec::new();
    for (key, pkg) in &self.packages {
      let parts: Vec<&str> = key.split('@').collect();
      let name = parts[0];
      let pkg_dir = dir.join("node_modules").join(name);
      if !pkg_dir.exists() {
        mismatches.push(format!("{key}: missing node_modules/{name}"));
        continue;
      }
      let pkg_json_path = pkg_dir.join("package.json");
      if pkg_json_path.exists() {
        if let Ok(data) = std::fs::read(&pkg_json_path) {
          let actual = crate::compute_integrity(&data);
          if actual != pkg.integrity {
            mismatches.push(format!("{key}: integrity mismatch"));
          }
        }
      }
    }
    Ok(mismatches)
  }

  pub fn frozen_check(&self, dir: &Path) -> Result<(), PmError> {
    let klyron_lock_path = dir.join("klyron.lock");
    if !klyron_lock_path.exists() {
      return Err(PmError::LockfileError("klyron.lock not found (frozen)".into()));
    }
    let existing = std::fs::read(&klyron_lock_path)?;
    let existing_lock = KlyronLockfile::from_bytes(&existing)?;
    if self.has_changed(&existing_lock) {
      return Err(PmError::LockfileError(
        "klyron.lock is frozen but does not match node_modules. Run install to update.".into(),
      ));
    }
    Ok(())
  }

  pub fn resolve_dependency(name: &str, version_range: &str) -> Result<String, PmError> {
    crate::resolve_version(name, version_range)
  }

  pub fn find_optimal_version(
    name: &str,
    range: &str,
    existing_packages: &HashMap<String, LockfilePackage>,
  ) -> Result<String, PmError> {
    let resolved = Self::resolve_dependency(name, range)?;
    for (key, _) in existing_packages {
      if key.starts_with(&format!("{name}@")) {
        let existing_ver = key.split('@').last().unwrap_or("");
        if existing_ver == &resolved {
          return Ok(resolved);
        }
      }
    }
    Ok(resolved)
  }
}

// ── Diffing ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffKind {
  Added,
  Removed,
  Changed,
  Upgraded,
  Downgraded,
}

#[derive(Debug, Clone)]
pub struct DiffEntry {
  pub kind: DiffKind,
  pub name: String,
  pub old_version: Option<String>,
  pub new_version: Option<String>,
}

pub fn lockfile_diff(a: &KlyronLockfile, b: &KlyronLockfile) -> Vec<DiffEntry> {
  let mut entries = Vec::new();
  let a_keys: HashSet<&str> = a.packages.keys().map(|s| s.as_str()).collect();
  let b_keys: HashSet<&str> = b.packages.keys().map(|s| s.as_str()).collect();

  for key in b_keys.difference(&a_keys) {
    if let Some(pkg) = b.packages.get(*key) {
      entries.push(DiffEntry {
        kind: DiffKind::Added,
        name: format!("{}@{}", pkg.name, pkg.version),
        old_version: None,
        new_version: Some(pkg.version.clone()),
      });
    }
  }

  for key in a_keys.difference(&b_keys) {
    if let Some(pkg) = a.packages.get(*key) {
      entries.push(DiffEntry {
        kind: DiffKind::Removed,
        name: format!("{}@{}", pkg.name, pkg.version),
        old_version: Some(pkg.version.clone()),
        new_version: None,
      });
    }
  }

  for key in a_keys.intersection(&b_keys) {
    let a_pkg = &a.packages[*key];
    let b_pkg = &b.packages[*key];
    if a_pkg.version != b_pkg.version {
      let kind = if semver::Version::parse(&b_pkg.version).ok()
        .zip(semver::Version::parse(&a_pkg.version).ok())
        .map(|(b_ver, a_ver)| b_ver > a_ver)
        .unwrap_or(false)
      {
        DiffKind::Upgraded
      } else if a_pkg.version != b_pkg.version {
        DiffKind::Downgraded
      } else {
        DiffKind::Changed
      };
      entries.push(DiffEntry {
        kind,
        name: format!("{}@{}", a_pkg.name, a_pkg.version),
        old_version: Some(a_pkg.version.clone()),
        new_version: Some(b_pkg.version.clone()),
      });
    }
  }

  entries
}

pub fn print_diff(entries: &[DiffEntry]) {
  if entries.is_empty() {
    println!("No differences between lockfiles");
    return;
  }
  for entry in entries {
    let symbol = match entry.kind {
      DiffKind::Added => '+',
      DiffKind::Removed => '-',
      DiffKind::Changed => '~',
      DiffKind::Upgraded => '^',
      DiffKind::Downgraded => 'v',
    };
    match entry.kind {
      DiffKind::Added => println!(" {} {} (new)", symbol, entry.name),
      DiffKind::Removed => println!(" {} {} (removed)", symbol, entry.name),
      _ => println!(" {} {} ({} -> {})", symbol, entry.name, entry.old_version.as_deref().unwrap_or("?"), entry.new_version.as_deref().unwrap_or("?")),
    }
  }
}

// ── Auto-repair ──────────────────────────────────────────────────────────────

impl KlyronLockfile {
  pub fn try_repair(&mut self, dir: &Path) -> Result<Vec<String>, PmError> {
    let mut repairs = Vec::new();

    // Repair 1: remove duplicate entries
    let mut seen: HashSet<String> = HashSet::new();
    let dupes: Vec<String> = self.packages.keys()
      .filter(|k| !seen.insert(k.to_string()))
      .cloned()
      .collect();
    for key in &dupes {
      self.packages.remove(key);
      repairs.push(format!("Removed duplicate: {key}"));
    }

    // Repair 2: fix missing resolved URLs
    for (key, pkg) in &self.packages.clone() {
      if pkg.resolved.is_empty() {
        if let Some(pkg_mut) = self.packages.get_mut(key) {
          pkg_mut.resolved = format!("https://registry.npmjs.org/{}/-/{}-{}.tgz", pkg.name, pkg.name, pkg.version);
          repairs.push(format!("Added missing resolved URL for {key}"));
        }
      }
    }

    // Repair 3: fix negative/zero install times
    for (key, pkg) in &self.packages.clone() {
      if pkg.install_time_ms == 0 || pkg.install_time_ms == u64::MAX {
        if let Some(pkg_mut) = self.packages.get_mut(key) {
          pkg_mut.install_time_ms = 1;
          repairs.push(format!("Fixed install_time for {key}"));
        }
      }
    }

    // Repair 4: regenerate integrity from node_modules if available
    let nm_dir = dir.join("node_modules");
    if nm_dir.exists() {
      for (key, pkg) in &self.packages.clone() {
        let pkg_dir = nm_dir.join(&pkg.name);
        let pkg_json_path = pkg_dir.join("package.json");
        if pkg_json_path.exists() {
          if let Ok(data) = std::fs::read(&pkg_json_path) {
            let actual = crate::compute_integrity(&data);
            if actual != pkg.integrity {
              if let Some(pkg_mut) = self.packages.get_mut(key) {
                pkg_mut.integrity = actual;
                repairs.push(format!("Regenerated integrity for {key} from node_modules"));
              }
            }
          }
        }
      }
    }

    // Repair 5: try JSON fallback if binary data was corrupted
    // (handled by caller)

    // Repair 6: normalize metadata
    self.metadata.install_count = self.packages.len() as u64;

    repairs.sort();
    repairs.dedup();
    Ok(repairs)
  }

  pub fn from_bytes_with_repair(bytes: &[u8], dir: &Path) -> (Result<Self, PmError>, Vec<String>) {
    match Self::from_bytes(bytes) {
      Ok(lock) => {
        let mut l = lock;
        match l.try_repair(dir) {
          Ok(repairs) => (Ok(l), repairs),
          Err(e) => (Err(e), Vec::new()),
        }
      }
      Err(bin_err) => {
        // Try JSON fallback
        if let Ok(text) = std::str::from_utf8(bytes) {
          match Self::from_json(text) {
            Ok(json_lock) => {
              let mut l = json_lock;
              let repairs = l.try_repair(dir).unwrap_or_default();
              let mut all = vec!["Recovered from JSON fallback after binary parse error".to_string()];
              all.extend(repairs);
              (Ok(l), all)
            }
            Err(_) => (Err(bin_err), Vec::new()),
          }
        } else {
          (Err(bin_err), Vec::new())
        }
      }
    }
  }
}

impl Default for KlyronLockfile {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn make_test_lockfile() -> KlyronLockfile {
    let mut lock = KlyronLockfile::new();
    lock.add_package("lodash", "4.17.21", LockfilePackage {
      name: "lodash".into(),
      version: "4.17.21".into(),
      resolved: "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz".into(),
      integrity: "sha512-v2kDEe57lecTulaDIuNTPy3Ry4gLGJ6Z1O3vE1krgXZNrsQ+LFTGHVxVjcXPs17LhbZVGedAJv8XZ1tvj5FvSg==".into(),
      dependencies: HashMap::new(),
      optional_dependencies: HashMap::new(),
      peer_dependencies: HashMap::new(),
      bin: None,
      has_node_modules: false,
      install_time_ms: 150,
    });
    lock.add_package("express", "4.18.2", LockfilePackage {
      name: "express".into(),
      version: "4.18.2".into(),
      resolved: "https://registry.npmjs.org/express/-/express-4.18.2.tgz".into(),
      integrity: "sha512-5T6P4xPgpp0YDFvSW1EZ5SJvOBAT6mNb4H1WIQ7Wk1g6MqBx6RZPit8WZ1H8+UZFDbZ7CXHkBhCJgwFqK8z5g==".into(),
      dependencies: HashMap::from([
        ("accepts".into(), "1.3.8".into()),
        ("array-flatten".into(), "1.1.1".into()),
      ]),
      optional_dependencies: HashMap::new(),
      peer_dependencies: HashMap::new(),
      bin: None,
      has_node_modules: false,
      install_time_ms: 230,
    });
    lock
  }

  #[test]
  fn test_binary_roundtrip() {
    let lock = make_test_lockfile();
    let bytes = lock.to_bytes().unwrap();
    assert!(bytes.starts_with(b"KLYR"));
    let decoded = KlyronLockfile::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.lockfile_version, 1);
    assert_eq!(decoded.packages.len(), 2);
    assert!(decoded.packages.contains_key("lodash@4.17.21"));
    assert!(decoded.packages.contains_key("express@4.18.2"));
    let lodash = decoded.packages.get("lodash@4.17.21").unwrap();
    assert_eq!(lodash.integrity, "sha512-v2kDEe57lecTulaDIuNTPy3Ry4gLGJ6Z1O3vE1krgXZNrsQ+LFTGHVxVjcXPs17LhbZVGedAJv8XZ1tvj5FvSg==");
  }

  #[test]
  fn test_json_roundtrip() {
    let lock = make_test_lockfile();
    let json = lock.to_json_pretty().unwrap();
    let decoded = KlyronLockfile::from_json(&json).unwrap();
    assert_eq!(decoded.packages.len(), 2);
    assert_eq!(decoded.lockfile_version, 1);
    let express = decoded.packages.get("express@4.18.2").unwrap();
    assert_eq!(express.dependencies.len(), 2);
  }

  #[test]
  fn test_merge() {
    let lock1 = make_test_lockfile();
    let mut lock2 = KlyronLockfile::new();
    lock2.add_package("react", "18.2.0", LockfilePackage {
      name: "react".into(),
      version: "18.2.0".into(),
      resolved: "https://registry.npmjs.org/react/-/react-18.2.0.tgz".into(),
      integrity: "sha512-/3IjMdb2L9QbBdWiW5e3P2/npwMBaU9mHCSCUzNln0ZCYbcfTsGbTJrU/kGemdH2IWmB2ioZ+zkxtmq6g09fGQ==".into(),
      dependencies: HashMap::new(),
      optional_dependencies: HashMap::new(),
      peer_dependencies: HashMap::new(),
      bin: None,
      has_node_modules: false,
      install_time_ms: 120,
    });
    let merged = lock1.merge(&lock2);
    assert_eq!(merged.packages.len(), 3);
    assert!(merged.packages.contains_key("react@18.2.0"));
  }

  #[test]
  fn test_has_changed() {
    let lock1 = make_test_lockfile();
    let lock2 = make_test_lockfile();
    assert!(!lock1.has_changed(&lock2));
    let mut lock3 = lock2.clone();
    if let Some(pkg) = lock3.packages.get_mut("lodash@4.17.21") {
      pkg.version = "4.17.22".into();
    }
    assert!(lock1.has_changed(&lock3));
  }

  #[test]
  fn test_get_package() {
    let lock = make_test_lockfile();
    let pkg = lock.get_package("lodash").unwrap();
    assert_eq!(pkg.version, "4.17.21");
    let none = lock.get_package("nonexistent");
    assert!(none.is_none());
  }

  #[test]
  fn test_remove_package() {
    let mut lock = make_test_lockfile();
    lock.remove_package("lodash");
    assert_eq!(lock.packages.len(), 1);
    assert!(lock.get_package("lodash").is_none());
  }

  #[test]
  fn test_binary_compactness() {
    let mut lock = KlyronLockfile::new();
    for i in 0..100 {
      let name = format!("pkg{i}");
      let ver = format!("1.{i}.0");
      lock.add_package(&name, &ver, LockfilePackage {
        name: name.clone(),
        version: ver.clone(),
        resolved: format!("https://registry.npmjs.org/{name}/-/{name}-{ver}.tgz"),
        integrity: format!("sha512-{}", "abcdef0123456789".repeat(4)),
        dependencies: HashMap::from([
          ("dep-a".into(), "1.0.0".into()),
          ("dep-b".into(), "2.0.0".into()),
        ]),
        optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::from([
          ("peer-a".into(), "^1.0.0".into()),
        ]),
        bin: Some(HashMap::from([("cli".into(), "bin/cli.js".into())])),
        has_node_modules: true,
        install_time_ms: 42 + i as u64,
      });
    }
    let bytes = lock.to_bytes().unwrap();
    let json = lock.to_json_pretty().unwrap();
    assert!(bytes.len() < json.len() * 7 / 10, "Binary ({}) should be smaller than JSON ({})", bytes.len(), json.len());
  }

  #[test]
  fn test_new_lockfile_defaults() {
    let lock = KlyronLockfile::new();
    assert_eq!(lock.lockfile_version, 1);
    assert!(lock.packages.is_empty());
    assert_eq!(lock.metadata.install_count, 0);
  }

  #[test]
  fn test_resolve_dependency() {
    let ver = KlyronLockfile::resolve_dependency("test-pkg", "^1.0.0").unwrap();
    assert!(!ver.is_empty());
  }

  #[test]
  fn test_rejects_bad_magic() {
    let result = KlyronLockfile::from_bytes(b"BADS");
    assert!(result.is_err());
  }

  #[test]
  fn test_rejects_truncated() {
    let result = KlyronLockfile::from_bytes(b"KLYR");
    assert!(result.is_err());
  }
}
