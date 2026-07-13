//! Rails application server + generators.
//!
//! Translates `klyron run rails <command>` into Rails CLI commands.
//! Supports Rails-specific workflows:
//! - `rails server` — development server
//! - `rails generate` — scaffolding
//! - `rails db:migrate` — database migrations
//! - `rails console` — interactive console
//! - `rails runner` — script runner

use crate::{RubyConfig, RubyResult};

/// Rails CLI proxy
pub struct RailsCli {
  ruby_config: RubyConfig,
  project_root: String,
}

impl RailsCli {
  pub fn new(ruby_config: RubyConfig, project_root: &str) -> Self {
    Self { ruby_config, project_root: project_root.to_string() }
  }

  /// Find the Rails `bin/rails` script
  pub fn find_rails_bin(&self) -> Option<String> {
    let candidates = [
      format!("{}/bin/rails", self.project_root),
    ];
    candidates.iter().find(|p| std::path::Path::new(p).exists()).cloned()
  }

  /// Run a Rails command via `bin/rails` or `rails` CLI
  pub fn run(&self, command: &[String]) -> Result<RubyResult, String> {
    let ruby_bin = self.ruby_config.ruby_path.as_deref().unwrap_or("ruby");
    let use_bundler = self.ruby_config.use_bundler;

    let rails_script = self.find_rails_bin();

    if let Some(script) = rails_script {
      if use_bundler {
        let mut args = vec!["exec".to_string(), ruby_bin.to_string(), script];
        args.extend_from_slice(command);
        let output = std::process::Command::new("bundle")
          .args(&args)
          .stdout(std::process::Stdio::piped())
          .stderr(std::process::Stdio::piped())
          .current_dir(&self.project_root)
          .output()
          .map_err(|e| format!("Failed to run Rails command: {e}"))?;

        return Ok(RubyResult {
          stdout: String::from_utf8_lossy(&output.stdout).to_string(),
          stderr: String::from_utf8_lossy(&output.stderr).to_string(),
          exit_code: output.status.code().unwrap_or(-1),
          output_vars: std::collections::HashMap::new(),
        });
      } else {
        let mut args = vec![script];
        args.extend_from_slice(command);
        let output = std::process::Command::new(ruby_bin)
          .args(&args)
          .current_dir(&self.project_root)
          .output()
          .map_err(|e| format!("Failed to run Rails command: {e}"))?;

        return Ok(RubyResult {
          stdout: String::from_utf8_lossy(&output.stdout).to_string(),
          stderr: String::from_utf8_lossy(&output.stderr).to_string(),
          exit_code: output.status.code().unwrap_or(-1),
          output_vars: std::collections::HashMap::new(),
        });
      }
    }

    // Fall back: use rails CLI via bundler
    if use_bundler {
      let mut args = vec!["exec".to_string(), "rails".to_string()];
      args.extend_from_slice(command);
      let output = std::process::Command::new("bundle")
        .args(&args)
        .current_dir(&self.project_root)
        .output()
        .map_err(|e| format!("Failed to run Rails command: {e}"))?;

      Ok(RubyResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        output_vars: std::collections::HashMap::new(),
      })
    } else {
      let mut args = vec!["rails".to_string()];
      args.extend_from_slice(command);
      let output = std::process::Command::new(ruby_bin)
        .args(&args)
        .current_dir(&self.project_root)
        .output()
        .map_err(|e| format!("Failed to run Rails command: {e}"))?;

      Ok(RubyResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        output_vars: std::collections::HashMap::new(),
      })
    }
  }

  /// Start the Rails development server
  pub fn server(&self, host: &str, port: u16) -> Result<RubyResult, String> {
    self.run(&["server".to_string(), "-b".to_string(), host.to_string(), "-p".to_string(), port.to_string()])
  }

  /// Run `rails generate`
  pub fn generate(&self, generator: &str, args: &[&str]) -> Result<RubyResult, String> {
    let mut cmd = vec!["generate".to_string(), generator.to_string()];
    cmd.extend(args.iter().map(|a| (*a).to_string()));
    self.run(&cmd)
  }

  /// Run database migrations
  pub fn db_migrate(&self) -> Result<RubyResult, String> {
    self.run(&["db:migrate".to_string()])
  }

  /// Open Rails console
  pub fn console(&self) -> Result<RubyResult, String> {
    self.run(&["console".to_string()])
  }

  /// Run a script via `rails runner`
  pub fn runner(&self, script: &str) -> Result<RubyResult, String> {
    self.run(&["runner".to_string(), script.to_string()])
  }

  /// Run `rails db:seed`
  pub fn db_seed(&self) -> Result<RubyResult, String> {
    self.run(&["db:seed".to_string()])
  }

  /// Run `rails test`
  pub fn test(&self, path: Option<&str>) -> Result<RubyResult, String> {
    let mut cmd = vec!["test".to_string()];
    if let Some(p) = path {
      cmd.push(p.to_string());
    }
    self.run(&cmd)
  }

  /// Run `rails assets:precompile`
  pub fn assets_precompile(&self) -> Result<RubyResult, String> {
    self.run(&["assets:precompile".to_string()])
  }
}
