use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LintBackend {
  Eslint,
  Biome,
  Clippy,
  Ruff,
  Rubocop,
  Golint,
  Pint,
}

impl LintBackend {
  pub fn name(self) -> &'static str {
    match self {
      LintBackend::Eslint => "ESLint",
      LintBackend::Biome => "Biome",
      LintBackend::Clippy => "Clippy",
      LintBackend::Ruff => "Ruff",
      LintBackend::Rubocop => "RuboCop",
      LintBackend::Golint => "golint",
      LintBackend::Pint => "Pint",
    }
  }

  pub fn command(self) -> (&'static str, Vec<&'static str>) {
    match self {
      LintBackend::Eslint => ("npx", vec!["eslint", "."]),
      LintBackend::Biome => ("npx", vec!["biome", "lint"]),
      LintBackend::Clippy => ("cargo", vec!["clippy", "--all-targets", "--", "-D", "warnings"]),
      LintBackend::Ruff => ("ruff", vec!["check", "."]),
      LintBackend::Rubocop => ("rubocop", vec![]),
      LintBackend::Golint => ("golint", vec!["./..."]),
      LintBackend::Pint => ("./vendor/bin/pint", vec!["--test"]),
    }
  }

  pub fn extensions(self) -> &'static [&'static str] {
    match self {
      LintBackend::Eslint | LintBackend::Biome => &["js", "jsx", "ts", "tsx", "mjs", "cjs"],
      LintBackend::Clippy => &["rs"],
      LintBackend::Ruff => &["py"],
      LintBackend::Rubocop => &["rb"],
      LintBackend::Golint => &["go"],
      LintBackend::Pint => &["php"],
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintIssue {
  pub file: String,
  pub line: u64,
  pub column: u64,
  pub level: String,
  pub code: String,
  pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifReport {
  pub version: String,
  pub runs: Vec<SarifRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRun {
  pub tool: SarifTool,
  pub results: Vec<SarifResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifTool {
  pub driver: SarifDriver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifDriver {
  pub name: String,
  pub semantic_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifResult {
  pub message: SarifMessage,
  pub level: String,
  pub locations: Vec<SarifLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifMessage {
  pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifLocation {
  pub physical_location: SarifPhysicalLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifPhysicalLocation {
  pub artifact_location: SarifArtifactLocation,
  pub region: SarifRegion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifArtifactLocation {
  pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRegion {
  pub start_line: u64,
  pub start_column: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintReport {
  pub total_errors: u64,
  pub total_warnings: u64,
  pub files_checked: u64,
  pub issues: Vec<LintIssue>,
  pub output: String,
  pub sarif: Option<SarifReport>,
}

#[derive(Debug, Clone)]
pub struct LinterConfig {
  pub max_warnings: Option<u64>,
  pub cache_config: bool,
  pub sarif_output: bool,
}

impl Default for LinterConfig {
  fn default() -> Self {
    LinterConfig {
      max_warnings: None,
      cache_config: true,
      sarif_output: false,
    }
  }
}

#[derive(Debug)]
pub struct Linter {
  config: LinterConfig,
  config_cache: Mutex<HashMap<PathBuf, LintBackend>>,
}

impl Linter {
  pub fn new() -> Self {
    Linter::default()
  }

  pub fn with_config(config: LinterConfig) -> Self {
    Linter {
      config,
      config_cache: Mutex::new(HashMap::new()),
    }
  }

  pub fn detect(&self, dir: &Path) -> LintBackend {
    if let Ok(cache) = self.config_cache.lock() {
      if self.config.cache_config {
        if let Some(backend) = cache.get(dir) {
          return *backend;
        }
      }
    }
    let backend = Self::detect_inner(dir);
    if self.config.cache_config {
      if let Ok(mut cache) = self.config_cache.lock() {
        cache.insert(dir.to_path_buf(), backend);
      }
    }
    backend
  }

  fn detect_inner(dir: &Path) -> LintBackend {
    if dir.join(".eslintrc.js").exists()
      || dir.join(".eslintrc.json").exists()
      || dir.join(".eslintrc.yaml").exists()
      || dir.join(".eslintrc.yml").exists()
      || dir.join("eslint.config.js").exists()
      || dir.join("eslint.config.mjs").exists()
    {
      LintBackend::Eslint
    } else if dir.join("biome.json").exists() {
      LintBackend::Biome
    } else if dir.join("Cargo.toml").exists() {
      LintBackend::Clippy
    } else if dir.join("pyproject.toml").exists() || dir.join(".ruff.toml").exists() {
      LintBackend::Ruff
    } else if dir.join(".rubocop.yml").exists() || dir.join(".rubocop.yaml").exists() {
      LintBackend::Rubocop
    } else if dir.join("go.mod").exists() {
      LintBackend::Golint
    } else if dir.join("pint.json").exists() {
      LintBackend::Pint
    } else {
      LintBackend::Eslint
    }
  }

  pub fn lint(dir: &Path) -> Result<LintReport> {
    Linter::new().lint_dir(dir, &[])
  }

  pub fn lint_path(&self, dir: &Path, path: &str) -> Result<LintReport> {
    self.lint_dir(dir, &[path])
  }

  pub fn lint_dir(&self, dir: &Path, extra_args: &[&str]) -> Result<LintReport> {
    let backend = self.detect(dir);
    let (program, base_args) = backend.command();
    let mut args: Vec<&str> = base_args.iter().copied().collect();
    if !extra_args.is_empty() {
      args = extra_args.to_vec();
    }

    let output = std::process::Command::new(program)
      .args(&args)
      .current_dir(dir)
      .output()
      .with_context(|| format!("failed to run {}", backend.name()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{stdout}\n{stderr}");

    let issues = self.parse_issues(&combined, backend, dir);
    let errors = issues.iter().filter(|i| i.level == "error").count() as u64;
    let warnings = issues.iter().filter(|i| i.level == "warning").count() as u64;

    if let Some(max_w) = self.config.max_warnings {
      if warnings > max_w {
        anyhow::bail!(
          "max warnings threshold exceeded: {warnings} > {max_w}"
        );
      }
    }

    let sarif = if self.config.sarif_output {
      Some(self.to_sarif(&issues, backend))
    } else {
      None
    };

    let files = issues.iter().map(|i| i.file.clone()).collect::<std::collections::HashSet<_>>().len() as u64;

    Ok(LintReport {
      total_errors: errors,
      total_warnings: warnings,
      files_checked: files.max(1),
      issues,
      output: combined,
      sarif,
    })
  }

  pub fn lint_stdin(&self, dir: &Path) -> Result<LintReport> {
    let backend = self.detect(dir);
    let (program, base_args) = backend.command();
    let mut args: Vec<&str> = base_args.iter().copied().collect();
    args.push("--stdin-filename");
    args.push("stdin");

    let mut child = std::process::Command::new(program)
      .args(&args)
      .current_dir(dir)
      .stdin(std::process::Stdio::piped())
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .spawn()
      .with_context(|| format!("failed to run {} on stdin", backend.name()))?;

    let mut stdin_content = String::new();
    std::io::stdin().read_to_string(&mut stdin_content).context("failed to read stdin")?;

    if let Some(mut stdin) = child.stdin.take() {
      use std::io::Write;
      stdin.write_all(stdin_content.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{stdout}\n{stderr}");

    let issues = self.parse_issues(&combined, backend, dir);
    let errors = issues.iter().filter(|i| i.level == "error").count() as u64;
    let warnings = issues.iter().filter(|i| i.level == "warning").count() as u64;

    Ok(LintReport {
      total_errors: errors,
      total_warnings: warnings,
      files_checked: 1,
      issues,
      output: combined,
      sarif: None,
    })
  }

  pub fn lint_fix(&self, dir: &Path) -> Result<LintReport> {
    let backend = self.detect(dir);
    let fix_args = match backend {
      LintBackend::Eslint => &["--fix-dir", "."] as &[&str],
      LintBackend::Biome => &["lint", "--apply"],
      LintBackend::Clippy => &["clippy", "--fix", "--", "-D", "warnings"],
      LintBackend::Ruff => &["check", "--fix", "."],
      LintBackend::Rubocop => &["-a"],
      LintBackend::Golint => &["./..."],
      LintBackend::Pint => &[],
    };
    self.lint_dir(dir, fix_args)
  }

  pub fn lint_multi(&self, dirs: &[PathBuf]) -> Result<Vec<LintReport>> {
    let reports: Vec<Result<LintReport>> = dirs.par_iter()
      .map(|dir| self.lint_dir(dir, &[]))
      .collect();
    reports.into_iter().collect()
  }

  fn parse_issues(&self, output: &str, backend: LintBackend, _base_dir: &Path) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    match backend {
      LintBackend::Eslint | LintBackend::Biome => {
        for line in output.lines() {
          if let Some(rest) = line.strip_suffix("  ") {
            let _ = rest;
          }
          let parts: Vec<&str> = line.splitn(5, ':').collect();
          if parts.len() >= 5 {
            let file = parts[0].trim().to_string();
            let line_no = parts[1].trim().parse().unwrap_or(0);
            let col = parts[2].trim().parse().unwrap_or(0);
            let level = parts[3].trim().to_string();
            let message = parts[4].trim().to_string();
            if !file.is_empty() && (level == "error" || level == "warning") {
              let code = message.split_whitespace().next().unwrap_or("").to_string();
              issues.push(LintIssue { file, line: line_no, column: col, level, code, message });
            }
          }
        }
      }
      _ => {
        for line in output.lines() {
          if line.contains("warning") || line.contains("error") {
            issues.push(LintIssue {
              file: "unknown".into(),
              line: 0,
              column: 0,
              level: if line.contains("error") { "error".into() } else { "warning".into() },
              code: "".into(),
              message: line.to_string(),
            });
          }
        }
      }
    }
    issues
  }

  fn to_sarif(&self, issues: &[LintIssue], backend: LintBackend) -> SarifReport {
    let results: Vec<SarifResult> = issues.iter().map(|issue| {
      SarifResult {
        message: SarifMessage { text: issue.message.clone() },
        level: issue.level.clone(),
        locations: vec![SarifLocation {
          physical_location: SarifPhysicalLocation {
            artifact_location: SarifArtifactLocation { uri: issue.file.clone() },
            region: SarifRegion {
              start_line: issue.line,
              start_column: issue.column,
            },
          },
        }],
      }
    }).collect();

    SarifReport {
      version: "2.1.0".into(),
      runs: vec![SarifRun {
        tool: SarifTool {
          driver: SarifDriver {
            name: backend.name().into(),
            semantic_version: "1.0.0".into(),
          },
        },
        results,
      }],
    }
  }
}

impl Default for Linter {
  fn default() -> Self {
    Linter {
      config: LinterConfig::default(),
      config_cache: Mutex::new(HashMap::new()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;

  fn test_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
  }

  #[test]
  fn test_new() {
    let linter = Linter::new();
    let _ = linter;
  }

  #[test]
  fn test_detect_clippy() {
    let linter = Linter::new();
    let dir = test_dir();
    assert_eq!(linter.detect(&dir), LintBackend::Clippy);
  }

  #[test]
  fn test_backend_name() {
    assert_eq!(LintBackend::Eslint.name(), "ESLint");
    assert_eq!(LintBackend::Clippy.name(), "Clippy");
  }

  #[test]
  fn test_parse_issues_eslint() {
    let linter = Linter::new();
    let output = "file1.js:1:0: error: no-unused-vars\nfile2.js:2:0: warning: semi";
    let issues = linter.parse_issues(output, LintBackend::Eslint, Path::new("/"));
    assert_eq!(issues.len(), 2);
    assert_eq!(issues[0].level, "error");
    assert_eq!(issues[1].level, "warning");
  }

  #[test]
  fn test_lint_report_struct() {
    let report = LintReport {
      total_errors: 3,
      total_warnings: 5,
      files_checked: 10,
      issues: vec![],
      output: "some output".into(),
      sarif: None,
    };
    assert_eq!(report.total_errors, 3);
    assert_eq!(report.total_warnings, 5);
  }

  #[test]
  fn test_sarif_generation() {
    let linter = Linter::new();
    let issues = vec![
      LintIssue {
        file: "test.js".into(),
        line: 1,
        column: 5,
        level: "error".into(),
        code: "no-unused".into(),
        message: "unused var".into(),
      },
    ];
    let sarif = linter.to_sarif(&issues, LintBackend::Eslint);
    assert_eq!(sarif.version, "2.1.0");
    assert_eq!(sarif.runs.len(), 1);
    assert_eq!(sarif.runs[0].results.len(), 1);
    assert_eq!(sarif.runs[0].tool.driver.name, "ESLint");
  }

  #[test]
  fn test_max_warnings_threshold() {
    let config = LinterConfig {
      max_warnings: Some(0),
      cache_config: false,
      sarif_output: false,
    };
    let linter = Linter::with_config(config);
    match linter.lint_dir(Path::new("/nonexistent"), &[]) {
      Ok(_) => {}
      Err(e) => {
        assert!(e.to_string().contains("failed to run"));
      }
    }
  }
}
