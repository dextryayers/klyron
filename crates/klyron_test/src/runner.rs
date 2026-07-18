use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use chrono::Utc;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{TestBackend, TestCategory, TestResult, TestSuiteResult};

#[derive(Debug, Clone)]
pub struct TestRunnerConfig {
    pub parallel: bool,
    pub retry: u32,
    pub timeout: Option<Duration>,
    pub category: Option<TestCategory>,
    pub junit_output: Option<PathBuf>,
    pub shuffle: bool,
    pub verbose: bool,
}

impl Default for TestRunnerConfig {
    fn default() -> Self {
        TestRunnerConfig {
            parallel: true,
            retry: 0,
            timeout: None,
            category: None,
            junit_output: None,
            shuffle: false,
            verbose: false,
        }
    }
}

#[derive(Debug)]
pub struct TestRunner {
    config: TestRunnerConfig,
}

impl TestRunner {
    pub fn new() -> Self {
        TestRunner::default()
    }

    pub fn with_config(config: TestRunnerConfig) -> Self {
        TestRunner { config }
    }

    pub fn detect(dir: &Path) -> TestBackend {
        if dir.join("vitest.config.ts").exists() || dir.join("vitest.config.js").exists() {
            TestBackend::Vitest
        } else if dir.join("jest.config.ts").exists()
            || dir.join("jest.config.js").exists()
            || dir.join("jest.config.json").exists()
        {
            TestBackend::Jest
        } else if dir.join("phpunit.xml").exists() || dir.join("phpunit.xml.dist").exists() {
            let content = std::fs::read_to_string(dir.join("phpunit.xml"))
                .or_else(|_| std::fs::read_to_string(dir.join("phpunit.xml.dist")))
                .unwrap_or_default();
            if content.contains("pest") || content.contains("Pest") {
                TestBackend::Pest
            } else {
                TestBackend::PhpUnit
            }
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

    pub fn list_tests(dir: &Path) -> Result<Vec<String>> {
        let backend = Self::detect(dir);
        match backend {
            TestBackend::CargoTest => {
                let output = std::process::Command::new("cargo")
                    .args(["test", "--", "--list"])
                    .current_dir(dir)
                    .output()
                    .context("failed to list cargo tests")?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                Ok(stdout
                    .lines()
                    .filter(|l| l.contains(": test"))
                    .map(|l| l.trim_end_matches(": test").to_string())
                    .collect())
            }
            TestBackend::Vitest => {
                let output = std::process::Command::new("npx")
                    .args(["vitest", "run", "--list"])
                    .current_dir(dir)
                    .output()
                    .context("failed to list vitest tests")?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                Ok(stdout.lines().map(|l| l.to_string()).collect())
            }
            TestBackend::Pytest => {
                let output = std::process::Command::new("python")
                    .args(["-m", "pytest", "--collect-only", "-q"])
                    .current_dir(dir)
                    .output()
                    .context("failed to list pytest tests")?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                Ok(stdout
                    .lines()
                    .filter(|l| l.contains("::"))
                    .map(|l| l.trim().to_string())
                    .collect())
            }
            _ => Ok(Vec::new()),
        }
    }

    pub fn run(dir: &Path, filter: Option<&str>) -> Result<TestResult> {
        let runner = TestRunner::new();
        runner.run_internal(dir, filter, TestCategory::Unit)
    }

    pub fn run_with_config(&self, dir: &Path, filter: Option<&str>) -> Result<TestSuiteResult> {
        let categories = match self.config.category {
            Some(cat) => vec![cat],
            None => vec![
                TestCategory::Unit,
                TestCategory::Integration,
                TestCategory::E2e,
            ],
        };

        let results: Vec<TestResult> = if self.config.parallel {
            categories
                .into_par_iter()
                .map(|cat| {
                    let f = match cat {
                        TestCategory::Unit => filter.or(Some("unit")),
                        TestCategory::Integration => filter.or(Some("integration")),
                        TestCategory::E2e => filter.or(Some("e2e")),
                    };
                    self.run_internal(dir, f, cat).unwrap_or_else(|e| TestResult {
                        name: format!("{cat:?}"),
                        passed: 0,
                        failed: 1,
                        skipped: 0,
                        time: 0.0,
                        backend: TestRunner::detect(dir),
                        category: cat,
                        output: format!("{e:#}"),
                    })
                })
                .collect()
        } else {
            categories
                .into_iter()
                .map(|cat| {
                    let f = match cat {
                        TestCategory::Unit => filter.or(Some("unit")),
                        TestCategory::Integration => filter.or(Some("integration")),
                        TestCategory::E2e => filter.or(Some("e2e")),
                    };
                    self.run_internal(dir, f, cat).unwrap_or_else(|e| TestResult {
                        name: format!("{cat:?}"),
                        passed: 0,
                        failed: 1,
                        skipped: 0,
                        time: 0.0,
                        backend: TestRunner::detect(dir),
                        category: cat,
                        output: format!("{e:#}"),
                    })
                })
                .collect()
        };

        let total_time = results.iter().map(|r| r.time).sum();
        let total_passed = results.iter().map(|r| r.passed).sum();
        let total_failed = results.iter().map(|r| r.failed).sum();
        let total_skipped = results.iter().map(|r| r.skipped).sum();

        let suite = TestSuiteResult {
            name: dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            results,
            total_time,
            total_passed,
            total_failed,
            total_skipped,
        };

        if let Some(ref junit_path) = self.config.junit_output {
            let junit_xml = Self::generate_junit_xml(&suite)?;
            std::fs::write(junit_path, junit_xml).context("failed to write JUnit XML")?;
        }

        Ok(suite)
    }

    fn run_internal(
        &self,
        dir: &Path,
        filter: Option<&str>,
        category: TestCategory,
    ) -> Result<TestResult> {
        let backend = Self::detect(dir);
        let (program, mut args) = backend.command();

        if self.config.shuffle {
            match backend {
                TestBackend::CargoTest => args.push("--shuffle"),
                TestBackend::Pytest => {
                    args.push("--random-order");
                    args.push("-p");
                    args.push("randomly");
                }
                TestBackend::Vitest => args.push("--sequence.shuffle"),
                TestBackend::Jest => args.push("--randomize"),
                _ => {}
            }
        }

        if self.config.verbose {
            match backend {
                TestBackend::CargoTest => args.push("--nocapture"),
                TestBackend::Pytest => args.push("-v"),
                TestBackend::Vitest => args.push("--reporter=verbose"),
                TestBackend::Jest => args.push("--verbose"),
                _ => {}
            }
        }
        if let Some(f) = filter {
            match backend {
                TestBackend::CargoTest => args.push(f),
                TestBackend::Pytest => args.extend(["-k", f]),
                TestBackend::Jest | TestBackend::Vitest => args.extend(["-t", f]),
                _ => {}
            }
        }

        let max_retries = self.config.retry;

        for attempt in 0..=max_retries {
            let start = Instant::now();

            let mut child = std::process::Command::new(program)
                .args(&args)
                .current_dir(dir)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .with_context(|| format!("failed to run {} tests", backend.name()))?;

            if let Some(timeout) = self.config.timeout {
                let start_wait = Instant::now();
                loop {
                    match child.try_wait() {
                        Ok(Some(_status)) => {
                            let elapsed = start_wait.elapsed();
                            let output = child.wait_with_output()?;
                            return Ok(Self::process_output(output, backend, category, elapsed));
                        }
                        Ok(None) => {
                            if start_wait.elapsed() >= timeout {
                                child.kill()?;
                                anyhow::bail!("test timed out after {timeout:?}");
                            }
                            std::thread::sleep(Duration::from_millis(10));
                        }
                        Err(e) => anyhow::bail!("failed to wait for test process: {e}"),
                    }
                }
            } else {
                let output = child.wait_with_output()?;
                let elapsed = start.elapsed();
                let result = Self::process_output(output, backend, category, elapsed);

                if result.failed > 0 && attempt < max_retries {
                    eprintln!(
                        "retry {}/{} for {category:?} tests...",
                        attempt + 1,
                        max_retries
                    );
                    continue;
                }
                return Ok(result);
            };
        }

        anyhow::bail!("test execution failed after {max_retries} retries")
    }

    fn process_output(
        output: std::process::Output,
        backend: TestBackend,
        category: TestCategory,
        elapsed: Duration,
    ) -> TestResult {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = format!("{stdout}\n{stderr}");
        let (passed, failed, skipped) = Self::parse_counts(&combined, backend);
        TestResult {
            name: format!("{category:?}"),
            passed,
            failed,
            skipped,
            time: elapsed.as_secs_f64(),
            backend,
            category,
            output: combined,
        }
    }

    pub fn to_junit_xml(suite: &TestSuiteResult) -> Result<String> {
        Self::generate_junit_xml(suite)
    }

    fn generate_junit_xml(suite: &TestSuiteResult) -> Result<String> {
        let mut buffer = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buffer, b' ', 2);

        writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

        let mut testsuites = BytesStart::new("testsuites");
        testsuites.push_attribute(("name", suite.name.as_str()));
        testsuites.push_attribute(("tests", suite.results.len().to_string().as_str()));
        testsuites.push_attribute(("failures", suite.total_failed.to_string().as_str()));
        testsuites.push_attribute(("time", format!("{:.3}", suite.total_time).as_str()));
        writer.write_event(Event::Start(testsuites))?;

        for result in &suite.results {
            let mut testsuite = BytesStart::new("testsuite");
            testsuite.push_attribute(("name", result.name.as_str()));
            testsuite.push_attribute((
                "tests",
                (result.passed + result.failed + result.skipped)
                    .to_string()
                    .as_str(),
            ));
            testsuite.push_attribute(("failures", result.failed.to_string().as_str()));
            testsuite.push_attribute(("skipped", result.skipped.to_string().as_str()));
            testsuite.push_attribute(("time", format!("{:.3}", result.time).as_str()));
            testsuite.push_attribute(("timestamp", Utc::now().to_rfc3339().as_str()));
            writer.write_event(Event::Start(testsuite))?;

            let mut tc = BytesStart::new("testcase");
            tc.push_attribute((
                "name",
                format!("{}_{}", result.name, result.backend.name()).as_str(),
            ));
            tc.push_attribute(("classname", suite.name.as_str()));
            tc.push_attribute(("time", format!("{:.3}", result.time).as_str()));
            writer.write_event(Event::Start(tc))?;

            if result.failed > 0 {
                let mut failure = BytesStart::new("failure");
                failure.push_attribute(("message", "tests failed"));
                writer.write_event(Event::Start(failure))?;
                writer.write_event(Event::Text(BytesText::new(&result.output)))?;
                writer.write_event(Event::End(BytesEnd::new("failure")))?;
            }

            writer.write_event(Event::End(BytesEnd::new("testcase")))?;
            writer.write_event(Event::End(BytesEnd::new("testsuite")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("testsuites")))?;
        Ok(String::from_utf8(buffer)?)
    }

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
                let skipped =
                    output.matches("SKIP").count() as u64 + output.matches("○").count() as u64;
                (passed, failed, skipped)
            }
        }
    }

    fn extract_num(line: &str, label: &str) -> Option<u64> {
        let pattern = format!(" {label}");
        line.find(&pattern).and_then(|pos| {
            let before = &line[..pos].trim();
            before
                .split_whitespace()
                .last()
                .and_then(|s| s.parse().ok())
        })
    }

    pub fn run_watch(dir: &Path) -> Result<()> {
        let backend = Self::detect(dir);
        let (program, args) = backend.command();
        let mut cmd = std::process::Command::new(program);
        cmd.args(&args).current_dir(dir);
        match backend {
            TestBackend::Vitest => {
                cmd.arg("--watch");
            }
            TestBackend::Jest => {
                cmd.arg("--watchAll");
            }
            TestBackend::CargoTest => {
                cmd.arg("--");
                cmd.arg("--nocapture");
            }
            _ => {}
        }
        let status = cmd.status().with_context(|| "watch mode failed")?;
        if status.success() {
            Ok(())
        } else {
            anyhow::bail!("watch mode exited with status: {}", status)
        }
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        TestRunner {
            config: TestRunnerConfig::default(),
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
        let runner = TestRunner::new();
        let _ = runner;
    }

    #[test]
    fn test_detect_cargo() {
        let dir = test_dir();
        let backend = TestRunner::detect(&dir);
        assert_eq!(backend, TestBackend::CargoTest);
    }

    #[test]
    fn test_parse_counts_cargo() {
        let output =
            "test result: ok. 10 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out";
        let (p, f, s) = TestRunner::parse_counts(output, TestBackend::CargoTest);
        assert_eq!(p, 10);
        assert_eq!(f, 0);
        assert_eq!(s, 1);
    }

    #[test]
    fn test_extract_num() {
        let line = "test result: ok. 10 passed; 0 failed; 1 ignored";
        assert_eq!(TestRunner::extract_num(line, "passed"), Some(10));
        assert_eq!(TestRunner::extract_num(line, "failed"), Some(0));
        assert_eq!(TestRunner::extract_num(line, "ignored"), Some(1));
    }

    #[test]
    fn test_junit_generation() {
        let suite = TestSuiteResult {
            name: "test-suite".into(),
            results: vec![TestResult {
                name: "unit".into(),
                passed: 5,
                failed: 1,
                skipped: 0,
                time: 1.23,
                backend: TestBackend::CargoTest,
                category: TestCategory::Unit,
                output: "error log".into(),
            }],
            total_time: 1.23,
            total_passed: 5,
            total_failed: 1,
            total_skipped: 0,
        };
        let xml = TestRunner::to_junit_xml(&suite).unwrap();
        assert!(xml.contains("testsuites"));
        assert!(xml.contains("testcase"));
        assert!(xml.contains("failure"));
    }
}
