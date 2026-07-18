use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LintBackend {
    Eslint,
    Biome,
    Clippy,
    Ruff,
    Rubocop,
    Golint,
    Pint,
}

impl LintBackend {
    pub fn name(self) -> &'static str {
        match self {
            LintBackend::Eslint => "ESLint",
            LintBackend::Biome => "Biome",
            LintBackend::Clippy => "Clippy",
            LintBackend::Ruff => "Ruff",
            LintBackend::Rubocop => "RuboCop",
            LintBackend::Golint => "golint",
            LintBackend::Pint => "Pint",
        }
    }

    pub fn command(self) -> (&'static str, Vec<&'static str>) {
        match self {
            LintBackend::Eslint => ("npx", vec!["eslint", "."]),
            LintBackend::Biome => ("npx", vec!["biome", "lint"]),
            LintBackend::Clippy => ("cargo", vec!["clippy", "--all-targets", "--", "-D", "warnings"]),
            LintBackend::Ruff => ("ruff", vec!["check", "."]),
            LintBackend::Rubocop => ("rubocop", vec![]),
            LintBackend::Golint => ("golint", vec!["./..."]),
            LintBackend::Pint => ("./vendor/bin/pint", vec!["--test"]),
        }
    }

    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            LintBackend::Eslint | LintBackend::Biome => &["js", "jsx", "ts", "tsx", "mjs", "cjs"],
            LintBackend::Clippy => &["rs"],
            LintBackend::Ruff => &["py"],
            LintBackend::Rubocop => &["rb"],
            LintBackend::Golint => &["go"],
            LintBackend::Pint => &["php"],
        }
    }

    pub fn can_fix(self) -> bool {
        matches!(
            self,
            LintBackend::Eslint
                | LintBackend::Biome
                | LintBackend::Clippy
                | LintBackend::Ruff
                | LintBackend::Rubocop
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleCategory {
    Error,
    Warning,
    Style,
    Performance,
    Security,
    Complexity,
}

#[derive(Debug, Clone)]
pub struct LintRule {
    pub code: String,
    pub message: String,
    pub category: RuleCategory,
    pub auto_fixable: bool,
    pub severity: Severity,
}

impl LintRule {
    pub fn new(code: &str, message: &str, category: RuleCategory, auto_fixable: bool) -> Self {
        LintRule {
            code: code.to_string(),
            message: message.to_string(),
            category,
            auto_fixable,
            severity: Severity::Warning,
        }
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Off,
    Warning,
    Error,
}

#[derive(Debug, Default)]
pub struct RuleRegistry {
    rules: HashMap<String, Vec<LintRule>>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        let mut registry = RuleRegistry {
            rules: HashMap::new(),
        };
        registry.register_builtins();
        registry
    }

    pub fn register(&mut self, backend: LintBackend, rule: LintRule) {
        let key = format!("{}:{}", backend.name(), rule.code);
        self.rules
            .entry(backend.name().to_string())
            .or_default()
            .push(rule);
        let _ = key;
    }

    pub fn get_rules(&self, backend: LintBackend) -> &[LintRule] {
        self.rules
            .get(backend.name())
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn all_rules(&self) -> impl Iterator<Item = &LintRule> {
        self.rules.values().flatten()
    }

    fn register_builtins(&mut self) {
        let eslint_rules = vec![
            LintRule::new("no-unused-vars", "Unused variable detected", RuleCategory::Error, true),
            LintRule::new("no-undef", "Undeclared variable usage", RuleCategory::Error, false),
            LintRule::new("semi", "Missing semicolon", RuleCategory::Style, true),
            LintRule::new("quotes", "Incorrect quote style", RuleCategory::Style, true),
            LintRule::new("no-console", "Console statement detected", RuleCategory::Warning, true),
            LintRule::new("eqeqeq", "Use === instead of ==", RuleCategory::Style, true),
            LintRule::new("no-eval", "eval() is dangerous", RuleCategory::Security, false),
            LintRule::new("max-len", "Line too long", RuleCategory::Style, false),
            LintRule::new("comma-dangle", "Missing trailing comma", RuleCategory::Style, true),
            LintRule::new("no-redeclare", "Variable redeclared", RuleCategory::Error, false),
        ];
        for rule in eslint_rules {
            self.register(LintBackend::Eslint, rule);
        }

        let biome_rules = vec![
            LintRule::new("noUnusedVariables", "Unused variable", RuleCategory::Error, true),
            LintRule::new("noUndeclaredVariables", "Undeclared variable", RuleCategory::Error, false),
            LintRule::new("useSingleQuotes", "Use single quotes", RuleCategory::Style, true),
        ];
        for rule in biome_rules {
            self.register(LintBackend::Biome, rule);
        }

        let clippy_rules = vec![
            LintRule::new("unused_variable", "Unused variable", RuleCategory::Warning, true),
            LintRule::new("needless_return", "Needless return statement", RuleCategory::Style, true),
            LintRule::new("redundant_clone", "Redundant clone", RuleCategory::Performance, true),
            LintRule::new("unwrap_used", "unwrap() used", RuleCategory::Warning, false),
        ];
        for rule in clippy_rules {
            self.register(LintBackend::Clippy, rule);
        }

        let ruff_rules = vec![
            LintRule::new("F401", "Unused import", RuleCategory::Error, true),
            LintRule::new("E501", "Line too long", RuleCategory::Style, false),
        ];
        for rule in ruff_rules {
            self.register(LintBackend::Ruff, rule);
        }

        let rubocop_rules = vec![
            LintRule::new("Layout/LineLength", "Line too long", RuleCategory::Style, false),
            LintRule::new("Style/FrozenStringLiteralComment", "Missing frozen string literal", RuleCategory::Style, true),
        ];
        for rule in rubocop_rules {
            self.register(LintBackend::Rubocop, rule);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintIssue {
    pub file: String,
    pub line: u64,
    pub column: u64,
    pub level: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintReport {
    pub total_errors: u64,
    pub total_warnings: u64,
    pub files_checked: u64,
    pub issues: Vec<LintIssue>,
    pub output: String,
    pub sarif: Option<SarifReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifReport {
    pub version: String,
    pub runs: Vec<SarifRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRun {
    pub tool: SarifTool,
    pub results: Vec<SarifResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifTool {
    pub driver: SarifDriver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifDriver {
    pub name: String,
    pub semantic_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifResult {
    pub message: SarifMessage,
    pub level: String,
    pub locations: Vec<SarifLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifMessage {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifLocation {
    pub physical_location: SarifPhysicalLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifPhysicalLocation {
    pub artifact_location: SarifArtifactLocation,
    pub region: SarifRegion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifArtifactLocation {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRegion {
    pub start_line: u64,
    pub start_column: u64,
}
