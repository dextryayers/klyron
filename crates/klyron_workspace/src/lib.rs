use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub members: Vec<String>,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub shared_deps: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    pub name: String,
    pub path: PathBuf,
    pub kind: MemberKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberKind {
    NodePackage,
    RustCrate,
    PythonProject,
    GoModule,
}

pub struct Workspace;

impl Workspace {
    pub fn new() -> Self {
        Self
    }

    pub fn init(dir: &Path, name: &str) -> Result<WorkspaceConfig> {
        let config = WorkspaceConfig {
            members: vec!["packages/*".into()],
            name: name.to_string(),
            version: "0.1.0".into(),
            shared_deps: HashMap::new(),
        };
        let config_path = dir.join("klyron.toml");
        let toml_str =
            toml::to_string_pretty(&config).context("failed to serialize workspace config")?;
        std::fs::write(&config_path, toml_str).context("failed to write klyron.toml")?;
        Ok(config)
    }

    pub fn detect(dir: &Path) -> Option<WorkspaceConfig> {
        if let Some(cfg) = Self::detect_klyron_toml(dir) {
            return Some(cfg);
        }
        if let Some(cfg) = Self::detect_package_json(dir) {
            return Some(cfg);
        }
        if let Some(cfg) = Self::detect_cargo_workspace(dir) {
            return Some(cfg);
        }
        None
    }

    fn detect_klyron_toml(dir: &Path) -> Option<WorkspaceConfig> {
        let path = dir.join("klyron.toml");
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(path).ok()?;
        toml::from_str::<WorkspaceConfig>(&content).ok()
    }

    fn detect_package_json(dir: &Path) -> Option<WorkspaceConfig> {
        let path = dir.join("package.json");
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        let workspaces = json
            .get("workspaces")
            .or_else(|| json.get("workspaces").and_then(|w| w.get("packages")))?;
        let members: Vec<String> = workspaces
            .as_array()?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        if members.is_empty() {
            return None;
        }
        let name = json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("workspace")
            .to_string();
        let version = json
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.1.0")
            .to_string();
        Some(WorkspaceConfig {
            members,
            name,
            version,
            shared_deps: HashMap::new(),
        })
    }

    fn detect_cargo_workspace(dir: &Path) -> Option<WorkspaceConfig> {
        let path = dir.join("Cargo.toml");
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(path).ok()?;
        let value: toml::Value = toml::from_str(&content).ok()?;
        let workspace = value.get("workspace")?;
        let members_arr = workspace.get("members")?.as_array()?;
        let members: Vec<String> = members_arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        if members.is_empty() {
            return None;
        }
        Some(WorkspaceConfig {
            members,
            name: "workspace".into(),
            version: "0.1.0".into(),
            shared_deps: HashMap::new(),
        })
    }

    pub fn list_members(dir: &Path) -> Result<Vec<WorkspaceMember>> {
        let config = Self::detect(dir)
            .ok_or_else(|| anyhow::anyhow!("no workspace config found in {}", dir.display()))?;
        let mut members = Vec::new();
        for pattern_str in &config.members {
            let full_pattern = dir.join(pattern_str).to_string_lossy().to_string();
            if let Ok(entries) = glob::glob(&full_pattern) {
                for entry in entries.flatten() {
                    if entry.is_dir() {
                        if let Some(member) = Self::detect_member(&entry) {
                            members.push(member);
                        }
                    }
                }
            }
        }
        members.sort_by(|a, b| a.name.cmp(&b.name));
        members.dedup_by(|a, b| a.name == b.name && a.path == b.path);
        Ok(members)
    }

    fn detect_member(dir: &Path) -> Option<WorkspaceMember> {
        let pkg_path = dir.join("package.json");
        if pkg_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&pkg_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                        return Some(WorkspaceMember {
                            name: name.to_string(),
                            path: dir.to_path_buf(),
                            kind: MemberKind::NodePackage,
                        });
                    }
                }
            }
        }
        let cargo_path = dir.join("Cargo.toml");
        if cargo_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_path) {
                if let Ok(value) = toml::from_str::<toml::Value>(&content) {
                    if let Some(package) = value.get("package") {
                        if let Some(name) = package.get("name").and_then(|v| v.as_str()) {
                            return Some(WorkspaceMember {
                                name: name.to_string(),
                                path: dir.to_path_buf(),
                                kind: MemberKind::RustCrate,
                            });
                        }
                    }
                }
            }
        }
        let pyproject_path = dir.join("pyproject.toml");
        if pyproject_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&pyproject_path) {
                if let Ok(value) = toml::from_str::<toml::Value>(&content) {
                    if let Some(proj) = value.get("project") {
                        if let Some(name) = proj.get("name").and_then(|v| v.as_str()) {
                            return Some(WorkspaceMember {
                                name: name.to_string(),
                                path: dir.to_path_buf(),
                                kind: MemberKind::PythonProject,
                            });
                        }
                    }
                }
            }
        }
        let setup_path = dir.join("setup.py");
        if setup_path.exists() {
            let name = dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "python-project".into());
            return Some(WorkspaceMember {
                name,
                path: dir.to_path_buf(),
                kind: MemberKind::PythonProject,
            });
        }
        let go_path = dir.join("go.mod");
        if go_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&go_path) {
                for line in content.lines() {
                    if let Some(module) = line.strip_prefix("module ") {
                        return Some(WorkspaceMember {
                            name: module.trim().to_string(),
                            path: dir.to_path_buf(),
                            kind: MemberKind::GoModule,
                        });
                    }
                }
            }
        }
        None
    }

    pub fn add_member(dir: &Path, name: &str) -> Result<()> {
        let member_path = dir.join(name);
        if member_path.exists() {
            anyhow::bail!(
                "member '{}' already exists at {}",
                name,
                member_path.display()
            );
        }
        std::fs::create_dir_all(&member_path)
            .with_context(|| format!("failed to create member directory: {}", member_path.display()))?;
        let pkg_json = serde_json::json!({
            "name": name,
            "version": "0.1.0",
            "private": true,
        });
        let pkg_path = member_path.join("package.json");
        std::fs::write(&pkg_path, serde_json::to_string_pretty(&pkg_json)?)
            .context("failed to write package.json")?;
        let config_path = dir.join("klyron.toml");
        let mut config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            toml::from_str::<WorkspaceConfig>(&content)?
        } else {
            WorkspaceConfig {
                members: vec![],
                name: "workspace".into(),
                version: "0.1.0".into(),
                shared_deps: HashMap::new(),
            }
        };
        if !config.members.contains(&name.to_string()) {
            config.members.push(name.to_string());
        }
        let toml_str =
            toml::to_string_pretty(&config).context("failed to serialize updated config")?;
        std::fs::write(&config_path, toml_str)?;
        Ok(())
    }

    pub fn remove_member(dir: &Path, name: &str) -> Result<()> {
        let member_path = dir.join(name);
        if member_path.exists() {
            std::fs::remove_dir_all(&member_path).with_context(|| {
                format!("failed to remove member directory: {}", member_path.display())
            })?;
        }
        let config_path = dir.join("klyron.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let mut config = toml::from_str::<WorkspaceConfig>(&content)?;
            config.members.retain(|m| m != name);
            let toml_str =
                toml::to_string_pretty(&config).context("failed to serialize updated config")?;
            std::fs::write(&config_path, toml_str)?;
        }
        Ok(())
    }

    pub fn run_script(dir: &Path, script: &str) -> Result<()> {
        let members = Self::list_members(dir)?;
        if members.is_empty() {
            anyhow::bail!("no workspace members found");
        }
        for member in &members {
            eprintln!("[{}] running '{}'...", member.name, script);
            let status = Command::new("sh")
                .args(["-c", script])
                .current_dir(&member.path)
                .status()
                .with_context(|| format!("failed to run script in {}", member.name))?;
            if !status.success() {
                anyhow::bail!(
                    "script '{}' failed in {} with exit code {:?}",
                    script,
                    member.name,
                    status.code()
                );
            }
        }
        Ok(())
    }

    pub fn exec_in_members(dir: &Path, cmd: &str, args: &[&str]) -> Result<()> {
        let members = Self::list_members(dir)?;
        if members.is_empty() {
            anyhow::bail!("no workspace members found");
        }
        for member in &members {
            eprintln!(
                "[{}] executing '{} {}'...",
                member.name,
                cmd,
                args.join(" ")
            );
            let status = Command::new(cmd)
                .args(args)
                .current_dir(&member.path)
                .status()
                .with_context(|| format!("failed to execute in {}", member.name))?;
            if !status.success() {
                anyhow::bail!(
                    "command '{} {}' failed in {} with exit code {:?}",
                    cmd,
                    args.join(" "),
                    member.name,
                    status.code()
                );
            }
        }
        Ok(())
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("klyron_test_ws_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_workspace_new() {
        let _ws = Workspace::new();
    }

    #[test]
    fn test_init_creates_config() {
        let dir = temp_dir("init");
        let config = Workspace::init(&dir, "my-workspace").unwrap();
        assert_eq!(config.name, "my-workspace");
        assert_eq!(config.version, "0.1.0");
        assert_eq!(config.members, vec!["packages/*"]);
        assert!(dir.join("klyron.toml").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_klyron_toml() {
        let dir = temp_dir("detect_klyron");
        let config = WorkspaceConfig {
            members: vec!["crates/*".into()],
            name: "test-ws".into(),
            version: "0.1.0".into(),
            shared_deps: HashMap::new(),
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        fs::write(dir.join("klyron.toml"), toml_str).unwrap();
        let detected = Workspace::detect(&dir).unwrap();
        assert_eq!(detected.name, "test-ws");
        assert_eq!(detected.members, vec!["crates/*"]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_package_json_workspaces() {
        let dir = temp_dir("detect_pkg");
        let pkg = serde_json::json!({
            "name": "monorepo",
            "version": "1.0.0",
            "workspaces": ["packages/*", "apps/*"]
        });
        fs::write(
            dir.join("package.json"),
            serde_json::to_string_pretty(&pkg).unwrap(),
        )
        .unwrap();
        let detected = Workspace::detect(&dir).unwrap();
        assert_eq!(detected.name, "monorepo");
        assert!(detected.members.contains(&"packages/*".into()));
        assert!(detected.members.contains(&"apps/*".into()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_cargo_workspace() {
        let dir = temp_dir("detect_cargo");
        let cargo = r#"[workspace]
members = ["crates/*", "libs/*"]

[package]
name = "root"
version = "0.1.0"
"#;
        fs::write(dir.join("Cargo.toml"), cargo).unwrap();
        let detected = Workspace::detect(&dir).unwrap();
        assert!(detected.members.contains(&"crates/*".into()));
        assert!(detected.members.contains(&"libs/*".into()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_no_config() {
        let dir = temp_dir("detect_none");
        assert!(Workspace::detect(&dir).is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_add_member() {
        let dir = temp_dir("add_member");
        Workspace::init(&dir, "test-ws").unwrap();
        Workspace::add_member(&dir, "my-pkg").unwrap();
        let members = Workspace::list_members(&dir).unwrap();
        assert!(members.iter().any(|m| m.name == "my-pkg"));
        assert_eq!(
            members.iter().find(|m| m.name == "my-pkg").unwrap().kind,
            MemberKind::NodePackage
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_remove_member() {
        let dir = temp_dir("remove_member");
        Workspace::init(&dir, "test-ws").unwrap();
        Workspace::add_member(&dir, "my-pkg").unwrap();
        assert!(dir.join("my-pkg").exists());
        Workspace::remove_member(&dir, "my-pkg").unwrap();
        assert!(!dir.join("my-pkg").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_member_node() {
        let dir = temp_dir("detect_member_node");
        let pkg = serde_json::json!({"name": "my-pkg", "version": "1.0.0"});
        fs::write(
            dir.join("package.json"),
            serde_json::to_string_pretty(&pkg).unwrap(),
        )
        .unwrap();
        let member = Workspace::detect_member(&dir).unwrap();
        assert_eq!(member.name, "my-pkg");
        assert_eq!(member.kind, MemberKind::NodePackage);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_member_rust() {
        let dir = temp_dir("detect_member_rust");
        let cargo_toml = r#"[package]
name = "my-crate"
version = "0.1.0"
edition = "2021"
"#;
        fs::write(dir.join("Cargo.toml"), cargo_toml).unwrap();
        let member = Workspace::detect_member(&dir).unwrap();
        assert_eq!(member.name, "my-crate");
        assert_eq!(member.kind, MemberKind::RustCrate);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_member_python_pyproject() {
        let dir = temp_dir("detect_member_python");
        let pyproject = r#"[project]
name = "my-py-lib"
version = "0.1.0"
"#;
        fs::write(dir.join("pyproject.toml"), pyproject).unwrap();
        let member = Workspace::detect_member(&dir).unwrap();
        assert_eq!(member.name, "my-py-lib");
        assert_eq!(member.kind, MemberKind::PythonProject);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_member_go() {
        let dir = temp_dir("detect_member_go");
        fs::write(dir.join("go.mod"), "module github.com/user/my-module\n\ngo 1.21\n").unwrap();
        let member = Workspace::detect_member(&dir).unwrap();
        assert_eq!(member.name, "github.com/user/my-module");
        assert_eq!(member.kind, MemberKind::GoModule);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_members_empty() {
        let dir = temp_dir("list_empty");
        Workspace::init(&dir, "empty-ws").unwrap();
        let members = Workspace::list_members(&dir).unwrap();
        assert!(members.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_workspace_round_trip_serialize() {
        let config = WorkspaceConfig {
            members: vec!["packages/*".into()],
            name: "test".into(),
            version: "0.1.0".into(),
            shared_deps: {
                let mut m = HashMap::new();
                m.insert("react".into(), "^18".into());
                m
            },
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: WorkspaceConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.name, "test");
        assert_eq!(deserialized.members, vec!["packages/*"]);
        assert_eq!(deserialized.shared_deps.get("react").unwrap(), "^18");
    }
}
