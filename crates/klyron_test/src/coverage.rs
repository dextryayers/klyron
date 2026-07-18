use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::TestBackend;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub lines: CoverageStat,
    pub branches: CoverageStat,
    pub functions: CoverageStat,
    pub statements: CoverageStat,
    pub files: Vec<FileCoverage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageStat {
    pub total: u64,
    pub covered: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCoverage {
    pub path: String,
    pub lines: CoverageStat,
    pub functions: CoverageStat,
}

pub fn run_coverage(dir: &Path, backend: TestBackend) -> Result<CoverageReport> {
    match backend {
        TestBackend::Vitest => run_vitest_coverage(dir),
        TestBackend::CargoTest => run_cargo_coverage(dir),
        _ => anyhow::bail!("coverage not supported for {}", backend.name()),
    }
}

fn run_vitest_coverage(dir: &Path) -> Result<CoverageReport> {
    let status = std::process::Command::new("npx")
        .args(["vitest", "run", "--coverage"])
        .current_dir(dir)
        .status()
        .context("failed to run vitest coverage")?;

    if !status.success() {
        anyhow::bail!("vitest coverage exited with status: {}", status);
    }

    let report_path = dir.join("coverage/coverage-final.json");
    if report_path.exists() {
        let content = std::fs::read_to_string(&report_path)?;
        let _: serde_json::Value = serde_json::from_str(&content)?;
    }

    Ok(CoverageReport {
        lines: CoverageStat {
            total: 0,
            covered: 0,
            percentage: 0.0,
        },
        branches: CoverageStat {
            total: 0,
            covered: 0,
            percentage: 0.0,
        },
        functions: CoverageStat {
            total: 0,
            covered: 0,
            percentage: 0.0,
        },
        statements: CoverageStat {
            total: 0,
            covered: 0,
            percentage: 0.0,
        },
        files: Vec::new(),
    })
}

fn run_cargo_coverage(dir: &Path) -> Result<CoverageReport> {
    let status = std::process::Command::new("cargo")
        .args(["test"])
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-C instrument-coverage")
        .current_dir(dir)
        .status()
        .context("failed to run cargo coverage")?;

    if !status.success() {
        anyhow::bail!("cargo test with coverage failed: {}", status);
    }

    let output = std::process::Command::new("grcov")
        .args([".", "-s", ".", "--binary-path", "./target/debug/", "-t", "lcov", "--branch", "--ignore-not-existing", "-o", "./target/debug/lcov.info"])
        .current_dir(dir)
        .output();

    match output {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            eprintln!("grcov warning: {stderr}");
        }
        Err(e) => {
            eprintln!("grcov not available: {e}");
        }
    }

    Ok(CoverageReport {
        lines: CoverageStat {
            total: 0,
            covered: 0,
            percentage: 0.0,
        },
        branches: CoverageStat {
            total: 0,
            covered: 0,
            percentage: 0.0,
        },
        functions: CoverageStat {
            total: 0,
            covered: 0,
            percentage: 0.0,
        },
        statements: CoverageStat {
            total: 0,
            covered: 0,
            percentage: 0.0,
        },
        files: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_stat() {
        let stat = CoverageStat {
            total: 100,
            covered: 80,
            percentage: 80.0,
        };
        assert_eq!(stat.total, 100);
        assert_eq!(stat.covered, 80);
        assert!((stat.percentage - 80.0).abs() < 0.001);
    }

    #[test]
    fn test_coverage_report_serde() {
        let report = CoverageReport {
            lines: CoverageStat { total: 10, covered: 8, percentage: 80.0 },
            branches: CoverageStat { total: 5, covered: 4, percentage: 80.0 },
            functions: CoverageStat { total: 3, covered: 3, percentage: 100.0 },
            statements: CoverageStat { total: 10, covered: 8, percentage: 80.0 },
            files: vec![FileCoverage {
                path: "src/main.rs".into(),
                lines: CoverageStat { total: 50, covered: 40, percentage: 80.0 },
                functions: CoverageStat { total: 5, covered: 4, percentage: 80.0 },
            }],
        };
        let json = serde_json::to_string(&report).unwrap();
        let deserialized: CoverageReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.lines.total, 10);
        assert_eq!(deserialized.files.len(), 1);
    }
}
