//! Django management CLI proxy.
//!
//! Translates `klyron run manage.py <command>` into Django management commands.
//! Also provides direct framework helpers.

use crate::{PythonConfig, PythonResult};

/// Django management CLI proxy
pub struct DjangoCli {
  python_config: PythonConfig,
  project_root: String,
}

impl DjangoCli {
  pub fn new(python_config: PythonConfig, project_root: &str) -> Self {
    Self { python_config, project_root: project_root.to_string() }
  }

  /// Find manage.py in the project root
  pub fn find_manage_py(&self) -> Option<String> {
    let candidates = [
      format!("{}/manage.py", self.project_root),
      format!("{}/src/manage.py", self.project_root),
    ];
    candidates.iter().find(|p| std::path::Path::new(p).exists()).cloned()
  }

  /// Run a Django management command (e.g., `runserver`, `migrate`, `makemigrations`)
  pub fn run(&self, command: &[String]) -> Result<PythonResult, String> {
    let manage_py = self.find_manage_py()
      .ok_or_else(|| "No manage.py found. Run `klyron pip install django` first.".to_string())?;

    let mut args = vec![manage_py];
    args.extend_from_slice(command);

    let python_bin = self.python_config.python_path.as_deref().unwrap_or("python3");
    let output = std::process::Command::new(python_bin)
      .args(&args)
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .current_dir(&self.project_root)
      .output()
      .map_err(|e| format!("Failed to run Django command: {e}"))?;

    Ok(PythonResult {
      stdout: String::from_utf8_lossy(&output.stdout).to_string(),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      output_vars: std::collections::HashMap::new(),
    })
  }

  /// Shortcut: `runserver` with optional host:port
  pub fn runserver(&self, host: &str, port: u16) -> Result<PythonResult, String> {
    self.run(&["runserver".to_string(), format!("{host}:{port}")])
  }

  /// Shortcut: `migrate`
  pub fn migrate(&self) -> Result<PythonResult, String> {
    self.run(&["migrate".to_string()])
  }

  /// Shortcut: `makemigrations`
  pub fn makemigrations(&self) -> Result<PythonResult, String> {
    self.run(&["makemigrations".to_string()])
  }

  /// Shortcut: `createsuperuser`
  pub fn createsuperuser(&self) -> Result<PythonResult, String> {
    self.run(&["createsuperuser".to_string()])
  }

  /// Shortcut: `shell`
  pub fn shell(&self) -> Result<PythonResult, String> {
    self.run(&["shell".to_string()])
  }

  /// Shortcut: `test`
  pub fn test(&self, app: Option<&str>) -> Result<PythonResult, String> {
    let mut args = vec!["test".to_string()];
    if let Some(a) = app {
      args.push(a.to_string());
    }
    self.run(&args)
  }

  /// Shortcut: `collectstatic`
  pub fn collectstatic(&self) -> Result<PythonResult, String> {
    self.run(&["collectstatic".to_string(), "--noinput".to_string()])
  }
}
