use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct LineCoverage {
    pub line_number: usize,
    pub executed: u64,
}

#[derive(Debug, Clone)]
pub struct BranchCoverage {
    pub line_number: usize,
    pub taken: u64,
    pub not_taken: u64,
}

#[derive(Debug, Clone)]
pub struct ModuleCoverage {
    pub module_name: String,
    pub source_file: String,
    pub lines: Vec<LineCoverage>,
    pub branches: Vec<BranchCoverage>,
}

pub struct CoverageTracker {
    modules: Mutex<HashMap<String, ModuleCoverage>>,
    enabled: Mutex<bool>,
}

impl CoverageTracker {
    pub fn new() -> Self {
        Self {
            modules: Mutex::new(HashMap::new()),
            enabled: Mutex::new(false),
        }
    }

    pub fn enable(&self) {
        *self.enabled.lock().unwrap() = true;
    }

    pub fn disable(&self) {
        *self.enabled.lock().unwrap() = false;
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }

    pub fn track_line(&self, module: &str, source_file: &str, line: usize) {
        if !self.is_enabled() {
            return;
        }
        let mut modules = self.modules.lock().unwrap();
        let coverage = modules.entry(module.to_string()).or_insert_with(|| ModuleCoverage {
            module_name: module.to_string(),
            source_file: source_file.to_string(),
            lines: Vec::new(),
            branches: Vec::new(),
        });

        if let Some(existing) = coverage.lines.iter_mut().find(|l| l.line_number == line) {
            existing.executed += 1;
        } else {
            coverage.lines.push(LineCoverage {
                line_number: line,
                executed: 1,
            });
        }
    }

    pub fn track_branch(&self, module: &str, source_file: &str, line: usize, taken: bool) {
        if !self.is_enabled() {
            return;
        }
        let mut modules = self.modules.lock().unwrap();
        let coverage = modules.entry(module.to_string()).or_insert_with(|| ModuleCoverage {
            module_name: module.to_string(),
            source_file: source_file.to_string(),
            lines: Vec::new(),
            branches: Vec::new(),
        });

        if let Some(existing) = coverage.branches.iter_mut().find(|b| b.line_number == line) {
            if taken {
                existing.taken += 1;
            } else {
                existing.not_taken += 1;
            }
        } else {
            coverage.branches.push(BranchCoverage {
                line_number: line,
                taken: if taken { 1 } else { 0 },
                not_taken: if taken { 0 } else { 1 },
            });
        }
    }

    pub fn export_lcov(&self) -> String {
        let modules = self.modules.lock().unwrap();
        let mut out = String::new();

        for coverage in modules.values() {
            out.push_str(&format!("SF:{}\n", coverage.source_file));
            for line in &coverage.lines {
                out.push_str(&format!("DA:{},{}\n", line.line_number, line.executed));
            }
            for branch in &coverage.branches {
                out.push_str(&format!("BRDA:{},0,0,{}\n", branch.line_number,
                    if branch.taken > 0 { branch.taken.to_string() } else { "-".to_string() }));
            }
            let line_count = coverage.lines.len();
            let hit_count = coverage.lines.iter().filter(|l| l.executed > 0).count();
            out.push_str(&format!("LF:{}\n", line_count));
            out.push_str(&format!("LH:{}\n", hit_count));
            out.push_str("end_of_record\n");
        }

        out
    }

    pub fn summary(&self) -> String {
        let modules = self.modules.lock().unwrap();
        let mut out = String::new();
        out.push_str("Coverage Summary:\n");
        out.push_str("================\n");

        for coverage in modules.values() {
            let total_lines = coverage.lines.len();
            let covered_lines = coverage.lines.iter().filter(|l| l.executed > 0).count();
            let pct = if total_lines > 0 {
                (covered_lines as f64 / total_lines as f64) * 100.0
            } else {
                0.0
            };
            out.push_str(&format!("  {}: {:.1}% ({}/{}) lines covered\n",
                coverage.module_name, pct, covered_lines, total_lines));
        }

        out
    }

    pub fn clear(&self) {
        self.modules.lock().unwrap().clear();
    }
}

impl Default for CoverageTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_tracker_new() {
        let tracker = CoverageTracker::new();
        assert!(!tracker.is_enabled());
    }

    #[test]
    fn test_coverage_tracker_enable_disable() {
        let tracker = CoverageTracker::new();
        tracker.enable();
        assert!(tracker.is_enabled());
        tracker.disable();
        assert!(!tracker.is_enabled());
    }

    #[test]
    fn test_track_line_disabled() {
        let tracker = CoverageTracker::new();
        tracker.track_line("mod", "file.js", 10);
        let summary = tracker.summary();
        assert!(summary.contains("0.0%"));
    }

    #[test]
    fn test_track_line_enabled() {
        let tracker = CoverageTracker::new();
        tracker.enable();
        tracker.track_line("mod", "file.js", 10);
        let summary = tracker.summary();
        assert!(summary.contains("100.0%"));
    }

    #[test]
    fn test_track_line_multiple_hits() {
        let tracker = CoverageTracker::new();
        tracker.enable();
        tracker.track_line("mod", "file.js", 10);
        tracker.track_line("mod", "file.js", 10);
        tracker.track_line("mod", "file.js", 20);
        let summary = tracker.summary();
        assert!(summary.contains("100.0%"));
    }

    #[test]
    fn test_track_branch_taken() {
        let tracker = CoverageTracker::new();
        tracker.enable();
        tracker.track_branch("mod", "file.js", 5, true);
        let summary = tracker.summary();
        assert!(summary.contains("0.0%"));
    }

    #[test]
    fn test_track_branch_not_taken() {
        let tracker = CoverageTracker::new();
        tracker.enable();
        tracker.track_branch("mod", "file.js", 5, false);
        let lcov = tracker.export_lcov();
        assert!(lcov.contains("BRDA:5,0,0,-"));
    }

    #[test]
    fn test_export_lcov_format() {
        let tracker = CoverageTracker::new();
        tracker.enable();
        tracker.track_line("mod", "src/file.js", 1);
        tracker.track_line("mod", "src/file.js", 2);
        tracker.track_branch("mod", "src/file.js", 5, true);
        let lcov = tracker.export_lcov();
        assert!(lcov.contains("SF:src/file.js"));
        assert!(lcov.contains("DA:1,1"));
        assert!(lcov.contains("DA:2,1"));
        assert!(lcov.contains("LF:2"));
        assert!(lcov.contains("LH:2"));
        assert!(lcov.contains("end_of_record"));
    }

    #[test]
    fn test_clear_coverage() {
        let tracker = CoverageTracker::new();
        tracker.enable();
        tracker.track_line("mod", "file.js", 1);
        tracker.clear();
        let summary = tracker.summary();
        assert!(!summary.contains("mod"));
    }

    #[test]
    fn test_multiple_modules() {
        let tracker = CoverageTracker::new();
        tracker.enable();
        tracker.track_line("mod_a", "a.js", 1);
        tracker.track_line("mod_b", "b.js", 1);
        let summary = tracker.summary();
        assert!(summary.contains("mod_a"));
        assert!(summary.contains("mod_b"));
    }

    #[test]
    fn test_branch_coverage_tracking() {
        let tracker = CoverageTracker::new();
        tracker.enable();
        tracker.track_branch("mod", "file.js", 10, true);
        tracker.track_branch("mod", "file.js", 10, false);
        tracker.track_branch("mod", "file.js", 10, true);
        let lcov = tracker.export_lcov();
        assert!(lcov.contains("BRDA:10,0,0,2"));
    }

    #[test]
    fn test_line_coverage_structure() {
        let line = LineCoverage { line_number: 42, executed: 5 };
        assert_eq!(line.line_number, 42);
        assert_eq!(line.executed, 5);
    }

    #[test]
    fn test_branch_coverage_structure() {
        let branch = BranchCoverage { line_number: 7, taken: 3, not_taken: 1 };
        assert_eq!(branch.line_number, 7);
        assert_eq!(branch.taken, 3);
        assert_eq!(branch.not_taken, 1);
    }

    #[test]
    fn test_module_coverage_structure() {
        let module = ModuleCoverage {
            module_name: "test".to_string(),
            source_file: "test.js".to_string(),
            lines: vec![LineCoverage { line_number: 1, executed: 1 }],
            branches: vec![],
        };
        assert_eq!(module.module_name, "test");
        assert_eq!(module.lines.len(), 1);
    }
}
