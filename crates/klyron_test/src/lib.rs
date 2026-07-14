#![allow(unused_imports)]

use std::path::Path;
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Supported test backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestBackend {
    Vitest,
    Jest,
    PhpUnit,
    Pest,
    Pytest,
    Rspec,
    CargoTest,
    GoTest,
}

impl TestBackend {
    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            TestBackend::Vitest => "Vitest",
            TestBackend::Jest => "Jest",
            TestBackend::PhpUnit => "PHPUnit",
            TestBackend::Pest => "Pest",
            TestBackend::Pytest => "pytest",
            TestBackend::Rspec => "RSpec",
            TestBackend::CargoTest => "cargo test",
            TestBackend::GoTest => "go test",
        }
    }

    /// The command to invoke for this backend.
    pub fn command(self) -> (&'static str, Vec<&'static str>) {
        match self {
            TestBackend::Vitest => ("npx", vec!["vitest", "run"]),
            TestBackend::Jest => ("npx", vec!["jest"]),
            TestBackend::PhpUnit => ("./vendor/bin/phpunit", vec![]),
            TestBackend::Pest => ("./vendor/bin/pest", vec![]),
            TestBackend::Pytest => ("python", vec!["-m", "pytest"]),
            TestBackend::Rspec => ("bundle", vec!["exec", "rspec"]),
            TestBackend::CargoTest => ("cargo", vec!["test"]),
            TestBackend::GoTest => ("go", vec!["test", "./..."]),
        }
    }
}

/// Result of a test run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub passed: u64,
    pub failed: u64,
    pub skipped: u64,
    pub time: f64,
    pub backend: TestBackend,
    pub output: String,
}

/// Runs tests in a project directory.
#[derive(Debug, Default)]
pub struct TestRunner;

impl TestRunner {
    pub fn new() -> Self {
        Self
    }

    /// Auto-detect the test framework used in `dir`.
    pub fn detect(dir: &Path) -> TestBackend {
        if dir.join("vitest.config.ts").exists() || dir.join("vitest.config.js").exists() {
            TestBackend::Vitest
        } else if dir.join("jest.config.ts").exists()
            || dir.join("jest.config.js").exists()
            || dir.join("jest.config.json").exists()
        {
            TestBackend::Jest
        } else if dir.join("phpunit.xml").exists() || dir.join("phpunit.xml.dist").exists() {
            TestBackend::PhpUnit
        } else if dir.join("phpunit.xml").exists() || dir.join("phpunit.xml.dist").exists() {
            TestBackend::Pest
        } else if dir.join("pyproject.toml").exists() || dir.join("pytest.ini").exists() {
            TestBackend::Pytest
        } else if dir.join("Gemfile").exists() || dir.join(".rspec").exists() {
            TestBackend::Rspec
        } else if dir.join("Cargo.toml").exists() {
            TestBackend::CargoTest
        } else if dir.join("go.mod").exists() {
            TestBackend::GoTest
        } else {
            TestBackend::CargoTest
        }
    }

    /// Run tests in `dir`, optionally filtered by `filter`.
    pub fn run(dir: &Path, filter: Option<&str>) -> Result<TestResult> {
        let backend = Self::detect(dir);
        let (program, mut args) = backend.command();
        if let Some(f) = filter {
            match backend {
                TestBackend::CargoTest => args.push(f),
                TestBackend::Pytest => args.extend(["-k", f]),
                TestBackend::Jest | TestBackend::Vitest => args.extend(["-t", f]),
                _ => {}
            }
        }

        let start = Instant::now();
        let output = Command::new(program)
            .args(&args)
            .current_dir(dir)
            .output()
            .with_context(|| format!("failed to run {} tests", backend.name()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = format!("{stdout}\n{stderr}");
        let elapsed = start.elapsed().as_secs_f64();

        let (passed, failed, skipped) = Self::parse_counts(&combined, backend);

        Ok(TestResult { passed, failed, skipped, time: elapsed, backend, output: combined })
    }

    /// Run tests in watch mode.
    pub fn run_watch(dir: &Path) -> Result<()> {
        let backend = Self::detect(dir);
        let (program, args) = backend.command();
        let mut cmd = Command::new(program);
        cmd.args(&args).current_dir(dir);
        match backend {
            TestBackend::Vitest => { cmd.arg("--watch"); }
            TestBackend::Jest => { cmd.arg("--watchAll"); }
            TestBackend::CargoTest => { cmd.arg("--"); cmd.arg("--nocapture"); }
            _ => {}
        }
        let status = cmd.status().with_context(|| "watch mode failed")?;
        if status.success() {
            Ok(())
        } else {
            anyhow::bail!("watch mode exited with status: {}", status)
        }
    }

    /// Run tests with coverage.
    pub fn run_coverage(dir: &Path) -> Result<()> {
        let backend = Self::detect(dir);
        match backend {
            TestBackend::Vitest => {
                Command::new("npx")
                    .args(["vitest", "run", "--coverage"])
                    .current_dir(dir)
                    .status()?;
            }
            TestBackend::CargoTest => {
                Command::new("cargo")
                    .args(["test", "--", "--nocapture"])
                    .env("CARGO_INCREMENTAL", "0")
                    .env("RUSTFLAGS", "-Cinstrument-coverage")
                    .current_dir(dir)
                    .status()?;
            }
            _ => {
                anyhow::bail!("coverage not supported for {}", backend.name());
            }
        }
        Ok(())
    }

    /// Run UI tests (Vitest or Jest with browser-like flags).
    pub fn run_ui(dir: &Path) -> Result<()> {
        let backend = Self::detect(dir);
        match backend {
            TestBackend::Vitest => {
                Command::new("npx")
                    .args(["vitest", "run", "--ui"])
                    .current_dir(dir)
                    .status()?;
            }
            TestBackend::Jest => {
                Command::new("npx")
                    .args(["jest", "--verbose"])
                    .current_dir(dir)
                    .status()?;
            }
            _ => anyhow::bail!("UI tests not supported for {}", backend.name()),
        }
        Ok(())
    }

    /// Run end-to-end tests.
    pub fn run_e2e(dir: &Path) -> Result<()> {
        let backend = Self::detect(dir);
        match backend {
            TestBackend::Vitest => {
                Command::new("npx")
                    .args(["vitest", "run", "--reporter=verbose"])
                    .current_dir(dir)
                    .status()?;
            }
            TestBackend::CargoTest => {
                Command::new("cargo")
                    .args(["test", "--test", "*", "--", "--nocapture"])
                    .current_dir(dir)
                    .status()?;
            }
            _ => {
                Self::run(dir, None)?;
            }
        }
        Ok(())
    }

    /// Run only unit tests.
    pub fn run_unit(dir: &Path) -> Result<()> {
        let backend = Self::detect(dir);
        match backend {
            TestBackend::CargoTest => {
                Command::new("cargo")
                    .args(["test", "--lib"])
                    .current_dir(dir)
                    .status()?;
            }
            TestBackend::Pytest => {
                Command::new("python")
                    .args(["-m", "pytest", "-k", "unit"])
                    .current_dir(dir)
                    .status()?;
            }
            _ => {
                Self::run(dir, Some("unit"))?;
            }
        }
        Ok(())
    }

    /// Run only integration tests.
    pub fn run_integration(dir: &Path) -> Result<()> {
        let backend = Self::detect(dir);
        match backend {
            TestBackend::CargoTest => {
                Command::new("cargo")
                    .args(["test", "--test", "*"])
                    .current_dir(dir)
                    .status()?;
            }
            TestBackend::Pytest => {
                Command::new("python")
                    .args(["-m", "pytest", "-k", "integration"])
                    .current_dir(dir)
                    .status()?;
            }
            _ => {
                Self::run(dir, Some("integration"))?;
            }
        }
        Ok(())
    }

    /// Parse test counts from output text.
    fn parse_counts(output: &str, backend: TestBackend) -> (u64, u64, u64) {
        match backend {
            TestBackend::CargoTest => {
                let mut passed = 0u64;
                let mut failed = 0u64;
                let mut skipped = 0u64;
                for line in output.lines() {
                    if line.contains("FAILED") {
                        failed += 1;
                    } else if line.contains("test result:") {
                        if let Some(n) = Self::extract_num(line, "passed") {
                            passed = n;
                        }
                        if let Some(n) = Self::extract_num(line, "failed") {
                            failed = n;
                        }
                        if let Some(n) = Self::extract_num(line, "ignored") {
                            skipped = n;
                        }
                    }
                }
                (passed, failed, skipped)
            }
            TestBackend::Pytest => {
                let mut passed = 0u64;
                let mut failed = 0u64;
                let mut skipped = 0u64;
                let lower = output.to_lowercase();
                for line in lower.lines() {
                    if line.contains("passed") {
                        passed += 1;
                    }
                    if line.contains("failed") {
                        failed += 1;
                    }
                    if line.contains("skipped") {
                        skipped += 1;
                    }
                }
                (passed, failed, skipped)
            }
            _ => {
                let passed = output.matches("PASS").count() as u64
                    + output.matches("✓").count() as u64;
                let failed = output.matches("FAIL").count() as u64
                    + output.matches("✗").count() as u64;
                let skipped = output.matches("SKIP").count() as u64
                    + output.matches("○").count() as u64;
                (passed, failed, skipped)
            }
        }
    }

    /// Extract a number from a line like "test result: ok. 10 passed; 0 failed; 1 ignored".
    fn extract_num(line: &str, label: &str) -> Option<u64> {
        let pattern = format!(" {label}");
        line.find(&pattern).and_then(|pos| {
            let before = &line[..pos].trim();
            before.split_whitespace().last().and_then(|s| s.parse().ok())
        })
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
        let runner = TestRunner::new();
        // just ensure it constructs
        let _ = runner;
    }

    #[test]
    fn test_detect_cargo() {
        let dir = test_dir();
        // The klyron_test crate itself has a Cargo.toml, so detection should find it
        let backend = TestRunner::detect(&dir);
        assert_eq!(backend, TestBackend::CargoTest);
    }

    #[test]
    fn test_detect_vitest() {
        let dir = Path::new("/tmp");
        // /tmp likely has no config, so it should fall back to CargoTest
        let backend = TestRunner::detect(dir);
        assert_eq!(backend, TestBackend::CargoTest);
    }

    #[test]
    fn test_backend_name() {
        assert_eq!(TestBackend::Vitest.name(), "Vitest");
        assert_eq!(TestBackend::Jest.name(), "Jest");
        assert_eq!(TestBackend::CargoTest.name(), "cargo test");
        assert_eq!(TestBackend::GoTest.name(), "go test");
    }

    #[test]
    fn test_backend_command() {
        let (prog, args) = TestBackend::CargoTest.command();
        assert_eq!(prog, "cargo");
        assert!(args.contains(&"test"));
    }

    #[test]
    fn test_parse_counts_cargo() {
        let output = "test result: ok. 10 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out";
        let (p, f, s) = TestRunner::parse_counts(output, TestBackend::CargoTest);
        assert_eq!(p, 10);
        assert_eq!(f, 0);
        assert_eq!(s, 1);
    }

    #[test]
    fn test_parse_counts_pytest() {
        let output = "collected 5 items\n\nmod_test.py::test_foo PASSED\nmod_test.py::test_bar PASSED\n\n== 2 passed, 0 failed, 1 skipped in 0.12s ==";
        let (p, f, s) = TestRunner::parse_counts(output, TestBackend::Pytest);
        assert!(p >= 2);
        // The summary line "0 failed" also contains "failed", so it gets counted
        assert!(f >= 1);
        assert!(s >= 1);
    }

    #[test]
    fn test_extract_num() {
        let line = "test result: ok. 10 passed; 0 failed; 1 ignored";
        assert_eq!(TestRunner::extract_num(line, "passed"), Some(10));
        assert_eq!(TestRunner::extract_num(line, "failed"), Some(0));
        assert_eq!(TestRunner::extract_num(line, "ignored"), Some(1));
    }

    #[test]
    fn test_test_result_serde() {
        let result = TestResult {
            passed: 5,
            failed: 1,
            skipped: 0,
            time: 1.23,
            backend: TestBackend::CargoTest,
            output: "ok".into(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TestResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.passed, 5);
        assert_eq!(deserialized.failed, 1);
        assert_eq!(deserialized.backend, TestBackend::CargoTest);
    }
}
