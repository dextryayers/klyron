use crate::PmError;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_license_compliance_allowed() {
        let deps = vec![
            DepInfo {
                name: "lodash".into(), version: "4.17.21".into(),
                license: Some("MIT".into()), path: None,
            },
            DepInfo {
                name: "express".into(), version: "4.18.0".into(),
                license: Some("MIT".into()), path: None,
            },
        ];
        let allowed: Vec<String> = vec!["MIT".into()];
        let violations = check_license_compliance(&deps, &allowed);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_license_compliance_violation() {
        let deps = vec![
            DepInfo {
                name: "bad-license".into(), version: "1.0.0".into(),
                license: Some("GPL-3.0".into()), path: None,
            },
        ];
        let allowed: Vec<String> = vec!["MIT".into(), "Apache-2.0".into()];
        let violations = check_license_compliance(&deps, &allowed);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].package, "bad-license");
        assert_eq!(violations[0].license, "GPL-3.0");
        assert!(violations[0].required);
    }

    #[test]
    fn test_check_license_compliance_unknown_license() {
        let deps = vec![
            DepInfo {
                name: "no-license".into(), version: "1.0.0".into(),
                license: None, path: None,
            },
        ];
        let allowed: Vec<String> = vec!["MIT".into()];
        let violations = check_license_compliance(&deps, &allowed);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].license, "unknown");
        assert!(!violations[0].required);
    }

    #[test]
    fn test_check_license_compliance_multiple_allowed() {
        let deps = vec![
            DepInfo {
                name: "pkg-a".into(), version: "1.0.0".into(),
                license: Some("MIT".into()), path: None,
            },
            DepInfo {
                name: "pkg-b".into(), version: "2.0.0".into(),
                license: Some("Apache-2.0".into()), path: None,
            },
            DepInfo {
                name: "pkg-c".into(), version: "3.0.0".into(),
                license: Some("BSD-3-Clause".into()), path: None,
            },
        ];
        let allowed: Vec<String> = vec!["MIT".into(), "Apache-2.0".into()];
        let violations = check_license_compliance(&deps, &allowed);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].package, "pkg-c");
    }

    #[test]
    fn test_license_case_insensitive() {
        let deps = vec![
            DepInfo {
                name: "pkg".into(), version: "1.0.0".into(),
                license: Some("mit".into()), path: None,
            },
        ];
        let allowed: Vec<String> = vec!["MIT".into()];
        let violations = check_license_compliance(&deps, &allowed);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_dep_info_creation() {
        let dep = DepInfo {
            name: "react".into(),
            version: "18.0.0".into(),
            license: Some("MIT".into()),
            path: Some("node_modules/react".into()),
        };
        assert_eq!(dep.name, "react");
        assert_eq!(dep.license.as_deref(), Some("MIT"));
        assert_eq!(dep.path.as_deref(), Some("node_modules/react"));
    }

    #[test]
    fn test_license_violation_display() {
        let violation = LicenseViolation {
            package: "bad".into(), version: "1.0.0".into(),
            license: "GPL-3.0".into(), required: true,
        };
        assert_eq!(violation.package, "bad");
        assert_eq!(violation.license, "GPL-3.0");
    }

    #[test]
    fn test_empty_deps_no_violations() {
        let deps: Vec<DepInfo> = vec![];
        let allowed: Vec<String> = vec!["MIT".into()];
        let violations = check_license_compliance(&deps, &allowed);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_license_starts_with_matching() {
        // The check uses starts_with, so "MIT" matches "MIT" and "MIT License"
        let deps = vec![
            DepInfo {
                name: "pkg".into(), version: "1.0.0".into(),
                license: Some("MIT License".into()), path: None,
            },
        ];
        let allowed: Vec<String> = vec!["MIT".into()];
        let violations = check_license_compliance(&deps, &allowed);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_scan_node_modules_no_dir() {
        let tmp = std::env::temp_dir().join("klyron_license_scan");
        let _ = std::fs::remove_dir_all(&tmp);
        // Directory doesn't exist
        let result = scan_node_modules_for_licenses(&tmp);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}

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
