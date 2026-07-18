use crate::rules::{LintBackend, LintIssue, LintReport};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct FixSuggestion {
    pub file: String,
    pub line: u64,
    pub column: u64,
    pub code: String,
    pub replacement: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct FixResult {
    pub fixed_count: usize,
    pub failed_count: usize,
    pub fixes: Vec<FixSuggestion>,
    pub report: Option<LintReport>,
}

pub struct AutoFix {
    #[allow(dead_code)]
    backend: LintBackend,
}

impl AutoFix {
    pub fn new(backend: LintBackend) -> Self {
        AutoFix { backend }
    }

    pub fn apply_fix_suggestions(&self, suggestions: &[FixSuggestion]) -> Result<FixResult> {
        let mut fixed_count = 0;
        let mut failed_count = 0;

        for suggestion in suggestions {
            match self.apply_single_fix(suggestion) {
                Ok(true) => fixed_count += 1,
                Ok(false) => {}
                Err(_) => failed_count += 1,
            }
        }

        Ok(FixResult {
            fixed_count,
            failed_count,
            fixes: suggestions.to_vec(),
            report: None,
        })
    }

    fn apply_single_fix(&self, suggestion: &FixSuggestion) -> Result<bool> {
        use std::io::Write;

        let path = std::path::Path::new(&suggestion.file);
        if !path.exists() {
            return Ok(false);
        }

        let content = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();

        if suggestion.line == 0 || suggestion.line as usize > lines.len() {
            return Ok(false);
        }

        let line_idx = (suggestion.line - 1) as usize;
        let old_line = lines[line_idx];

        if suggestion.column as usize <= old_line.len() {
            let mut new_line = old_line.to_string();
            if !suggestion.replacement.is_empty() {
                new_line = suggestion.replacement.clone();
            }

            let mut output = String::new();
            for (i, line) in lines.iter().enumerate() {
                if i == line_idx {
                    output.push_str(&new_line);
                } else {
                    output.push_str(line);
                }
                output.push('\n');
            }

            let mut file = std::fs::File::create(path)?;
            file.write_all(output.as_bytes())?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn fix_report(report: &mut LintReport) -> Vec<FixSuggestion> {
        let mut suggestions = Vec::new();

        for issue in &report.issues {
            if let Some(fix) = Self::suggest_fix(issue) {
                suggestions.push(fix);
            }
        }

        suggestions
    }

    fn suggest_fix(issue: &LintIssue) -> Option<FixSuggestion> {
        match issue.code.as_str() {
            "semi" => Some(FixSuggestion {
                file: issue.file.clone(),
                line: issue.line,
                column: issue.column,
                code: issue.code.clone(),
                replacement: String::new(),
                description: "Add missing semicolon".to_string(),
            }),
            "no-unused-vars" | "F401" => Some(FixSuggestion {
                file: issue.file.clone(),
                line: issue.line,
                column: issue.column,
                code: issue.code.clone(),
                replacement: String::new(),
                description: "Remove unused variable or import".to_string(),
            }),
            "no-console" => Some(FixSuggestion {
                file: issue.file.clone(),
                line: issue.line,
                column: issue.column,
                code: issue.code.clone(),
                replacement: String::new(),
                description: "Remove console statement".to_string(),
            }),
            "needless_return" => Some(FixSuggestion {
                file: issue.file.clone(),
                line: issue.line,
                column: issue.column,
                code: issue.code.clone(),
                replacement: String::new(),
                description: "Remove needless return".to_string(),
            }),
            _ => None,
        }
    }
}

pub fn run_auto_fix(backend: LintBackend, issues: &[LintIssue]) -> Result<FixResult> {
    let auto_fix = AutoFix::new(backend);
    let suggestions: Vec<FixSuggestion> = issues
        .iter()
        .filter_map(|issue| AutoFix::suggest_fix(issue))
        .collect();
    auto_fix.apply_fix_suggestions(&suggestions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::LintIssue;

    #[test]
    fn test_auto_fix_new() {
        let fixer = AutoFix::new(LintBackend::Eslint);
        assert_eq!(fixer.backend, LintBackend::Eslint);
    }

    #[test]
    fn test_suggest_fix_semi() {
        let issue = LintIssue {
            file: "test.js".into(),
            line: 1,
            column: 10,
            level: "error".into(),
            code: "semi".into(),
            message: "Missing semicolon".into(),
        };
        let fix = AutoFix::suggest_fix(&issue);
        assert!(fix.is_some());
        assert_eq!(fix.unwrap().code, "semi");
    }

    #[test]
    fn test_suggest_fix_unused_var() {
        let issue = LintIssue {
            file: "test.js".into(),
            line: 2,
            column: 5,
            level: "warning".into(),
            code: "no-unused-vars".into(),
            message: "Unused variable".into(),
        };
        let fix = AutoFix::suggest_fix(&issue);
        assert!(fix.is_some());
    }

    #[test]
    fn test_suggest_fix_unknown() {
        let issue = LintIssue {
            file: "test.js".into(),
            line: 1,
            column: 1,
            level: "error".into(),
            code: "unknown-rule".into(),
            message: "Some error".into(),
        };
        let fix = AutoFix::suggest_fix(&issue);
        assert!(fix.is_none());
    }

    #[test]
    fn test_fix_report_empty_issues() {
        let mut report = LintReport {
            total_errors: 0,
            total_warnings: 0,
            files_checked: 1,
            issues: vec![],
            output: String::new(),
            sarif: None,
        };
        let fixes = AutoFix::fix_report(&mut report);
        assert!(fixes.is_empty());
    }

    #[test]
    fn test_run_auto_fix_empty() {
        let result = run_auto_fix(LintBackend::Eslint, &[]).unwrap();
        assert_eq!(result.fixed_count, 0);
        assert_eq!(result.failed_count, 0);
    }
}
