use std::collections::HashMap;

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use crate::{RegistryError, RegistryKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<DependencyNode>,
    pub dev: bool,
    pub optional: bool,
}

pub fn resolve_dependency_tree(
    name: &str,
    version: &str,
    dependencies: &HashMap<String, String>,
    all_versions: &HashMap<String, Vec<String>>,
) -> Result<DependencyNode, RegistryError> {
    let mut visited = Vec::new();
    build_tree(name, version, dependencies, all_versions, &mut visited)
}

fn build_tree(
    name: &str,
    version: &str,
    dependencies: &HashMap<String, String>,
    all_versions: &HashMap<String, Vec<String>>,
    visited: &mut Vec<String>,
) -> Result<DependencyNode, RegistryError> {
    let key = format!("{}@{}", name, version);
    if visited.contains(&key) {
        return Ok(DependencyNode {
            name: name.to_string(),
            version: version.to_string(),
            dependencies: Vec::new(),
            dev: false,
            optional: false,
        });
    }
    visited.push(key);

    let mut children = Vec::new();
    if let Some(deps) = dependencies.get(name) {
        for (dep_name, dep_constraint) in deps.split(',').filter_map(|s| {
            let mut parts = s.trim().splitn(2, '@');
            Some((parts.next()?, parts.next()?))
        }) {
            if let Some(versions) = all_versions.get(dep_name) {
                if let Some(resolved) = resolve_semver(versions, dep_constraint) {
                    if let Ok(child) = build_tree(
                        dep_name,
                        &resolved,
                        dependencies,
                        all_versions,
                        visited,
                    ) {
                        children.push(child);
                    }
                }
            }
        }
    }

    Ok(DependencyNode {
        name: name.to_string(),
        version: version.to_string(),
        dependencies: children,
        dev: false,
        optional: false,
    })
}

pub fn resolve_semver(versions: &[String], constraint: &str) -> Option<String> {
    if versions.is_empty() {
        return None;
    }
    if constraint == "*" || constraint == "latest" {
        return sort_versions(versions).into_iter().next();
    }
    if let Ok(req) = VersionReq::parse(constraint) {
        let mut matched: Vec<&String> = versions
            .iter()
            .filter(|v| Version::parse(v).map(|ver| req.matches(&ver)).unwrap_or(false))
            .collect();
        matched.sort_by(|a, b| {
            Version::parse(b)
                .unwrap_or(Version::new(0, 0, 0))
                .cmp(&Version::parse(a).unwrap_or(Version::new(0, 0, 0)))
        });
        return matched.first().map(|s| (*s).clone());
    }
    if versions.contains(&constraint.to_string()) {
        return Some(constraint.to_string());
    }
    None
}

pub fn sort_versions(versions: &[String]) -> Vec<String> {
    let mut sorted: Vec<String> = versions.to_vec();
    sorted.sort_by(|a, b| {
        let va = Version::parse(a).unwrap_or(Version::new(0, 0, 0));
        let vb = Version::parse(b).unwrap_or(Version::new(0, 0, 0));
        vb.cmp(&va)
    });
    sorted
}

pub fn satisfies(version: &str, constraint: &str) -> bool {
    if constraint == "*" {
        return true;
    }
    if let Ok(req) = VersionReq::parse(constraint) {
        if let Ok(ver) = Version::parse(version) {
            return req.matches(&ver);
        }
    }
    version == constraint
}

pub fn find_conflicts(
    packages: &HashMap<String, Vec<String>>,
) -> Vec<(String, String, String)> {
    let mut conflicts = Vec::new();
    let mut seen: HashMap<String, (String, String)> = HashMap::new();
    for (name, versions) in packages {
        if let Some(latest) = versions.first() {
            if let Some((prev_pkg, prev_ver)) = seen.get(name) {
                if prev_ver != latest {
                    conflicts.push((
                        name.clone(),
                        prev_ver.clone(),
                        latest.clone(),
                    ));
                }
            } else {
                seen.insert(name.clone(), (String::new(), latest.clone()));
            }
        }
    }
    conflicts
}

pub fn max_satisfying<'a>(
    versions: &'a [String],
    constraint: &str,
) -> Option<&'a String> {
    let req = VersionReq::parse(constraint).ok()?;
    versions
        .iter()
        .filter(|v| Version::parse(v).map(|ver| req.matches(&ver)).unwrap_or(false))
        .max_by(|a, b| {
            Version::parse(b)
                .unwrap_or(Version::new(0, 0, 0))
                .cmp(&Version::parse(a).unwrap_or(Version::new(0, 0, 0)))
        })
}

pub fn min_satisfying<'a>(
    versions: &'a [String],
    constraint: &str,
) -> Option<&'a String> {
    let req = VersionReq::parse(constraint).ok()?;
    versions
        .iter()
        .filter(|v| Version::parse(v).map(|ver| req.matches(&ver)).unwrap_or(false))
        .min_by(|a, b| {
            Version::parse(a)
                .unwrap_or(Version::new(0, 0, 0))
                .cmp(&Version::parse(b).unwrap_or(Version::new(0, 0, 0)))
        })
}

pub fn bump_version(version: &str, part: &str) -> Result<String, RegistryError> {
    let mut ver = Version::parse(version)
        .map_err(|e| RegistryError::ParseError(format!("Invalid version: {e}")))?;
    match part {
        "major" => {
            ver.major += 1;
            ver.minor = 0;
            ver.patch = 0;
        }
        "minor" => {
            ver.minor += 1;
            ver.patch = 0;
        }
        "patch" => {
            ver.patch += 1;
        }
        "premajor" => {
            ver.major += 1;
            ver.minor = 0;
            ver.patch = 0;
            ver.pre = semver::Prerelease::new("0").unwrap();
        }
        _ => {
            return Err(RegistryError::ParseError(format!(
                "Unknown bump part: {part}"
            )));
        }
    }
    Ok(ver.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConstraint {
    pub name: String,
    pub constraint: String,
    pub registry: RegistryKind,
}

impl PackageConstraint {
    pub fn parse(spec: &str) -> Result<Self, RegistryError> {
        let (name, constraint) = if let Some(idx) = spec.rfind('@') {
            let n = &spec[..idx];
            let c = &spec[idx + 1..];
            if c.is_empty() {
                (n, "*")
            } else {
                (n, c)
            }
        } else {
            (spec, "*")
        };
        let registry = RegistryKind::detect(name);
        if name.is_empty() {
            return Err(RegistryError::ParseError("Empty package name".into()));
        }
        Ok(Self {
            name: name.to_string(),
            constraint: constraint.to_string(),
            registry,
        })
    }

    pub fn to_spec(&self) -> String {
        if self.constraint == "*" {
            self.name.clone()
        } else {
            format!("{}@{}", self.name, self.constraint)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_semver_caret() {
        let versions = vec!["1.0.0".into(), "1.1.0".into(), "1.2.0".into(), "2.0.0".into()];
        assert_eq!(resolve_semver(&versions, "^1.0.0"), Some("1.2.0".into()));
    }

    #[test]
    fn test_resolve_semver_tilde() {
        let versions = vec!["1.0.0".into(), "1.0.1".into(), "1.0.2".into(), "1.1.0".into()];
        assert_eq!(resolve_semver(&versions, "~1.0.0"), Some("1.0.2".into()));
    }

    #[test]
    fn test_resolve_semver_star() {
        let versions = vec!["1.0.0".into(), "2.0.0".into(), "3.0.0".into()];
        assert_eq!(resolve_semver(&versions, "*"), Some("3.0.0".into()));
    }

    #[test]
    fn test_resolve_semver_exact() {
        let versions = vec!["1.0.0".into(), "1.1.0".into()];
        assert_eq!(resolve_semver(&versions, "1.0.0"), Some("1.0.0".into()));
    }

    #[test]
    fn test_resolve_semver_no_match() {
        let versions = vec!["1.0.0".into(), "1.1.0".into()];
        assert_eq!(resolve_semver(&versions, "^2.0.0"), None);
    }

    #[test]
    fn test_satisfies() {
        assert!(satisfies("1.2.3", "^1.0.0"));
        assert!(satisfies("1.2.3", ">=1.0.0"));
        assert!(!satisfies("2.0.0", "^1.0.0"));
        assert!(satisfies("1.0.0", "*"));
    }

    #[test]
    fn test_bump_version() {
        assert_eq!(bump_version("1.0.0", "major").unwrap(), "2.0.0");
        assert_eq!(bump_version("1.0.0", "minor").unwrap(), "1.1.0");
        assert_eq!(bump_version("1.0.0", "patch").unwrap(), "1.0.1");
    }

    #[test]
    fn test_package_constraint_parse() {
        let pc = PackageConstraint::parse("express@^4.0.0").unwrap();
        assert_eq!(pc.name, "express");
        assert_eq!(pc.constraint, "^4.0.0");
        assert_eq!(pc.registry, RegistryKind::Npm);
    }

    #[test]
    fn test_package_constraint_parse_no_version() {
        let pc = PackageConstraint::parse("lodash").unwrap();
        assert_eq!(pc.name, "lodash");
        assert_eq!(pc.constraint, "*");
    }

    #[test]
    fn test_max_satisfying() {
        let versions = vec!["1.0.0".into(), "1.5.0".into(), "2.0.0".into()];
        assert_eq!(max_satisfying(&versions, "^1.0.0"), Some(&"1.5.0".into()));
    }

    #[test]
    fn test_min_satisfying() {
        let versions = vec!["1.0.0".into(), "1.5.0".into(), "2.0.0".into()];
        assert_eq!(min_satisfying(&versions, "^1.0.0"), Some(&"1.0.0".into()));
    }

    #[test]
    fn test_sort_versions_desc() {
        let versions = vec!["1.0.0".into(), "3.0.0".into(), "2.0.0".into()];
        let sorted = sort_versions(&versions);
        assert_eq!(sorted, vec!["3.0.0", "2.0.0", "1.0.0"]);
    }

    #[test]
    fn test_find_conflicts() {
        let mut pkgs = HashMap::new();
        pkgs.insert("a".into(), vec!["1.0.0".into(), "2.0.0".into()]);
        pkgs.insert("b".into(), vec!["1.0.0".into()]);
        let conflicts = find_conflicts(&pkgs);
        assert!(conflicts.is_empty());
    }
}
