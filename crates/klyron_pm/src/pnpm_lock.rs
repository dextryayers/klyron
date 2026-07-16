use crate::{LockfileV3, LockfilePackage, PmError};
use std::collections::{BTreeMap, HashMap};

/// Parse a pnpm-lock.yaml v5/v6 file into our LockfileV3 format
pub fn parse_pnpm_lock(content: &str) -> Result<LockfileV3, PmError> {
    let mut packages = BTreeMap::new();
    let mut lockfile_version = None;

    let mut current_package: Option<String> = None;
    let mut current_version = String::new();
    let mut current_resolved = None;
    let mut current_integrity = None;
    let mut current_deps: Option<HashMap<String, String>> = None;
    let mut in_deps = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("lockfileVersion:") {
            lockfile_version = Some(trimmed.split(':').nth(1).unwrap_or("").trim().to_string());
            continue;
        }

        if trimmed.starts_with('/') && trimmed.ends_with(':') {
            if let Some(ref name) = current_package {
                if !current_version.is_empty() {
                    let node_path = format!("node_modules/{}", name);
                    let pkg = LockfilePackage {
                        version: current_version.clone(),
                        resolved: current_resolved.take(),
                        integrity: current_integrity.take(),
                        dependencies: current_deps.take(),
                        optional_dependencies: None,
                        peer_dependencies: None,
                        dev: None,
                        optional: None,
                        bundled: None,
                        engines: None,
                        os: None,
                        cpu: None,
                        has_dev_dependencies: None,
                    };
                    packages.insert(node_path, pkg);
                }
            }

            let spec = trimmed.trim_start_matches('/').trim_end_matches(':');
            let parts: Vec<&str> = spec.rsplitn(2, '@').collect();
            let name = if parts.len() >= 2 { parts[1] } else { spec };
            current_package = Some(name.to_string());
            current_version = parts[0].to_string();
            current_resolved = None;
            current_integrity = None;
            current_deps = None;
            in_deps = false;
            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if current_package.is_some() {
            let indent = line.len() - line.trim_start().len();

            if indent >= 4 && trimmed.starts_with("version:") {
                let val = trimmed.split(':').nth(1).unwrap_or("").trim().trim_matches('\'');
                if !val.is_empty() {
                    current_version = val.to_string();
                }
            } else if indent >= 4 && trimmed.starts_with("resolution:") {
            } else if indent >= 4 && trimmed.starts_with("integrity:") {
                current_integrity = Some(trimmed.split(':').nth(1).unwrap_or("").trim().trim_matches('\'').to_string());
            } else if indent >= 4 && trimmed.starts_with("dependencies:") {
                in_deps = true;
                current_deps = Some(HashMap::new());
            } else if indent >= 6 && in_deps {
                if let Some(deps) = &mut current_deps {
                    let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                    if parts.len() >= 2 {
                        let dep_name = parts[0].trim().trim_matches('\'');
                        let dep_ver = parts[1].trim().trim_matches('\'').trim_matches('"');
                        deps.insert(dep_name.to_string(), dep_ver.to_string());
                    }
                }
            } else if indent < 6 {
                in_deps = false;
            }
        }
    }

    if let Some(ref name) = current_package {
        if !current_version.is_empty() {
            let node_path = format!("node_modules/{}", name);
            let pkg = LockfilePackage {
                version: current_version.clone(),
                resolved: current_resolved,
                integrity: current_integrity,
                dependencies: current_deps,
                optional_dependencies: None,
                peer_dependencies: None,
                dev: None,
                optional: None,
                bundled: None,
                engines: None,
                os: None,
                cpu: None,
                has_dev_dependencies: None,
            };
            packages.insert(node_path, pkg);
        }
    }

    Ok(LockfileV3 {
        name: None,
        lockfile_version: lockfile_version.map(|v| format!("pnpm-v{}", v)),
        packages,
        workspaces: None,
        metadata: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pnpm_lock_basic() {
        let content = r#"lockfileVersion: '6.0'

settings:
  autoInstallPeers: true
  excludeLinksFromLockfile: false

importers:
  .:
    dependencies:
      express:
        specifier: ^4.18.0
        version: 4.18.2

packages:
  /express@4.18.2:
    resolution: {integrity: sha512-5/PsL6iGPdfQ/lKM1UuielYgv3BUoJfz1aUwU9vHZ+J7gyvwdQXFEBIEIaxeGf0GIcreATNyBExtalisDbuMqQ==}
    engines: {node: '>= 0.10.0'}
    dependencies:
      accepts: 1.3.8
      array-flatten: 1.1.1
    dev: false

  /accepts@1.3.8:
    resolution: {integrity: sha512-PYAthTa2m2VKxuvSD3DPC/Gy+U+sOA1LAuT8mkmRuvw+NACSaeXEQ+NHcVF7rONl6qcaxV3Uuemwawk+7+SJLw==}
    engines: {node: '>= 0.6'}
    dependencies:
      mime-types: 2.1.35
      negotiator: 0.6.3
"#;
        let lock = parse_pnpm_lock(content).unwrap();
        assert!(lock.packages.len() >= 2);
        assert!(lock.packages.contains_key("node_modules/express"));
        assert_eq!(lock.packages["node_modules/express"].version, "4.18.2");
    }

    #[test]
    fn test_parse_pnpm_lock_empty() {
        let content = "lockfileVersion: '6.0'\npackages: {}\n";
        let lock = parse_pnpm_lock(content).unwrap();
        assert_eq!(lock.packages.len(), 0);
    }

    #[test]
    fn test_pnpm_dependencies() {
        let content = r#"lockfileVersion: '6.0'

packages:
  /root@1.0.0:
    dependencies:
      child: 0.5.0
"#;
        let lock = parse_pnpm_lock(content).unwrap();
        let root = &lock.packages["node_modules/root"];
        let deps = root.dependencies.as_ref().unwrap();
        assert_eq!(deps.get("child").unwrap(), "0.5.0");
    }
}
