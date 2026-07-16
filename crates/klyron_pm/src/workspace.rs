use crate::PmError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_config_defaults() {
        let config = WorkspaceConfig::default();
        assert_eq!(config.members, vec!["packages/*"]);
        assert!(config.shared_deps.contains(&"typescript".to_string()));
        assert!(config.hoist_patterns.contains(&"*".to_string()));
        assert!(config.ignore_patterns.contains(&"**/node_modules/**".to_string()));
        assert!(config.features.is_none());
    }

    #[test]
    fn test_workspace_config_custom_members() {
        let config = WorkspaceConfig {
            members: vec!["packages/*".into(), "libs/*".into()],
            shared_deps: vec![],
            hoist_patterns: vec![],
            ignore_patterns: vec![],
            features: None,
        };
        assert_eq!(config.members.len(), 2);
    }

    #[test]
    fn test_hoist_analysis_empty() {
        let analysis = HoistAnalysis {
            duplicated_deps: vec![],
            potential_savings: 0,
            member_count: 0,
        };
        assert_eq!(analysis.member_count, 0);
        assert!(analysis.duplicated_deps.is_empty());
    }

    #[test]
    fn test_suggest_hoisting_empty() {
        let analysis = HoistAnalysis {
            duplicated_deps: vec![],
            potential_savings: 0,
            member_count: 0,
        };
        let suggestions = suggest_hoisting(&analysis);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_suggest_hoisting_with_duplicates() {
        let analysis = HoistAnalysis {
            duplicated_deps: vec![
                DuplicateDep {
                    name: "lodash".into(),
                    versions: vec!["4.17.21".into(), "4.17.20".into()],
                    installations: vec!["packages/a/node_modules/lodash".into()],
                    total_install_size: 1024 * 1024,
                },
            ],
            potential_savings: 1024 * 1024,
            member_count: 2,
        };
        let suggestions = suggest_hoisting(&analysis);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].package, "lodash");
    }

    #[test]
    fn test_duplicate_dep_creation() {
        let dep = DuplicateDep {
            name: "react".into(),
            versions: vec!["18.0.0".into()],
            installations: vec!["packages/app/node_modules/react".into()],
            total_install_size: 50000,
        };
        assert_eq!(dep.name, "react");
        assert_eq!(dep.total_install_size, 50000);
    }

    #[test]
    fn test_hoist_suggestion_creation() {
        let suggestion = HoistSuggestion {
            package: "lodash".into(),
            from_version: "4.17.20".into(),
            to_version: "4.17.21".into(),
            members_affected: vec!["pkg-a".into()],
            savings_bytes: 500000,
        };
        assert_eq!(suggestion.package, "lodash");
        assert_eq!(suggestion.savings_bytes, 500000);
    }

    #[test]
    fn test_add_to_member_no_file() {
        let tmp = std::env::temp_dir().join("klyron_ws_test_add");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let result = add_to_member(&tmp, "lodash", "4.17.21");
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_remove_from_member_no_file() {
        let tmp = std::env::temp_dir().join("klyron_ws_test_rm");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let result = remove_from_member(&tmp, "lodash");
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_load_workspace_config_no_config() {
        let tmp = std::env::temp_dir().join("klyron_ws_test_load");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let result = load_workspace_config(&tmp);
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_load_workspace_config_from_package_json() {
        let tmp = std::env::temp_dir().join("klyron_ws_test_pkg");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let pkg = serde_json::json!({
            "workspaces": ["packages/*", "libs/*"]
        });
        std::fs::write(tmp.join("package.json"), serde_json::to_string(&pkg).unwrap()).unwrap();
        let config = load_workspace_config(&tmp).unwrap();
        assert_eq!(config.members.len(), 2);
        assert!(config.members.contains(&"packages/*".to_string()));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_workspace_config_with_features() {
        let mut features = HashMap::new();
        features.insert("web".into(), vec!["react".into(), "react-dom".into()]);
        features.insert("mobile".into(), vec!["react-native".into()]);
        let config = WorkspaceConfig {
            members: vec!["packages/*".into()],
            shared_deps: vec![],
            hoist_patterns: vec![],
            ignore_patterns: vec![],
            features: Some(features),
        };
        assert!(config.features.is_some());
        assert_eq!(config.features.as_ref().unwrap().len(), 2);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub members: Vec<String>,
    pub shared_deps: Vec<String>,
    pub hoist_patterns: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub features: Option<HashMap<String, Vec<String>>>,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            members: vec!["packages/*".into()],
            shared_deps: vec!["typescript".into(), "eslint".into(), "prettier".into()],
            hoist_patterns: vec!["*".into()],
            ignore_patterns: vec!["**/node_modules/**".into()],
            features: None,
        }
    }
}

pub fn load_workspace_config(root_dir: &Path) -> Result<WorkspaceConfig, PmError> {
    let package_json = root_dir.join("package.json");
    if package_json.exists() {
        let content = std::fs::read_to_string(&package_json)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        if let Some(workspaces) = json.get("workspaces") {
            if let Some(arr) = workspaces.as_array() {
                let members: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                return Ok(WorkspaceConfig {
                    members,
                    ..Default::default()
                });
            }
            if let Some(obj) = workspaces.as_object() {
                let members = obj
                    .get("packages")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                return Ok(WorkspaceConfig {
                    members,
                    shared_deps: obj
                        .get("nohoist")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default(),
                    ..Default::default()
                });
            }
        }
    }
    let pnpm_workspace = root_dir.join("pnpm-workspace.yaml");
    if pnpm_workspace.exists() {
        let content = std::fs::read_to_string(&pnpm_workspace)
            .map_err(|e| PmError::WorkspaceError(format!("Cannot read pnpm-workspace.yaml: {e}")))?;
        let config: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| PmError::WorkspaceError(format!("YAML parse error: {e}")))?;
        let members = config
            .get("packages")
            .and_then(|v| v.as_sequence())
            .map(|seq| seq.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        return Ok(WorkspaceConfig {
            members,
            ..Default::default()
        });
    }
    Err(PmError::WorkspaceError("No workspace configuration found".into()))
}

pub fn list_members(root_dir: &Path) -> Result<Vec<String>, PmError> {
    let config = load_workspace_config(root_dir)?;
    let mut members = Vec::new();
    for pattern in &config.members {
        let glob_pattern = root_dir.join(pattern).to_string_lossy().to_string();
        let entries = glob::glob(&glob_pattern)
            .map_err(|e| PmError::WorkspaceError(format!("Glob error: {e}")))?;
        for entry in entries.flatten() {
            if entry.is_dir() && entry.join("package.json").exists() {
                if let Some(name) = entry.file_name().and_then(|n| n.to_str()) {
                    members.push(name.to_string());
                }
            }
        }
    }
    members.sort();
    members.dedup();
    Ok(members)
}

pub fn add_to_member(member_dir: &Path, dep: &str, version: &str) -> Result<(), PmError> {
    let pkg_path = member_dir.join("package.json");
    if !pkg_path.exists() {
        return Err(PmError::WorkspaceError(format!("Member has no package.json: {}", member_dir.display())));
    }
    let content = std::fs::read_to_string(&pkg_path)?;
    let mut pkg: serde_json::Value = serde_json::from_str(&content)?;
    if let Some(deps) = pkg.get_mut("dependencies").and_then(|d| d.as_object_mut()) {
        deps.insert(dep.to_string(), serde_json::Value::String(version.to_string()));
    } else {
        let mut deps = serde_json::Map::new();
        deps.insert(dep.to_string(), serde_json::Value::String(version.to_string()));
        pkg.as_object_mut()
            .ok_or_else(|| PmError::WorkspaceError("Invalid package.json".into()))?
            .insert("dependencies".into(), serde_json::Value::Object(deps));
    }
    std::fs::write(&pkg_path, serde_json::to_string_pretty(&pkg)?)?;
    Ok(())
}

pub fn remove_from_member(member_dir: &Path, dep: &str) -> Result<(), PmError> {
    let pkg_path = member_dir.join("package.json");
    if !pkg_path.exists() {
        return Err(PmError::WorkspaceError(format!("Member has no package.json: {}", member_dir.display())));
    }
    let content = std::fs::read_to_string(&pkg_path)?;
    let mut pkg: serde_json::Value = serde_json::from_str(&content)?;
    if let Some(deps) = pkg.get_mut("dependencies").and_then(|d| d.as_object_mut()) {
        deps.remove(dep);
    }
    if let Some(deps) = pkg.get_mut("devDependencies").and_then(|d| d.as_object_mut()) {
        deps.remove(dep);
    }
    std::fs::write(&pkg_path, serde_json::to_string_pretty(&pkg)?)?;
    Ok(())
}

pub fn run_in_members(root_dir: &Path, script: &str) -> HashMap<String, Result<(), PmError>> {
    let mut results = HashMap::new();
    let members = match list_members(root_dir) {
        Ok(m) => m,
        Err(e) => {
            results.insert("root".into(), Err(PmError::WorkspaceError(format!("Cannot list members: {e}"))));
            return results;
        }
    };
    for member in members {
        let member_dir = root_dir.join("packages").join(&member);
        let result = match std::process::Command::new("sh")
            .arg("-c")
            .arg(script)
            .current_dir(&member_dir)
            .output()
        {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => Err(PmError::WorkspaceError(format!(
                "Script failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))),
            Err(e) => Err(PmError::WorkspaceError(format!("Script error: {e}"))),
        };
        results.insert(member, result);
    }
    results
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoistAnalysis {
    pub duplicated_deps: Vec<DuplicateDep>,
    pub potential_savings: u64,
    pub member_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateDep {
    pub name: String,
    pub versions: Vec<String>,
    pub installations: Vec<String>,
    pub total_install_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoistSuggestion {
    pub package: String,
    pub from_version: String,
    pub to_version: String,
    pub members_affected: Vec<String>,
    pub savings_bytes: u64,
}

pub fn analyze_hoisting(root_dir: &Path) -> Result<HoistAnalysis, PmError> {
    let members = list_members(root_dir)?;
    let mut all_deps: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for member in &members {
        let member_dir = root_dir.join("packages").join(member);
        let pkg_path = member_dir.join("package.json");
        if !pkg_path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&pkg_path)?;
        if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
            for deps_key in &["dependencies", "devDependencies"] {
                if let Some(deps) = pkg.get(*deps_key).and_then(|d| d.as_object()) {
                    for (name, ver) in deps {
                        let version = ver.as_str().unwrap_or("*").to_string();
                        all_deps.entry(name.clone()).or_default().push((member.clone(), version));
                    }
                }
            }
        }
    }
    let mut duplicated = Vec::new();
    let mut savings: u64 = 0;
    for (name, deps) in &all_deps {
        if deps.len() > 1 {
            let versions: std::collections::BTreeSet<String> = deps.iter().map(|(_, v)| v.clone()).collect();
            if versions.len() > 1 {
                let installations: Vec<String> = deps.iter().map(|(m, _)| format!("packages/{m}/node_modules/{name}")).collect();
                duplicated.push(DuplicateDep {
                    name: name.clone(),
                    versions: versions.into_iter().collect(),
                    installations,
                    total_install_size: deps.len() as u64 * 1024 * 1024,
                });
                savings += (deps.len() - 1) as u64 * 1024 * 1024;
            }
        }
    }
    Ok(HoistAnalysis {
        duplicated_deps: duplicated,
        potential_savings: savings,
        member_count: members.len(),
    })
}

pub fn suggest_hoisting(analysis: &HoistAnalysis) -> Vec<HoistSuggestion> {
    let mut suggestions = Vec::new();
    for dup in &analysis.duplicated_deps {
        let target_version = dup.versions.first().cloned().unwrap_or_default();
        suggestions.push(HoistSuggestion {
            package: dup.name.clone(),
            from_version: dup.versions.last().cloned().unwrap_or_default(),
            to_version: target_version,
            members_affected: dup.installations.clone(),
            savings_bytes: analysis.potential_savings / analysis.duplicated_deps.len().max(1) as u64,
        });
    }
    suggestions
}
