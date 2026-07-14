//! klyron_linter — Klyron linter module

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

/// Supported lint backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// Human-readable name.
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

    /// The command and default arguments for this linter.
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
}

/// Report produced by a linter run.
#[derive(Debug, Clone)]
pub struct LintReport {
    pub total_errors: u64,
    pub total_warnings: u64,
    pub files_checked: u64,
    pub output: String,
}

/// Runs linters on a project directory.
#[derive(Debug, Default)]
pub struct Linter;

impl Linter {
    pub fn new() -> Self {
        Self
    }

    /// Auto-detect the linter backend used in `dir`.
    pub fn detect(dir: &Path) -> LintBackend {
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

    /// Run the linter on the entire project at `dir`.
    pub fn lint(dir: &Path) -> Result<LintReport> {
        let backend = Self::detect(dir);
        Self::run_linter(dir, &backend, &[])
    }

    /// Run the linter on a specific path relative to `dir`.
    pub fn lint_path(dir: &Path, path: &str) -> Result<LintReport> {
        let backend = Self::detect(dir);
        Self::run_linter(dir, &backend, &[path])
    }

    /// Run the linter with auto-fix enabled.
    pub fn lint_fix(dir: &Path) -> Result<LintReport> {
        let backend = Self::detect(dir);
        let fix_args = match backend {
            LintBackend::Eslint => &["--fix-dir", "."] as &[&str],
            LintBackend::Biome => &["lint", "--apply"],
            LintBackend::Clippy => &["clippy", "--fix", "--", "-D", "warnings"],
            LintBackend::Ruff => &["check", "--fix", "."],
            LintBackend::Rubocop => &["-a"],
            LintBackend::Golint => &["./..."],
            LintBackend::Pint => &[],
        };
        Self::run_linter(dir, &backend, fix_args)
    }

    /// Internal: run a linter and parse its output.
    fn run_linter(dir: &Path, backend: &LintBackend, extra_args: &[&str]) -> Result<LintReport> {
        let (program, base_args) = backend.command();
        let mut args: Vec<&str> = base_args.iter().copied().collect();
        if !extra_args.is_empty() {
            args = extra_args.to_vec();
        }

        let output = Command::new(program)
            .args(&args)
            .current_dir(dir)
            .output()
            .with_context(|| format!("failed to run {}", backend.name()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = format!("{stdout}\n{stderr}");

        let (errors, warnings, files) = Self::parse_counts(&combined, *backend);

        Ok(LintReport { total_errors: errors, total_warnings: warnings, files_checked: files, output: combined })
    }

    /// Parse error/warning/file counts from linter output.
    fn parse_counts(output: &str, backend: LintBackend) -> (u64, u64, u64) {
        match backend {
            LintBackend::Eslint | LintBackend::Biome => {
                let errors = output.matches("error").count() as u64;
                let warnings = output.matches("warning").count() as u64;
                let files = output.lines().filter(|l| l.contains(":")).count() as u64;
                (errors, warnings, files)
            }
            LintBackend::Clippy => {
                let errors = output.matches("error").count() as u64;
                let warnings = output.matches("warning").count() as u64;
                let files = output.lines().filter(|l| l.contains(":")).count() as u64;
                (errors, warnings, files)
            }
            LintBackend::Ruff => {
                let errors = output.matches("error").count() as u64;
                let warnings = output.matches("warning").count() as u64;
                let files = output.lines().filter(|l| l.contains(":")).count() as u64;
                (errors, warnings, files)
            }
            _ => {
                let errors = output.matches("error").count() as u64;
                let warnings = output.matches("warning").count() as u64;
                let files = output.lines().count() as u64;
                (errors, warnings, files)
            }
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
        let dir = test_dir();
        assert_eq!(Linter::detect(&dir), LintBackend::Clippy);
    }

    #[test]
    fn test_detect_eslint_fallback() {
        let dir = Path::new("/tmp");
        // /tmp has no config, falls back to ESLint
        assert_eq!(Linter::detect(dir), LintBackend::Eslint);
    }

    #[test]
    fn test_backend_name() {
        assert_eq!(LintBackend::Eslint.name(), "ESLint");
        assert_eq!(LintBackend::Clippy.name(), "Clippy");
        assert_eq!(LintBackend::Ruff.name(), "Ruff");
    }

    #[test]
    fn test_backend_command() {
        let (prog, args) = LintBackend::Clippy.command();
        assert_eq!(prog, "cargo");
        assert!(args.contains(&"clippy"));
    }

    #[test]
    fn test_parse_counts_eslint() {
        let output = "file1.js:1:0 error  no-unused-vars\nfile2.js:2:0 warning  semi";
        let (e, w, f) = Linter::parse_counts(output, LintBackend::Eslint);
        assert_eq!(e, 1);
        assert_eq!(w, 1);
        assert_eq!(f, 2);
    }

    #[test]
    fn test_parse_counts_empty() {
        let output = "";
        let (e, w, f) = Linter::parse_counts(output, LintBackend::Clippy);
        assert_eq!(e, 0);
        assert_eq!(w, 0);
        assert_eq!(f, 0);
    }

    #[test]
    fn test_lint_report_struct() {
        let report = LintReport {
            total_errors: 3,
            total_warnings: 5,
            files_checked: 10,
            output: "some output".into(),
        };
        assert_eq!(report.total_errors, 3);
        assert_eq!(report.total_warnings, 5);
        assert_eq!(report.files_checked, 10);
    }
}
