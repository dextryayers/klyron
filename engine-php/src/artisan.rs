//! Laravel Artisan CLI proxy.
//!
//! Translates `klyron run artisan <command>` into PHP Artisan execution.
//! Discovers artisan in the project directory and executes it via the PHP engine.

use crate::{PhpConfig, PhpResult};

/// Artisan CLI proxy — finds and runs `artisan` in the project
pub struct ArtisanCli {
  php_config: PhpConfig,
  project_root: String,
}

impl ArtisanCli {
  pub fn new(php_config: PhpConfig, project_root: &str) -> Self {
    Self { php_config, project_root: project_root.to_string() }
  }

  /// Find the artisan file in the project root
  pub fn find_artisan(&self) -> Option<String> {
    let candidates = [
      format!("{}/artisan", self.project_root),
      format!("{}/bin/artisan", self.project_root),
    ];
    candidates.iter().find(|p| std::path::Path::new(p).exists()).cloned()
  }

  /// Run an artisan command (e.g., `serve`, `make:model`, `migrate`)
  pub fn run(&self, command: &[String]) -> Result<PhpResult, String> {
    let artisan = self.find_artisan()
      .ok_or_else(|| "No artisan file found. Run `klyron composer install` first.".to_string())?;

    let mut args = vec![artisan];
    args.extend_from_slice(command);

    // Run PHP with the artisan script
    let php_bin = self.php_config.php_path.as_deref().unwrap_or("php");
    let output = std::process::Command::new(php_bin)
      .args(&args)
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .output()
      .map_err(|e| format!("Failed to run artisan: {e}"))?;

    Ok(PhpResult {
      stdout: String::from_utf8_lossy(&output.stdout).to_string(),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      output_vars: std::collections::HashMap::new(),
    })
  }

  /// Start the Laravel development server
  pub fn serve(&self, host: &str, port: u16) -> Result<PhpResult, String> {
    self.run(&[
      "serve".to_string(),
      format!("--host={host}"),
      format!("--port={port}"),
    ])
  }

  /// Shortcut for common artisan commands
  pub fn make(&self, kind: &str, name: &str) -> Result<PhpResult, String> {
    self.run(&[format!("make:{kind}"), name.to_string()])
  }

  pub fn migrate(&self) -> Result<PhpResult, String> {
    self.run(&["migrate".to_string()])
  }
}
