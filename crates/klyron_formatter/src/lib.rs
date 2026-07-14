//! klyron_formatter — Klyron formatter module

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

/// Supported formatting backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatBackend {
    Prettier,
    Biome,
    Rustfmt,
    Gofmt,
    Black,
    Rubocop,
    Pint,
}

impl FormatBackend {
    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            FormatBackend::Prettier => "Prettier",
            FormatBackend::Biome => "Biome",
            FormatBackend::Rustfmt => "rustfmt",
            FormatBackend::Gofmt => "gofmt",
            FormatBackend::Black => "Black",
            FormatBackend::Rubocop => "RuboCop",
            FormatBackend::Pint => "Pint",
        }
    }

    /// The command and default arguments for this formatter.
    pub fn command(self) -> (&'static str, Vec<&'static str>) {
        match self {
            FormatBackend::Prettier => ("npx", vec!["prettier", "--check", "."]),
            FormatBackend::Biome => ("npx", vec!["biome", "format"]),
            FormatBackend::Rustfmt => ("cargo", vec!["fmt", "--check"]),
            FormatBackend::Gofmt => ("gofmt", vec!["-l", "."]),
            FormatBackend::Black => ("black", vec!["--check", "."]),
            FormatBackend::Rubocop => ("rubocop", vec!["-a", "--only", "Layout"]),
            FormatBackend::Pint => ("./vendor/bin/pint", vec!["--test"]),
        }
    }
}

/// Report produced by a formatter run.
#[derive(Debug, Clone)]
pub struct FormatReport {
    pub files_changed: u64,
    pub files_unchanged: u64,
    pub output: String,
}

/// Formats source code in a project directory.
#[derive(Debug, Default)]
pub struct Formatter;

impl Formatter {
    pub fn new() -> Self {
        Self
    }

    /// Auto-detect the formatter backend used in `dir`.
    pub fn detect(dir: &Path) -> FormatBackend {
        if dir.join(".prettierrc").exists()
            || dir.join(".prettierrc.json").exists()
            || dir.join(".prettierrc.yaml").exists()
            || dir.join(".prettierrc.yml").exists()
            || dir.join(".prettierrc.js").exists()
            || dir.join("prettier.config.js").exists()
        {
            FormatBackend::Prettier
        } else if dir.join("biome.json").exists() {
            FormatBackend::Biome
        } else if dir.join("Cargo.toml").exists() {
            FormatBackend::Rustfmt
        } else if dir.join("go.mod").exists() {
            FormatBackend::Gofmt
        } else if dir.join("pyproject.toml").exists() {
            FormatBackend::Black
        } else if dir.join(".rubocop.yml").exists() || dir.join(".rubocop.yaml").exists() {
            FormatBackend::Rubocop
        } else if dir.join("pint.json").exists() {
            FormatBackend::Pint
        } else {
            FormatBackend::Prettier
        }
    }

    /// Check formatting without modifying files.
    pub fn format_check(dir: &Path) -> Result<FormatReport> {
        let backend = Self::detect(dir);
        Self::run_formatter(dir, &backend, false)
    }

    /// Format a specific path.
    pub fn format_path(dir: &Path, path: &str) -> Result<FormatReport> {
        let backend = Self::detect(dir);
        let (program, _) = backend.command();
        let args = match backend {
            FormatBackend::Prettier => vec!["prettier", "--write", path],
            FormatBackend::Biome => vec!["biome", "format", "--write", path],
            FormatBackend::Rustfmt => vec!["fmt", path],
            FormatBackend::Gofmt => vec!["-w", path],
            FormatBackend::Black => vec!["black", path],
            FormatBackend::Rubocop => vec!["-a", "--only", "Layout", path],
            FormatBackend::Pint => vec![path],
        };
        let output = Command::new(program)
            .args(&args)
            .current_dir(dir)
            .output()
            .with_context(|| format!("failed to format path {path}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = format!("{stdout}\n{stderr}");
        Ok(FormatReport { files_changed: 1, files_unchanged: 0, output: combined })
    }

    /// Format files in-place.
    pub fn format_write(dir: &Path) -> Result<FormatReport> {
        let backend = Self::detect(dir);
        Self::run_formatter(dir, &backend, true)
    }

    /// Internal: run a formatter and produce a report.
    fn run_formatter(dir: &Path, backend: &FormatBackend, write: bool) -> Result<FormatReport> {
        let (program, base_args) = backend.command();
        let mut args: Vec<&str> = Vec::new();

        if write {
            match backend {
                FormatBackend::Prettier => args.extend(["prettier", "--write", "."]),
                FormatBackend::Biome => args.extend(["biome", "format", "--write"]),
                FormatBackend::Rustfmt => args.extend(["fmt"]),
                FormatBackend::Gofmt => args.extend(["-w", "."]),
                FormatBackend::Black => args.extend(["black", "."]),
                FormatBackend::Rubocop => args.extend(["-a"]),
                FormatBackend::Pint => {} // no extra args needed
            }
        } else {
            args = base_args.iter().copied().collect();
        }

        let output = Command::new(program)
            .args(&args)
            .current_dir(dir)
            .output()
            .with_context(|| format!("failed to run {}", backend.name()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = format!("{stdout}\n{stderr}");

        let (changed, unchanged) = Self::parse_counts(&combined, *backend, write);

        Ok(FormatReport { files_changed: changed, files_unchanged: unchanged, output: combined })
    }

    /// Parse file counts from formatter output.
    fn parse_counts(output: &str, _backend: FormatBackend, wrote: bool) -> (u64, u64) {
        if wrote {
            let changed = output.lines().filter(|l| !l.is_empty()).count() as u64;
            (changed, 0)
        } else {
            let changed = output.lines().filter(|l| l.contains("would have been") || l.contains("Formatter would")).count() as u64;
            let unchanged = output.lines().filter(|l| l.contains("already formatted") || l.contains("no changes")).count() as u64;
            (changed, unchanged)
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
        let f = Formatter::new();
        let _ = f;
    }

    #[test]
    fn test_detect_rustfmt() {
        let dir = test_dir();
        assert_eq!(Formatter::detect(&dir), FormatBackend::Rustfmt);
    }

    #[test]
    fn test_detect_prettier_fallback() {
        let dir = Path::new("/tmp");
        assert_eq!(Formatter::detect(dir), FormatBackend::Prettier);
    }

    #[test]
    fn test_backend_name() {
        assert_eq!(FormatBackend::Prettier.name(), "Prettier");
        assert_eq!(FormatBackend::Rustfmt.name(), "rustfmt");
        assert_eq!(FormatBackend::Black.name(), "Black");
    }

    #[test]
    fn test_backend_command() {
        let (prog, args) = FormatBackend::Rustfmt.command();
        assert_eq!(prog, "cargo");
        assert!(args.contains(&"fmt"));
    }

    #[test]
    fn test_parse_counts_write() {
        let (c, u) = Formatter::parse_counts("file1.rs\nfile2.rs\n", FormatBackend::Rustfmt, true);
        assert_eq!(c, 2);
        assert_eq!(u, 0);
    }

    #[test]
    fn test_parse_counts_check() {
        let output = "file1.rs would have been formatted\nfile2.rs already formatted";
        let (c, u) = Formatter::parse_counts(output, FormatBackend::Prettier, false);
        assert_eq!(c, 1);
        assert_eq!(u, 1);
    }

    #[test]
    fn test_format_report_struct() {
        let report = FormatReport {
            files_changed: 3,
            files_unchanged: 7,
            output: "done".into(),
        };
        assert_eq!(report.files_changed, 3);
        assert_eq!(report.files_unchanged, 7);
    }
}
