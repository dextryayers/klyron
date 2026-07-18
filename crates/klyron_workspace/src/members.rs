use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use glob::glob;
use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{CacheConfig, MemberKind, TaskResult, Workspace, WorkspaceConfig, WorkspaceMember};

impl Workspace {
    pub fn list_members(dir: &Path) -> Result<Vec<WorkspaceMember>> {
        let config = Self::detect(dir)
            .ok_or_else(|| anyhow::anyhow!("no workspace config found in {}", dir.display()))?;
        let mut members = Vec::new();
        for pattern_str in &config.members {
            let full_pattern = dir.join(pattern_str).to_string_lossy().to_string();
            if let Ok(entries) = glob(&full_pattern) {
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

    pub fn detect_member(dir: &Path) -> Option<WorkspaceMember> {
        if let Ok(content) = std::fs::read_to_string(dir.join("package.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                    let hash = Self::compute_dir_hash(dir);
                    return Some(WorkspaceMember {
                        name: name.to_string(),
                        path: dir.to_path_buf(),
                        kind: MemberKind::NodePackage,
                        hash,
                    });
                }
            }
        }
        if let Ok(content) = std::fs::read_to_string(dir.join("Cargo.toml")) {
            if let Ok(value) = toml::from_str::<toml::Value>(&content) {
                if let Some(name) = value
                    .get("package")
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str())
                {
                    let hash = Self::compute_dir_hash(dir);
                    return Some(WorkspaceMember {
                        name: name.to_string(),
                        path: dir.to_path_buf(),
                        kind: MemberKind::RustCrate,
                        hash,
                    });
                }
            }
        }
        if let Ok(content) = std::fs::read_to_string(dir.join("pyproject.toml")) {
            if let Ok(value) = toml::from_str::<toml::Value>(&content) {
                if let Some(name) = value
                    .get("project")
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str())
                {
                    let hash = Self::compute_dir_hash(dir);
                    return Some(WorkspaceMember {
                        name: name.to_string(),
                        path: dir.to_path_buf(),
                        kind: MemberKind::PythonProject,
                        hash,
                    });
                }
            }
        }
        if dir.join("setup.py").exists() {
            let name = dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "python-project".into());
            let hash = Self::compute_dir_hash(dir);
            return Some(WorkspaceMember {
                name,
                path: dir.to_path_buf(),
                kind: MemberKind::PythonProject,
                hash,
            });
        }
        if let Ok(content) = std::fs::read_to_string(dir.join("go.mod")) {
            for line in content.lines() {
                if let Some(module) = line.strip_prefix("module ") {
                    let hash = Self::compute_dir_hash(dir);
                    return Some(WorkspaceMember {
                        name: module.trim().to_string(),
                        path: dir.to_path_buf(),
                        kind: MemberKind::GoModule,
                        hash,
                    });
                }
            }
        }
        None
    }

    pub fn compute_hash(member: &WorkspaceMember) -> String {
        Self::compute_dir_hash(&member.path)
    }

    pub fn compute_dir_hash(dir: &Path) -> String {
        let mut hasher = Sha256::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut files: Vec<PathBuf> = entries
                .flatten()
                .filter(|e| e.path().is_file())
                .map(|e| e.path())
                .collect();
            files.sort();
            for file in files {
                if let Ok(content) = std::fs::read(&file) {
                    hasher.update(&content);
                }
            }
        }
        format!("{:x}", hasher.finalize())
    }

    pub fn get_affected_members(dir: &Path, base_ref: &str) -> Result<Vec<WorkspaceMember>> {
        let all_members = Self::list_members(dir)?;
        let output = Command::new("git")
            .args(["diff", "--name-only", base_ref, "HEAD"])
            .current_dir(dir)
            .output()
            .context("failed to run git diff")?;

        let changed_files: std::collections::HashSet<String> =
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|l| l.to_string())
                .collect();

        if changed_files.is_empty() {
            return Ok(all_members);
        }

        let affected = all_members
            .into_iter()
            .filter(|member| {
                changed_files.iter().any(|f| {
                    f.starts_with(
                        member
                            .path
                            .strip_prefix(dir)
                            .unwrap_or(&member.path)
                            .to_string_lossy()
                            .as_ref(),
                    )
                })
            })
            .collect();

        Ok(affected)
    }
}
