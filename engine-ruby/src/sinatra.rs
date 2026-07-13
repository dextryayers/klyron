//! Sinatra development server integration.
//!
//! Starts Sinatra applications via Rack-compatible server (Thin/Puma/WEBrick).
//! Supports auto-detection of the Sinatra app file, debug mode, and port binding.

use crate::{RubyConfig, RubyResult};

/// Sinatra server manager
pub struct SinatraServer {
  ruby_config: RubyConfig,
  project_root: String,
}

impl SinatraServer {
  pub fn new(ruby_config: RubyConfig, project_root: &str) -> Self {
    Self { ruby_config, project_root: project_root.to_string() }
  }

  /// Auto-detect the main Sinatra app file
  pub fn detect_app(&self) -> String {
    let candidates = ["app.rb", "main.rb", "server.rb", "web.rb", "src/app.rb", "src/main.rb"];
    candidates.iter()
      .find(|p| std::path::Path::new(&format!("{}/{}", self.project_root, p)).exists())
      .copied()
      .unwrap_or("app.rb")
      .to_string()
  }

  /// Look for a config.ru (Rack) file
  pub fn has_config_ru(&self) -> bool {
    std::path::Path::new(&format!("{}/config.ru", self.project_root)).exists()
  }

  /// Start the Sinatra/Rack application
  pub fn run(&self, host: &str, port: u16, server: Option<&str>, app_file: Option<&str>) -> Result<RubyResult, String> {
    let ruby_bin = self.ruby_config.ruby_path.as_deref().unwrap_or("ruby");
    let use_bundler = self.ruby_config.use_bundler;

    // Prefer config.ru → Rack (thin/puma/webrick)
    if self.has_config_ru() && app_file.is_none() {
      let rack_server = server.unwrap_or("webrick");
      let script = format!(
        r#"require 'rack'
Rack::Server.start(
  Port: {port},
  Host: '{host}',
  server: '{rack_server}',
  config: 'config.ru'
)"#,
        port = port,
        host = host,
        rack_server = rack_server,
      );

      if use_bundler {
        let output = std::process::Command::new("bundle")
          .args(&["exec", ruby_bin, "-e", &script])
          .current_dir(&self.project_root)
          .output()
          .map_err(|e| format!("Failed to start Sinatra: {e}"))?;

        return Ok(RubyResult {
          stdout: String::from_utf8_lossy(&output.stdout).to_string(),
          stderr: String::from_utf8_lossy(&output.stderr).to_string(),
          exit_code: output.status.code().unwrap_or(-1),
          output_vars: std::collections::HashMap::new(),
        });
      } else {
        let output = std::process::Command::new(ruby_bin)
          .args(&["-e", &script])
          .current_dir(&self.project_root)
          .output()
          .map_err(|e| format!("Failed to start Sinatra: {e}"))?;

        return Ok(RubyResult {
          stdout: String::from_utf8_lossy(&output.stdout).to_string(),
          stderr: String::from_utf8_lossy(&output.stderr).to_string(),
          exit_code: output.status.code().unwrap_or(-1),
          output_vars: std::collections::HashMap::new(),
        });
      }
    }

    // Direct Sinatra app file execution
    let app = match app_file { Some(a) => a.to_string(), None => self.detect_app() };
    let app_path = format!("{}/{}", self.project_root, app);

    let script = format!(
      r#"ENV['PORT'] = '{port}'
ENV['HOST'] = '{host}'
require './{app}'
Sinatra::Application.run! port: {port}, bind: '{host}'
"#,
      port = port,
      host = host,
      app = app.trim_end_matches(".rb"),
    );

    if use_bundler {
      let output = std::process::Command::new("bundle")
        .args(&["exec", ruby_bin, "-e", &script])
        .current_dir(&self.project_root)
        .output()
        .map_err(|e| format!("Failed to start Sinatra: {e}"))?;

      Ok(RubyResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        output_vars: std::collections::HashMap::new(),
      })
    } else {
      let output = std::process::Command::new(ruby_bin)
        .args(&[&app_path])
        .current_dir(&self.project_root)
        .output()
        .map_err(|e| format!("Failed to start Sinatra: {e}"))?;

      Ok(RubyResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        output_vars: std::collections::HashMap::new(),
      })
    }
  }

  /// Start with dev defaults
  pub fn run_dev(&self) -> Result<RubyResult, String> {
    self.run("127.0.0.1", 4567, Some("webrick"), None)
  }
}
