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
