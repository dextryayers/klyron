use crate::report::BenchReport;
use crate::runner::BenchmarkRunner;
use crate::BenchConfig;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchComparison {
    pub label: String,
    pub baseline_avg: Duration,
    pub current_avg: Duration,
    pub change_pct: f64,
    pub regression: bool,
}

impl BenchComparison {
    pub fn formatted(&self) -> String {
        let arrow = if self.regression {
            "↑ REGRESSION"
        } else if self.change_pct < -1.0 {
            "↓ IMPROVEMENT"
        } else {
            "≈  unchanged "
        };
        format!(
            "{:30} | {:>12} -> {:>12} | {:>+7.2}% {}",
            self.label,
            fmt_duration(self.baseline_avg),
            fmt_duration(self.current_avg),
            self.change_pct,
            arrow,
        )
    }
}

#[derive(Debug, Clone)]
pub struct ComparisonReport {
    pub baseline: BenchReport,
    pub current: BenchReport,
    pub comparisons: Vec<BenchComparison>,
}

impl ComparisonReport {
    pub fn compare(baseline: &BenchReport, current: &BenchReport) -> Vec<BenchComparison> {
        let mut comps = Vec::new();
        for cur in &current.results {
            if let Some(b) = baseline.results.iter().find(|b| b.label == cur.label) {
                let base_avg = b.avg_time.as_secs_f64();
                let cur_avg = cur.avg_time.as_secs_f64();
                let change_pct = if base_avg > 0.0 {
                    ((cur_avg - base_avg) / base_avg) * 100.0
                } else {
                    0.0
                };
                comps.push(BenchComparison {
                    label: cur.label.clone(),
                    baseline_avg: b.avg_time,
                    current_avg: cur.avg_time,
                    change_pct,
                    regression: change_pct > 5.0,
                });
            }
        }
        comps
    }

    pub fn from_paths(baseline_path: &Path, current: BenchReport) -> Result<Self> {
        let content =
            std::fs::read_to_string(baseline_path).context("Failed to read baseline file")?;
        let baseline: BenchReport =
            serde_json::from_str(&content).context("Failed to parse baseline JSON")?;
        let comparisons = Self::compare(&baseline, &current);
        Ok(Self {
            baseline,
            current,
            comparisons,
        })
    }

    pub fn formatted(&self) -> String {
        let mut out = String::new();
        out.push_str("Comparison Report\n");
        out.push_str(&format!("{:-<100}\n", ""));
        for comp in &self.comparisons {
            out.push_str(&comp.formatted());
            out.push('\n');
        }
        out
    }

    pub fn print(&self) {
        println!("{}", self.formatted());
    }

    pub fn has_regressions(&self) -> bool {
        self.comparisons.iter().any(|c| c.regression)
    }

    pub fn regressions(&self) -> Vec<&BenchComparison> {
        self.comparisons.iter().filter(|c| c.regression).collect()
    }
}

pub fn compare_with_baseline(
    dir: &Path,
    config: &BenchConfig,
) -> Result<BenchReport> {
    let mut results = Vec::new();

    if let Ok(r) = BenchmarkRunner::run_micro("dir_read", &mut || {
        let _ = std::fs::read_dir(dir);
    }) {
        results.push(r);
    }

    if let Ok(r) = BenchmarkRunner::bench_memory() {
        results.push(r);
    }

    if let Ok(r) = BenchmarkRunner::bench_startup(10) {
        results.push(r);
    }

    let current = BenchReport::new(results);

    let baseline = if let Some(ref baseline_path) = config.baseline_path {
        if baseline_path.exists() {
            let content =
                std::fs::read_to_string(baseline_path).context("Failed to read baseline")?;
            Some(
                serde_json::from_str::<BenchReport>(&content)
                    .context("Failed to parse baseline")?,
            )
        } else {
            None
        }
    } else {
        None
    };

    let comparisons = if let Some(ref base) = baseline {
        ComparisonReport::compare(base, &current)
    } else {
        Vec::new()
    };

    let mut report = current;
    report.baseline = baseline.map(Box::new);
    report.comparisons = comparisons;

    if let Some(ref json_path) = config.json_output {
        let json = serde_json::to_string_pretty(&report)
            .context("Failed to serialize report")?;
        std::fs::write(json_path, json).context("Failed to write JSON output")?;
    }

    Ok(report)
}

fn fmt_duration(d: Duration) -> String {
    let secs = d.as_secs_f64();
    if secs >= 1.0 {
        format!("{:.3}s", secs)
    } else if secs >= 0.001 {
        format!("{:.3}ms", secs * 1000.0)
    } else if secs >= 0.000_001 {
        format!("{:.3}µs", secs * 1_000_000.0)
    } else {
        format!("{:.1}ns", secs * 1_000_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::BenchResult;
    use crate::BenchCategory;

    fn make_result(label: &str, avg_ns: u64) -> BenchResult {
        let avg = Duration::from_nanos(avg_ns);
        BenchResult {
            category: BenchCategory::Micro,
            label: label.into(),
            iterations: 100,
            warmup_iterations: 10,
            total_time: Duration::from_secs(1),
            avg_time: avg,
            min_time: avg,
            max_time: avg,
            median_time: avg,
            p50: avg,
            p95: avg,
            p99: avg,
            stddev: 0.0,
            ops_per_sec: 1_000_000_000.0 / avg_ns as f64,
            samples: vec![],
        }
    }

    #[test]
    fn test_compare_improvement() {
        let baseline = BenchReport::new(vec![make_result("test", 200)]);
        let current = BenchReport::new(vec![make_result("test", 100)]);
        let comps = ComparisonReport::compare(&baseline, &current);
        assert_eq!(comps.len(), 1);
        assert!((comps[0].change_pct - (-50.0)).abs() < 0.01);
        assert!(!comps[0].regression);
    }

    #[test]
    fn test_compare_regression() {
        let baseline = BenchReport::new(vec![make_result("test", 100)]);
        let current = BenchReport::new(vec![make_result("test", 200)]);
        let comps = ComparisonReport::compare(&baseline, &current);
        assert_eq!(comps.len(), 1);
        assert!(comps[0].change_pct > 5.0);
        assert!(comps[0].regression);
    }

    #[test]
    fn test_has_regressions() {
        let baseline = BenchReport::new(vec![make_result("a", 100), make_result("b", 50)]);
        let current = BenchReport::new(vec![make_result("a", 200), make_result("b", 45)]);
        let report = ComparisonReport {
            baseline: baseline.clone(),
            current: current.clone(),
            comparisons: ComparisonReport::compare(&baseline, &current),
        };
        assert!(report.has_regressions());
        assert_eq!(report.regressions().len(), 1);
    }
}
