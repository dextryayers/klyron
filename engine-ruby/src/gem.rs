//! Ruby gem integration — RubyGems registry, Gemfile, Bundler.
//!
//! Provides:
//! - RubyGems.org JSON API resolution
//! - Gemfile / Gemfile.lock parsing
//! - Bundler wrapper for `bundle install` / `bundle exec`
//! - `klyron gem install` / `uninstall` / `list`

use std::collections::HashMap;

/// Represents a parsed Gemfile dependency
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GemDependency {
  pub name: String,
  pub version_requirement: Option<String>,
  pub groups: Vec<String>,
  pub source: Option<String>,
  pub git: Option<String>,
  pub branch: Option<String>,
  pub require: Option<String>,
}

/// Parsed Gemfile
#[derive(Debug, Clone, Default)]
pub struct Gemfile {
  pub sources: Vec<String>,
  pub dependencies: Vec<GemDependency>,
  pub ruby_version: Option<String>,
}

impl Gemfile {
  /// Simple Gemfile parser — handles the common `gem "name", "~> x.y"` format
  pub fn parse(content: &str) -> Self {
    let mut gf = Gemfile::default();

    for line in content.lines() {
      let line = line.trim();
      if line.is_empty() || line.starts_with('#') {
        continue;
      }
      if line.starts_with("source ") {
        let src = line.trim_start_matches("source ")
          .trim()
          .trim_matches('"')
          .trim_matches('\'');
        gf.sources.push(src.to_string());
        continue;
      }
      if line.starts_with("ruby ") {
        gf.ruby_version = Some(
          line.trim_start_matches("ruby ")
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string(),
        );
        continue;
      }
      if line.starts_with("gem ") {
        // Parse: gem "name", "~> 1.0", group: :development
        let rest = line.trim_start_matches("gem ").trim();
        let parts: Vec<&str> = rest.split(',').map(|s| s.trim().trim_matches('"').trim_matches('\'')).collect();
        let name = parts.first().unwrap_or(&"").to_string();
        let version = parts.get(1).map(|s| s.to_string()).filter(|s| !s.starts_with("group") && !s.starts_with("git") && !s.starts_with("branch") && !s.starts_with("require"));
        let groups: Vec<String> = parts.iter()
          .filter(|s| s.starts_with("group") || s.starts_with(":group"))
          .flat_map(|s| s.split(':').skip(1))
          .map(|s| s.trim().to_string())
          .collect();

        gf.dependencies.push(GemDependency {
          name,
          version_requirement: version,
          groups,
          source: None,
          git: None,
          branch: None,
          require: None,
        });
      }
    }
    gf
  }
}

/// RubyGems package info from the JSON API
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RubyGemsInfo {
  pub name: String,
  pub version: String,
  pub info: Option<String>,
  pub authors: Option<Vec<String>>,
  pub licenses: Option<Vec<String>>,
  pub homepage_uri: Option<String>,
  pub gem_uri: Option<String>,
  pub dependencies: Option<HashMap<String, Vec<RubyGemsDep>>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RubyGemsDep {
  pub name: String,
  pub requirements: String,
}

/// gem package manager proxy
pub struct GemManager {
  project_root: String,
  _ruby_bin: String,
}

impl GemManager {
  pub fn new(project_root: &str) -> Self {
    Self {
      project_root: project_root.to_string(),
      _ruby_bin: "ruby".to_string(),
    }
  }

  /// Fetch gem info from RubyGems.org JSON API
  pub fn fetch_gem_info(&self, name: &str) -> Result<RubyGemsInfo, String> {
    let url = format!("https://rubygems.org/api/v1/gems/{}.json", name);
    let resp = reqwest::blocking::get(&url)
      .map_err(|e| format!("RubyGems request failed: {e}"))?;
    let info: RubyGemsInfo = serde_json::from_reader(resp)
      .map_err(|e| format!("Failed to parse RubyGems response: {e}"))?;
    Ok(info)
  }

  /// Run `bundle install` in the project root
  pub fn bundle_install(&self) -> Result<std::process::Output, String> {
    std::process::Command::new("bundle")
      .arg("install")
      .current_dir(&self.project_root)
      .output()
      .map_err(|e| format!("Failed to run bundle install: {e}"))
  }

  /// Run `bundle exec <command>` so gems from the Gemfile are available
  pub fn bundle_exec(&self, command: &[&str]) -> Result<std::process::Output, String> {
    let mut cmd = std::process::Command::new("bundle");
    cmd.arg("exec");
    for c in command {
      cmd.arg(c);
    }
    cmd.current_dir(&self.project_root)
      .output()
      .map_err(|e| format!("Failed to run bundle exec: {e}"))
  }

  /// Run `gem install <package>`
  pub fn gem_install(&self, name: &str) -> Result<std::process::Output, String> {
    std::process::Command::new("gem")
      .args(&["install", name, "--no-document"])
      .output()
      .map_err(|e| format!("Failed to run gem install: {e}"))
  }

  /// Run `gem uninstall <package>`
  pub fn gem_uninstall(&self, name: &str) -> Result<std::process::Output, String> {
    std::process::Command::new("gem")
      .args(&["uninstall", name, "-x"])
      .output()
      .map_err(|e| format!("Failed to run gem uninstall: {e}"))
  }

  /// Run `gem list` for installed gems
  pub fn gem_list(&self) -> Result<String, String> {
    let output = std::process::Command::new("gem")
      .args(&["list"])
      .output()
      .map_err(|e| format!("Failed to run gem list: {e}"))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
  }

  /// Read and parse the Gemfile
  pub fn read_gemfile(&self) -> Result<Gemfile, String> {
    let content = std::fs::read_to_string(format!("{}/Gemfile", self.project_root))
      .map_err(|e| format!("Cannot read Gemfile: {e}"))?;
    Ok(Gemfile::parse(&content))
  }

  /// Check if Gemfile exists
  pub fn has_gemfile(&self) -> bool {
    std::path::Path::new(&format!("{}/Gemfile", self.project_root)).exists()
  }
}
