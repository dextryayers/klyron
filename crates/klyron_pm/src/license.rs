use crate::PmError;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseViolation {
    pub package: String,
    pub version: String,
    pub license: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepInfo {
    pub name: String,
    pub version: String,
    pub license: Option<String>,
    pub path: Option<String>,
}

pub fn check_license_compliance(deps: &[DepInfo], allowed: &[String]) -> Vec<LicenseViolation> {
    let mut violations = Vec::new();
    for dep in deps {
        if let Some(ref lic) = dep.license {
            let lic_lower = lic.to_lowercase();
            let is_allowed = allowed.iter().any(|a| {
                let a_lower = a.to_lowercase();
                lic_lower == a_lower || lic_lower.starts_with(&a_lower)
            });
            if !is_allowed {
                violations.push(LicenseViolation {
                    package: dep.name.clone(),
                    version: dep.version.clone(),
                    license: lic.clone(),
                    required: true,
                });
            }
        } else {
            violations.push(LicenseViolation {
                package: dep.name.clone(),
                version: dep.version.clone(),
                license: "unknown".into(),
                required: false,
            });
        }
    }
    violations
}

pub fn scan_node_modules_for_licenses(node_modules_dir: &Path) -> Result<Vec<DepInfo>, PmError> {
    let mut deps = Vec::new();
    if !node_modules_dir.exists() {
        return Ok(deps);
    }
    for entry in std::fs::read_dir(node_modules_dir)
        .map_err(|e| PmError::IoError(format!("Failed to read node_modules: {e}")))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.is_empty() || name.starts_with('.') {
            continue;
        }
        if name == "@" {
            if let Ok(scoped) = std::fs::read_dir(&path) {
                for se in scoped.flatten() {
                    let sp = se.path();
                    if let Some(dep) = scan_single_package(&sp)? {
                        deps.push(dep);
                    }
                }
            }
            continue;
        }
        if let Some(dep) = scan_single_package(&path)? {
            deps.push(dep);
        }
    }
    Ok(deps)
}

fn scan_single_package(pkg_dir: &Path) -> Result<Option<DepInfo>, PmError> {
    let pkg_json = pkg_dir.join("package.json");
    if !pkg_json.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&pkg_json)?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| PmError::IoError(format!("Invalid package.json: {e}")))?;
    let name = json.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let version = json.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0");
    let license = json.get("license").and_then(|l| l.as_str()).map(String::from);
    if name.is_empty() {
        return Ok(None);
    }
    Ok(Some(DepInfo {
        name: name.to_string(),
        version: version.to_string(),
        license,
        path: Some(pkg_dir.to_string_lossy().to_string()),
    }))
}
