pub mod compare;
pub mod engine;
pub mod lockfile;
pub mod report;
pub mod runner;

pub use compare::{compare_with_baseline, BenchComparison, ComparisonReport};
pub use report::{BenchReport, BenchResult};
pub use runner::{percentile, BenchmarkRunner};

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum BenchCategory {
    Runtime,
    Http,
    Memory,
    Startup,
    Micro,
}

#[derive(Debug, Clone)]
pub struct BenchConfig {
    pub warmup_iterations: u64,
    pub min_iterations: u64,
    pub min_duration: Duration,
    pub baseline_path: Option<std::path::PathBuf>,
    pub json_output: Option<std::path::PathBuf>,
    pub criterion_mode: bool,
    pub batch_size: u64,
}

impl Default for BenchConfig {
    fn default() -> Self {
        BenchConfig {
            warmup_iterations: 100,
            min_iterations: 1000,
            min_duration: Duration::from_secs(2),
            baseline_path: None,
            json_output: None,
            criterion_mode: false,
            batch_size: 100,
        }
    }
}

impl BenchConfig {
    pub fn with_warmup(mut self, n: u64) -> Self {
        self.warmup_iterations = n;
        self
    }

    pub fn with_iterations(mut self, n: u64) -> Self {
        self.min_iterations = n;
        self
    }

    pub fn with_duration(mut self, d: Duration) -> Self {
        self.min_duration = d;
        self
    }

    pub fn with_baseline(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.baseline_path = Some(path.into());
        self
    }

    pub fn with_json_output(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.json_output = Some(path.into());
        self
    }
}
