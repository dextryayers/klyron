//! FastAPI / Uvicorn server integration.
//!
//! Starts Uvicorn with the FastAPI app, managed by Klyron's process supervision.
//! Supports reload mode, custom host/port, and integration with Klyron's HTTP layer.

use crate::{PythonConfig, PythonResult};

/// FastAPI/Uvicorn server manager
pub struct FastApiServer {
  python_config: PythonConfig,
  project_root: String,
}

impl FastApiServer {
  pub fn new(python_config: PythonConfig, project_root: &str) -> Self {
    Self { python_config, project_root: project_root.to_string() }
  }

  /// Auto-detect the FastAPI app module (main:app or app:app)
  pub fn detect_app(&self) -> String {
    let candidates = ["main.py", "app.py", "api.py", "src/main.py", "src/app.py"];
    let module = candidates.iter()
      .find(|p| std::path::Path::new(&format!("{}/{}", self.project_root, p)).exists())
      .map(|p| p.trim_end_matches(".py").replace('/', "."))
      .unwrap_or_else(|| "main".to_string());
    format!("{}:app", module)
  }

  /// Start the Uvicorn server
  pub fn serve(&self, host: &str, port: u16, reload: bool, app: Option<&str>) -> Result<PythonResult, String> {
    let app_str = match app { Some(a) => a.to_string(), None => self.detect_app() };
    let python_bin = self.python_config.python_path.as_deref().unwrap_or("python3");

    let mut args = vec![
      "-m".to_string(),
      "uvicorn".to_string(),
      app_str,
      format!("--host={host}"),
      format!("--port={port}"),
    ];

    if reload {
      args.push("--reload".to_string());
    }

    let output = std::process::Command::new(python_bin)
      .args(&args)
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .current_dir(&self.project_root)
      .output()
      .map_err(|e| format!("Failed to start FastAPI/Uvicorn: {e}"))?;

    Ok(PythonResult {
      stdout: String::from_utf8_lossy(&output.stdout).to_string(),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      output_vars: std::collections::HashMap::new(),
    })
  }

  /// Start with production defaults (no reload, 0.0.0.0:8000)
  pub fn serve_production(&self) -> Result<PythonResult, String> {
    self.serve("0.0.0.0", 8000, false, None)
  }

  /// Start with dev defaults (with reload, localhost:8000)
  pub fn serve_dev(&self) -> Result<PythonResult, String> {
    self.serve("127.0.0.1", 8000, true, None)
  }
}
