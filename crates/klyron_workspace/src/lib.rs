use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::Instant;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use glob::glob;
use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
  pub members: Vec<String>,
  pub name: String,
  pub version: String,
  #[serde(default)]
  pub shared_deps: HashMap<String, String>,
  #[serde(default)]
  pub pipeline: HashMap<String, PipelineStep>,
  #[serde(default)]
  pub cache_config: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
  pub depends_on: Vec<String>,
  pub command: String,
  pub output_globs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
  pub enabled: bool,
  pub remote_url: Option<String>,
  pub remote_token: Option<String>,
}

impl Default for CacheConfig {
  fn default() -> Self {
    CacheConfig {
      enabled: true,
      remote_url: None,
      remote_token: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct WorkspaceMember {
  pub name: String,
  pub path: PathBuf,
  pub kind: MemberKind,
  pub hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberKind {
  NodePackage,
  RustCrate,
  PythonProject,
  GoModule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
  pub hash: String,
  pub timestamp: DateTime<Utc>,
  pub outputs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
  pub member: String,
  pub success: bool,
  pub duration: f64,
  pub output: String,
  pub cached: bool,
}

pub struct Workspace {
  cache: Mutex<HashMap<String, CacheEntry>>,
  #[allow(dead_code)]
  remote_cache: Option<String>,
}

impl Workspace {
  pub fn new() -> Self {
    Workspace {
      cache: Mutex::new(HashMap::new()),
      remote_cache: None,
    }
  }

  pub fn init(dir: &Path, name: &str) -> Result<WorkspaceConfig> {
    let config = WorkspaceConfig {
      members: vec!["packages/*".into()],
      name: name.to_string(),
      version: "0.1.0".into(),
      shared_deps: HashMap::new(),
      pipeline: HashMap::new(),
      cache_config: CacheConfig::default(),
    };
    let config_path = dir.join("klyron.toml");
    let toml_str = toml::to_string_pretty(&config).context("failed to serialize workspace config")?;
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
    if !path.exists() { return None; }
    let content = std::fs::read_to_string(path).ok()?;
    toml::from_str::<WorkspaceConfig>(&content).ok()
  }

  fn detect_package_json(dir: &Path) -> Option<WorkspaceConfig> {
    let path = dir.join("package.json");
    if !path.exists() { return None; }
    let content = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let workspaces = json.get("workspaces")
      .or_else(|| json.get("workspaces").and_then(|w| w.get("packages")))?;
    let members: Vec<String> = workspaces.as_array()?
      .iter().filter_map(|v| v.as_str().map(String::from)).collect();
    if members.is_empty() { return None; }
    let name = json.get("name").and_then(|v| v.as_str()).unwrap_or("workspace").to_string();
    let version = json.get("version").and_then(|v| v.as_str()).unwrap_or("0.1.0").to_string();
    Some(WorkspaceConfig {
      members, name, version, shared_deps: HashMap::new(), pipeline: HashMap::new(),
      cache_config: CacheConfig::default(),
    })
  }

  fn detect_cargo_workspace(dir: &Path) -> Option<WorkspaceConfig> {
    let path = dir.join("Cargo.toml");
    if !path.exists() { return None; }
    let content = std::fs::read_to_string(path).ok()?;
    let value: toml::Value = toml::from_str(&content).ok()?;
    let workspace = value.get("workspace")?;
    let members_arr = workspace.get("members")?.as_array()?;
    let members: Vec<String> = members_arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
    if members.is_empty() { return None; }
    Some(WorkspaceConfig {
      members, name: "workspace".into(), version: "0.1.0".into(),
      shared_deps: HashMap::new(), pipeline: HashMap::new(), cache_config: CacheConfig::default(),
    })
  }

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

  fn detect_member(dir: &Path) -> Option<WorkspaceMember> {
    if let Ok(content) = std::fs::read_to_string(dir.join("package.json")) {
      if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
          let hash = Self::compute_dir_hash(dir);
          return Some(WorkspaceMember { name: name.to_string(), path: dir.to_path_buf(), kind: MemberKind::NodePackage, hash });
        }
      }
    }
    if let Ok(content) = std::fs::read_to_string(dir.join("Cargo.toml")) {
      if let Ok(value) = toml::from_str::<toml::Value>(&content) {
        if let Some(name) = value.get("package").and_then(|p| p.get("name")).and_then(|v| v.as_str()) {
          let hash = Self::compute_dir_hash(dir);
          return Some(WorkspaceMember { name: name.to_string(), path: dir.to_path_buf(), kind: MemberKind::RustCrate, hash });
        }
      }
    }
    if let Ok(content) = std::fs::read_to_string(dir.join("pyproject.toml")) {
      if let Ok(value) = toml::from_str::<toml::Value>(&content) {
        if let Some(name) = value.get("project").and_then(|p| p.get("name")).and_then(|v| v.as_str()) {
          let hash = Self::compute_dir_hash(dir);
          return Some(WorkspaceMember { name: name.to_string(), path: dir.to_path_buf(), kind: MemberKind::PythonProject, hash });
        }
      }
    }
    if dir.join("setup.py").exists() {
      let name = dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "python-project".into());
      let hash = Self::compute_dir_hash(dir);
      return Some(WorkspaceMember { name, path: dir.to_path_buf(), kind: MemberKind::PythonProject, hash });
    }
    if let Ok(content) = std::fs::read_to_string(dir.join("go.mod")) {
      for line in content.lines() {
        if let Some(module) = line.strip_prefix("module ") {
          let hash = Self::compute_dir_hash(dir);
          return Some(WorkspaceMember { name: module.trim().to_string(), path: dir.to_path_buf(), kind: MemberKind::GoModule, hash });
        }
      }
    }
    None
  }

  pub fn compute_hash(member: &WorkspaceMember) -> String {
    Self::compute_dir_hash(&member.path)
  }

  fn compute_dir_hash(dir: &Path) -> String {
    let mut hasher = Sha256::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
      let mut files: Vec<PathBuf> = entries.flatten().filter(|e| e.path().is_file()).map(|e| e.path()).collect();
      files.sort();
      for file in files {
        if let Ok(content) = std::fs::read(&file) {
          hasher.update(&content);
        }
      }
    }
    format!("{:x}", hasher.finalize())
  }

  pub fn build_dependency_graph(dir: &Path) -> Result<DiGraph<String, ()>> {
    let members = Self::list_members(dir)?;
    let mut graph = DiGraph::<String, ()>::new();
    let mut indices = HashMap::new();

    for member in &members {
      let idx = graph.add_node(member.name.clone());
      indices.insert(member.name.clone(), idx);
    }

    for member in &members {
      let member_dir = &member.path;
      if let Ok(content) = std::fs::read_to_string(member_dir.join("package.json")) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
          if let Some(deps) = json.get("dependencies").and_then(|v| v.as_object()) {
            for dep_name in deps.keys() {
              if let Some(&dep_idx) = indices.get(dep_name) {
                if let Some(&member_idx) = indices.get(&member.name) {
                  graph.add_edge(member_idx, dep_idx, ());
                }
              }
            }
          }
        }
      }
    }

    Ok(graph)
  }

  pub fn render_dependency_graph(dir: &Path) -> Result<String> {
    let graph = Self::build_dependency_graph(dir)?;
    let mut dot = String::from("digraph workspace {\n  rankdir=LR;\n  node [shape=box, style=rounded];\n");

    for node in graph.node_indices() {
      dot.push_str(&format!("  \"{}\";\n", graph[node]));
    }
    for edge in graph.edge_references() {
      dot.push_str(&format!(
        "  \"{}\" -> \"{}\";\n",
        graph[edge.source()],
        graph[edge.target()]
      ));
    }
    dot.push_str("}\n");
    Ok(dot)
  }

  pub fn run_task_parallel(dir: &Path, command: &str, args: &[&str]) -> Result<Vec<TaskResult>> {
    let members = Self::list_members(dir)?;
    if members.is_empty() {
      anyhow::bail!("no workspace members found");
    }

    let results: Vec<TaskResult> = members.par_iter().map(|member| {
      let start = Instant::now();
      let output = Command::new(command)
        .args(args)
        .current_dir(&member.path)
        .output();
      let duration = start.elapsed().as_secs_f64();

      match output {
        Ok(out) => {
          let stdout = String::from_utf8_lossy(&out.stdout).to_string();
          let stderr = String::from_utf8_lossy(&out.stderr).to_string();
          TaskResult {
            member: member.name.clone(),
            success: out.status.success(),
            duration,
            output: format!("{stdout}\n{stderr}"),
            cached: false,
          }
        }
        Err(e) => TaskResult {
          member: member.name.clone(),
          success: false,
          duration,
          output: format!("{e:#}"),
          cached: false,
        },
      }
    }).collect();

    Ok(results)
  }

  pub fn run_affected(dir: &Path, base_ref: &str, command: &str, args: &[&str]) -> Result<Vec<TaskResult>> {
    let affected = Self::get_affected_members(dir, base_ref)?;
    if affected.is_empty() {
      return Ok(Vec::new());
    }

    let results: Vec<TaskResult> = affected.par_iter().map(|member| {
      let start = Instant::now();
      let output = Command::new(command)
        .args(args)
        .current_dir(&member.path)
        .output();
      let duration = start.elapsed().as_secs_f64();

      match output {
        Ok(out) => {
          let stdout = String::from_utf8_lossy(&out.stdout).to_string();
          let stderr = String::from_utf8_lossy(&out.stderr).to_string();
          TaskResult {
            member: member.name.clone(),
            success: out.status.success(),
            duration,
            output: format!("{stdout}\n{stderr}"),
            cached: false,
          }
        }
        Err(e) => TaskResult {
          member: member.name.clone(),
          success: false,
          duration,
          output: format!("{e:#}"),
          cached: false,
        },
      }
    }).collect();

    Ok(results)
  }

  pub fn get_affected_members(dir: &Path, base_ref: &str) -> Result<Vec<WorkspaceMember>> {
    let all_members = Self::list_members(dir)?;
    let output = Command::new("git")
      .args(["diff", "--name-only", base_ref, "HEAD"])
      .current_dir(dir)
      .output()
      .context("failed to run git diff")?;

    let changed_files: HashSet<String> = String::from_utf8_lossy(&output.stdout)
      .lines()
      .map(|l| l.to_string())
      .collect();

    if changed_files.is_empty() {
      return Ok(all_members);
    }

    let affected = all_members.into_iter().filter(|member| {
      changed_files.iter().any(|f| f.starts_with(
        member.path.strip_prefix(dir).unwrap_or(&member.path).to_string_lossy().as_ref()
      ))
    }).collect();

    Ok(affected)
  }

  pub fn run_with_cache(&self, dir: &Path, member: &WorkspaceMember, task_name: &str, command: &str, args: &[&str]) -> Result<TaskResult> {
    let config = Self::detect(dir).unwrap_or(WorkspaceConfig {
      members: vec![], name: "ws".into(), version: "0.1.0".into(),
      shared_deps: HashMap::new(), pipeline: HashMap::new(),
      cache_config: CacheConfig::default(),
    });

    let cache_key = format!("{}:{}:{}", member.name, task_name, member.hash);

    if config.cache_config.enabled {
      if let Ok(cache) = self.cache.lock() {
        if let Some(entry) = cache.get(&cache_key) {
          if entry.hash == member.hash {
            return Ok(TaskResult {
              member: member.name.clone(),
              success: true,
              duration: 0.0,
              output: "(cached)".into(),
              cached: true,
            });
          }
        }
      }
    }

    let start = Instant::now();
    let output = Command::new(command)
      .args(args)
      .current_dir(&member.path)
      .output();
    let duration = start.elapsed().as_secs_f64();

    let result = match output {
      Ok(out) => {
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        TaskResult {
          member: member.name.clone(),
          success: out.status.success(),
          duration,
          output: format!("{stdout}\n{stderr}"),
          cached: false,
        }
      }
      Err(e) => TaskResult {
        member: member.name.clone(),
        success: false,
        duration,
        output: format!("{e:#}"),
        cached: false,
      },
    };

    if config.cache_config.enabled && result.success {
      let entry = CacheEntry {
        hash: member.hash.clone(),
        timestamp: Utc::now(),
        outputs: HashMap::new(),
      };
      if let Ok(mut cache) = self.cache.lock() {
        cache.insert(cache_key.clone(), entry);
      }

      if let Some(ref remote_url) = config.cache_config.remote_url {
        if let Ok(json) = serde_json::to_string(&self.cache.lock().unwrap().get(&cache_key)) {
          let _ = Self::put_remote_cache(remote_url, &cache_key, &json);
        }
      }
    }

    Ok(result)
  }

  fn put_remote_cache(url: &str, key: &str, data: &str) -> Result<()> {
    let endpoint = format!("{url}/cache/{key}");
    let result =
      std::process::Command::new("curl")
        .args(["-X", "PUT", "-H", "Content-Type: application/json", "-d", data, &endpoint])
        .output();
    match result {
      Ok(out) if out.status.success() => Ok(()),
      Ok(out) => anyhow::bail!("curl failed: {}", String::from_utf8_lossy(&out.stderr)),
      Err(e) => anyhow::bail!("failed to run curl: {e}"),
    }
  }

  pub fn add_member(dir: &Path, name: &str) -> Result<()> {
    let member_path = dir.join(name);
    if member_path.exists() {
      anyhow::bail!("member '{}' already exists at {}", name, member_path.display());
    }
    std::fs::create_dir_all(&member_path)
      .with_context(|| format!("failed to create member directory: {}", member_path.display()))?;
    let pkg_json = serde_json::json!({
      "name": name,
      "version": "0.1.0",
      "private": true,
    });
    std::fs::write(member_path.join("package.json"), serde_json::to_string_pretty(&pkg_json)?)
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
        pipeline: HashMap::new(),
        cache_config: CacheConfig::default(),
      }
    };
    if !config.members.contains(&name.to_string()) {
      config.members.push(name.to_string());
    }
    let toml_str = toml::to_string_pretty(&config).context("failed to serialize updated config")?;
    std::fs::write(&config_path, toml_str)?;
    Ok(())
  }

  pub fn remove_member(dir: &Path, name: &str) -> Result<()> {
    let member_path = dir.join(name);
    if member_path.exists() {
      std::fs::remove_dir_all(&member_path)
        .with_context(|| format!("failed to remove member directory: {}", member_path.display()))?;
    }
    let config_path = dir.join("klyron.toml");
    if config_path.exists() {
      let content = std::fs::read_to_string(&config_path)?;
      let mut config = toml::from_str::<WorkspaceConfig>(&content)?;
      config.members.retain(|m| m != name);
      let toml_str = toml::to_string_pretty(&config).context("failed to serialize updated config")?;
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
        anyhow::bail!("script '{}' failed in {} with exit code {:?}", script, member.name, status.code());
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
      eprintln!("[{}] executing '{} {}'...", member.name, cmd, args.join(" "));
      let status = Command::new(cmd)
        .args(args)
        .current_dir(&member.path)
        .status()
        .with_context(|| format!("failed to execute in {}", member.name))?;
      if !status.success() {
        anyhow::bail!("command '{} {}' failed in {} with exit code {:?}", cmd, args.join(" "), member.name, status.code());
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
      pipeline: HashMap::new(),
      cache_config: CacheConfig::default(),
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
    fs::write(dir.join("package.json"), serde_json::to_string_pretty(&pkg).unwrap()).unwrap();
    let detected = Workspace::detect(&dir).unwrap();
    assert_eq!(detected.name, "monorepo");
    assert!(detected.members.contains(&"packages/*".into()));
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
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_compute_dir_hash() {
    let dir = temp_dir("hash");
    fs::write(dir.join("a.txt"), "hello").unwrap();
    let h1 = Workspace::compute_dir_hash(&dir);
    fs::write(dir.join("a.txt"), "world").unwrap();
    let h2 = Workspace::compute_dir_hash(&dir);
    assert_ne!(h1, h2);
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_build_dependency_graph() {
    let dir = temp_dir("dep_graph");
    let pkg_a = dir.join("packages/a");
    let pkg_b = dir.join("packages/b");
    fs::create_dir_all(&pkg_a).unwrap();
    fs::create_dir_all(&pkg_b).unwrap();
    fs::write(
      pkg_a.join("package.json"),
      r#"{"name":"pkg-a","dependencies":{"pkg-b":"1.0.0"}}"#,
    ).unwrap();
    fs::write(pkg_b.join("package.json"), r#"{"name":"pkg-b"}"#).unwrap();
    fs::write(
      dir.join("package.json"),
      r#"{"name":"ws","workspaces":["packages/*"]}"#,
    ).unwrap();

    let graph = Workspace::build_dependency_graph(&dir).unwrap();
    let dot = Workspace::render_dependency_graph(&dir).unwrap();
    assert!(dot.contains("pkg-a"));
    assert!(dot.contains("pkg-b"));
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_add_and_remove_member() {
    let dir = temp_dir("add_remove");
    Workspace::init(&dir, "test-ws").unwrap();
    Workspace::add_member(&dir, "my-pkg").unwrap();
    let members = Workspace::list_members(&dir).unwrap();
    assert!(members.iter().any(|m| m.name == "my-pkg"));
    Workspace::remove_member(&dir, "my-pkg").unwrap();
    assert!(!dir.join("my-pkg").exists());
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_workspace_config_serialization() {
    let config = WorkspaceConfig {
      members: vec!["packages/*".into()],
      name: "test".into(),
      version: "0.1.0".into(),
      shared_deps: {
        let mut m = HashMap::new();
        m.insert("react".into(), "^18".into());
        m
      },
      pipeline: HashMap::new(),
      cache_config: CacheConfig::default(),
    };
    let toml_str = toml::to_string_pretty(&config).unwrap();
    let deserialized: WorkspaceConfig = toml::from_str(&toml_str).unwrap();
    assert_eq!(deserialized.name, "test");
    assert_eq!(deserialized.shared_deps.get("react").unwrap(), "^18");
  }
}
