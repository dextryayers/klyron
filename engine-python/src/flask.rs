//! Flask development server integration.
//!
//! Starts Flask's built-in dev server, managed by Klyron's process supervision.
//! Supports debug mode, custom host/port, and auto-detection of the Flask app.

use crate::{PythonConfig, PythonResult};

/// Flask development server manager
pub struct FlaskServer {
  python_config: PythonConfig,
  project_root: String,
}

impl FlaskServer {
  pub fn new(python_config: PythonConfig, project_root: &str) -> Self {
    Self { python_config, project_root: project_root.to_string() }
  }

  /// Auto-detect the Flask app module
  pub fn detect_app(&self) -> String {
    let candidates = ["app.py", "main.py", "wsgi.py", "src/app.py", "src/main.py"];
    candidates.iter()
      .find(|p| std::path::Path::new(&format!("{}/{}", self.project_root, p)).exists())
      .map(|p| p.trim_end_matches(".py").replace('/', "."))
      .unwrap_or_else(|| "app".to_string())
  }

  /// Run the Flask development server
  pub fn run(&self, host: &str, port: u16, debug: bool, app: Option<&str>) -> Result<PythonResult, String> {
    let app_module = match app { Some(a) => a.to_string(), None => self.detect_app() };
    let python_bin = self.python_config.python_path.as_deref().unwrap_or("python3");

    let script = format!(
      r#"import os, sys
os.environ['FLASK_APP'] = '{app_module}'
os.environ['FLASK_DEBUG'] = '{debug}'
from werkzeug.serving import run_simple
import importlib
mod = importlib.import_module('{app_module}')
app = getattr(mod, 'app', getattr(mod, 'application', None))
if app is None:
    for attr in dir(mod):
        if hasattr(getattr(mod, attr), 'run'):
            app = getattr(mod, attr)
            break
if app is None:
    sys.stderr.write('Could not find Flask app in {app_module}')
    sys.exit(1)
run_simple('{host}', {port}, app, use_debugger={debug}, use_reloader={debug})
"#,
      app_module = app_module,
      host = host,
      port = port,
      debug = if debug { "1" } else { "0" },
    );

    let output = std::process::Command::new(python_bin)
      .arg("-c")
      .arg(&script)
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .current_dir(&self.project_root)
      .output()
      .map_err(|e| format!("Failed to start Flask: {e}"))?;

    Ok(PythonResult {
      stdout: String::from_utf8_lossy(&output.stdout).to_string(),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      output_vars: std::collections::HashMap::new(),
    })
  }

  /// Start with dev defaults
  pub fn run_dev(&self) -> Result<PythonResult, String> {
    self.run("127.0.0.1", 5000, true, None)
  }
}
