pub mod assert;
pub mod assertions;
pub mod coverage;
pub mod mock;
pub mod property;
pub mod runner;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub use assert::*;
pub use coverage::*;
pub use mock::*;
pub use property::*;
pub use runner::*;

pub fn discover_js_test_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let patterns = &[
        "**/*.test.js",
        "**/*.test.jsx",
        "**/*.test.ts",
        "**/*.test.tsx",
        "**/*.spec.js",
        "**/*.spec.jsx",
        "**/*.spec.ts",
        "**/*.spec.tsx",
    ];
    for pattern in patterns {
        if let Ok(glob_results) = glob::glob(&dir.join(pattern).to_string_lossy()) {
            for entry in glob_results.flatten() {
                files.push(entry);
            }
        }
    }
    files.sort();
    files.dedup();
    files
}

pub fn has_test_script(dir: &Path) -> bool {
    let pkg = dir.join("package.json");
    let content = match std::fs::read_to_string(pkg) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if let Some(scripts) = json.get("scripts").and_then(|s| s.as_object()) {
        if let Some(test) = scripts.get("test").and_then(|v| v.as_str()) {
            if !test.is_empty() && test != "echo \"Error: no test specified\"" {
                return true;
            }
        }
    }
    false
}

pub fn has_test_framework_dep(dir: &Path) -> bool {
    let frameworks = ["vitest", "jest", "mocha", "ava", "tape", "uvu", "node:test"];
    let pkg = dir.join("package.json");
    let content = match std::fs::read_to_string(pkg) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return false,
    };
    for section in &["dependencies", "devDependencies"] {
        if let Some(deps) = json.get(section).and_then(|d| d.as_object()) {
            for fw in &frameworks {
                if deps.contains_key(*fw) {
                    return true;
                }
            }
        }
    }
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestCategory {
    Unit,
    Integration,
    E2e,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub passed: u64,
    pub failed: u64,
    pub skipped: u64,
    pub time: f64,
    pub backend: TestBackend,
    pub category: TestCategory,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteResult {
    pub name: String,
    pub results: Vec<TestResult>,
    pub total_time: f64,
    pub total_passed: u64,
    pub total_failed: u64,
    pub total_skipped: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn test_backend_name() {
        assert_eq!(TestBackend::Vitest.name(), "Vitest");
        assert_eq!(TestBackend::CargoTest.name(), "cargo test");
    }

    #[test]
    fn test_detect_cargo() {
        let dir = test_dir();
        let backend = TestRunner::detect(&dir);
        assert_eq!(backend, TestBackend::CargoTest);
    }

    #[test]
    fn test_test_result_serde() {
        let result = TestResult {
            name: "test".into(),
            passed: 5,
            failed: 1,
            skipped: 0,
            time: 1.23,
            backend: TestBackend::CargoTest,
            category: TestCategory::Unit,
            output: "ok".into(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TestResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.passed, 5);
        assert_eq!(deserialized.failed, 1);
    }
}
