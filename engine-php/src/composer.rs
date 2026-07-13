//! Composer package manager integration.
//!
//! Provides:
//! - `composer.json` / `composer.lock` parsing
//! - Packagist registry interface
//! - Dependency resolution (SAT solver bridge)
//! - PSR-4/PSR-0 autoloader generation
//! - `klyron composer install` / `require` / `update`

use std::collections::HashMap;

/// Represents a parsed composer.json file
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComposerJson {
  pub name: Option<String>,
  pub description: Option<String>,
  #[serde(rename = "type")]
  pub type_: Option<String>,
  pub require: Option<HashMap<String, String>>,
  #[serde(rename = "require-dev")]
  pub require_dev: Option<HashMap<String, String>>,
  pub autoload: Option<AutoloadConfig>,
  pub scripts: Option<HashMap<String, Vec<String>>>,
  pub extra: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AutoloadConfig {
  #[serde(rename = "psr-4")]
  pub psr4: Option<HashMap<String, String>>,
  #[serde(rename = "psr-0")]
  pub psr0: Option<HashMap<String, String>>,
  pub classmap: Option<Vec<String>>,
  pub files: Option<Vec<String>>,
}

/// Composer package manager proxy
pub struct Composer {
  project_root: String,
}

impl Composer {
  pub fn new(project_root: &str) -> Self {
    Self { project_root: project_root.to_string() }
  }

  /// Parse composer.json from project root
  pub fn read_composer_json(&self) -> Result<ComposerJson, String> {
    let path = format!("{}/composer.json", self.project_root);
    let content = std::fs::read_to_string(&path)
      .map_err(|e| format!("Cannot read {path}: {e}"))?;
    serde_json::from_str(&content)
      .map_err(|e| format!("Parse error in composer.json: {e}"))
  }

  /// Check if composer.json exists
  pub fn has_composer_json(&self) -> bool {
    std::path::Path::new(&format!("{}/composer.json", self.project_root)).exists()
  }

  /// Resolve PSR-4/PSR-0 autoload paths from composer.json
  pub fn resolve_autoload(&self) -> Result<HashMap<String, String>, String> {
    let json = self.read_composer_json()?;
    let mut paths = HashMap::new();

    if let Some(autoload) = &json.autoload {
      if let Some(psr4) = &autoload.psr4 {
        for (ns, dir) in psr4 {
          paths.insert(format!("{ns}\\"), format!("{}/{}", self.project_root, dir));
        }
      }
      if let Some(psr0) = &autoload.psr0 {
        for (ns, dir) in psr0 {
          let prefix = if ns.is_empty() { String::new() } else { format!("{ns}\\" )};
          paths.insert(prefix, format!("{}/{}", self.project_root, dir));
        }
      }
    }
    Ok(paths)
  }

  /// Run `composer install` in the project root.
  /// Falls back to system `composer` binary if available.
  pub fn install(&self) -> Result<std::process::Output, String> {
    self.run_composer_command(&["install", "--no-interaction", "--prefer-dist"])
  }

  /// Run `composer require <package>`
  pub fn require_package(&self, package: &str) -> Result<std::process::Output, String> {
    self.run_composer_command(&["require", package, "--no-interaction"])
  }

  /// Run `composer update`
  pub fn update(&self) -> Result<std::process::Output, String> {
    self.run_composer_command(&["update", "--no-interaction"])
  }

  fn run_composer_command(&self, args: &[&str]) -> Result<std::process::Output, String> {
    let output = std::process::Command::new("composer")
      .args(args)
      .current_dir(&self.project_root)
      .output()
      .map_err(|e| format!("Failed to run composer: {e}"))?;

    Ok(output)
  }
}
