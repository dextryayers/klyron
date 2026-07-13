//! pip integration — PyPI registry, requirements.txt, virtual environments.
//!
//! Provides:
//! - PyPI registry resolution
//! - requirements.txt parser
//! - Virtual environment creation/management
//! - `klyron pip install` / `uninstall` / `freeze` / `list`

use std::collections::HashMap;

/// Represents a parsed requirement from requirements.txt
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Requirement {
  pub name: String,
  pub version_spec: Option<String>,
  pub extras: Vec<String>,
  pub markers: Option<String>,
  pub url: Option<String>,
}

/// Parsed requirements.txt
#[derive(Debug, Clone)]
pub struct RequirementsTxt {
  pub requirements: Vec<Requirement>,
  pub recursive: Vec<String>,
}

impl RequirementsTxt {
  /// Parse a requirements.txt string
  pub fn parse(content: &str) -> Self {
    let mut requirements = Vec::new();
    let mut recursive = Vec::new();

    for line in content.lines() {
      let line = line.trim();
      if line.is_empty() || line.starts_with('#') || line.starts_with("-i") || line.starts_with("--") {
        continue;
      }
      if line.starts_with("-r ") || line.starts_with("--requirement ") {
        let path = line.split_once(' ').map(|(_, p)| p.trim()).unwrap_or("");
        recursive.push(path.to_string());
        continue;
      }
      if let Some((name, rest)) = line.split_once("==") {
        requirements.push(Requirement {
          name: name.trim().to_string(),
          version_spec: Some(format!("=={}", rest.trim())),
          extras: vec![],
          markers: None,
          url: None,
        });
      } else if let Some((name, _rest)) = line.split_once(">=") {
        requirements.push(Requirement {
          name: name.trim().to_string(),
          version_spec: Some(line.trim().to_string()),
          extras: vec![],
          markers: None,
          url: None,
        });
      } else {
        requirements.push(Requirement {
          name: line.to_string(),
          version_spec: None,
          extras: vec![],
          markers: None,
          url: None,
        });
      }
    }

    Self { requirements, recursive }
  }
}

/// PyPI package info from the JSON API
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PyPiPackageInfo {
  pub info: PyPiInfo,
  pub releases: HashMap<String, Vec<PyPiRelease>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PyPiInfo {
  pub name: String,
  pub version: String,
  pub summary: Option<String>,
  pub description: Option<String>,
  pub author: Option<String>,
  pub license: Option<String>,
  pub requires_dist: Option<Vec<String>>,
  pub requires_python: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PyPiRelease {
  pub filename: String,
  pub url: String,
  pub python_version: String,
  pub packagetype: String,
}

/// pip package manager proxy
pub struct PipManager {
  project_root: String,
  python_bin: String,
}

impl PipManager {
  pub fn new(project_root: &str) -> Self {
    Self {
      project_root: project_root.to_string(),
      python_bin: "python3".to_string(),
    }
  }

  /// Fetch package info from PyPI JSON API
  pub fn fetch_package_info(&self, name: &str) -> Result<PyPiPackageInfo, String> {
    let url = format!("https://pypi.org/pypi/{}/json", name);
    let resp = reqwest::blocking::get(&url)
      .map_err(|e| format!("PyPI request failed: {e}"))?;
    let info: PyPiPackageInfo = serde_json::from_reader(resp)
      .map_err(|e| format!("Failed to parse PyPI response: {e}"))?;
    Ok(info)
  }

  /// Create a virtual environment in the project root
  pub fn create_venv(&self) -> Result<std::process::Output, String> {
    let venv_path = format!("{}/.venv", self.project_root);
    let output = std::process::Command::new(&self.python_bin)
      .args(&["-m", "venv", &venv_path])
      .output()
      .map_err(|e| format!("Failed to create venv: {e}"))?;
    Ok(output)
  }

  /// Run `pip install` with given packages
  pub fn install(&self, packages: &[&str]) -> Result<std::process::Output, String> {
    let pip = self.resolve_pip();
    let mut cmd = std::process::Command::new(&pip);
    cmd.args(&["install", "--no-input"]);
    for pkg in packages {
      cmd.arg(pkg);
    }
    cmd.output()
      .map_err(|e| format!("Failed to run pip install: {e}"))
  }

  /// Install packages from requirements.txt
  pub fn install_requirements(&self) -> Result<std::process::Output, String> {
    let pip = self.resolve_pip();
    std::process::Command::new(&pip)
      .args(&["install", "-r", "requirements.txt", "--no-input"])
      .output()
      .map_err(|e| format!("Failed to run pip install -r: {e}"))
  }

  /// Run `pip freeze` to list installed packages
  pub fn freeze(&self) -> Result<String, String> {
    let pip = self.resolve_pip();
    let output = std::process::Command::new(&pip)
      .args(&["freeze"])
      .output()
      .map_err(|e| format!("Failed to run pip freeze: {e}"))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
  }

  /// Run `pip list` for installed packages
  pub fn list_packages(&self) -> Result<std::process::Output, String> {
    let pip = self.resolve_pip();
    std::process::Command::new(&pip)
      .args(&["list", "--format=columns"])
      .output()
      .map_err(|e| format!("Failed to run pip list: {e}"))
  }

  /// Run `pip uninstall` for given packages
  pub fn uninstall(&self, packages: &[&str]) -> Result<std::process::Output, String> {
    let pip = self.resolve_pip();
    let mut cmd = std::process::Command::new(&pip);
    cmd.args(&["uninstall", "-y"]);
    for pkg in packages {
      cmd.arg(pkg);
    }
    cmd.output()
      .map_err(|e| format!("Failed to run pip uninstall: {e}"))
  }

  fn resolve_pip(&self) -> String {
    // Prefer venv pip if available
    let venv_pip = format!("{}/.venv/bin/pip", self.project_root);
    if std::path::Path::new(&venv_pip).exists() {
      return venv_pip;
    }
    // Fall back to python3 -m pip
    format!("{} -m pip", self.python_bin)
  }
}
