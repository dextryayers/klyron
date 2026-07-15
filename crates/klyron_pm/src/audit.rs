use crate::{LockfileV3, PmError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub severity: Severity,
    pub package: String,
    pub version_range: String,
    pub title: String,
    pub description: String,
    pub references: Vec<String>,
    pub cvss_score: f64,
    pub patched_versions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    None,
}

impl Severity {
    pub fn from_cvss(score: f64) -> Self {
        match score {
            s if s >= 9.0 => Severity::Critical,
            s if s >= 7.0 => Severity::High,
            s if s >= 4.0 => Severity::Medium,
            s if s >= 0.1 => Severity::Low,
            _ => Severity::None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Critical => "CRITICAL",
            Self::High => "HIGH",
            Self::Medium => "MEDIUM",
            Self::Low => "LOW",
            Self::None => "NONE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub total: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub findings: Vec<PackageFinding>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageFinding {
    pub package: String,
    pub version: String,
    pub vulnerabilities: Vec<Vulnerability>,
    pub highest_severity: Severity,
}

pub struct AuditDatabase {
    advisories: HashMap<String, Vec<Vulnerability>>,
}

impl AuditDatabase {
    pub fn new() -> Self {
        Self {
            advisories: Self::load_advisories(),
        }
    }

    pub fn check_package(&self, name: &str, version: &str) -> Vec<Vulnerability> {
        let mut result = Vec::new();
        let pkg_key = name.to_lowercase();
        if let Some(advisories) = self.advisories.get(&pkg_key) {
            for vuln in advisories {
                if version_matches_range(version, &vuln.version_range) {
                    result.push(vuln.clone());
                }
            }
        }
        result
    }

    pub fn audit_lockfile(&self, lockfile: &LockfileV3) -> AuditReport {
        let mut findings = Vec::new();
        let mut total = 0;
        let mut critical = 0;
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;

        for (path, pkg) in lockfile.all_packages() {
            let name = path
                .split("node_modules/")
                .last()
                .unwrap_or(path)
                .split('/')
                .next()
                .unwrap_or(path);
            let vulns = self.check_package(name, &pkg.version);
            if !vulns.is_empty() {
                let highest = vulns
                    .iter()
                    .map(|v| v.severity)
                    .max()
                    .unwrap_or(Severity::None);
                findings.push(PackageFinding {
                    package: name.to_string(),
                    version: pkg.version.clone(),
                    vulnerabilities: vulns,
                    highest_severity: highest,
                });
                total += 1;
                for v in &findings.last().unwrap().vulnerabilities {
                    match v.severity {
                        Severity::Critical => critical += 1,
                        Severity::High => high += 1,
                        Severity::Medium => medium += 1,
                        Severity::Low => low += 1,
                        Severity::None => {}
                    }
                }
            }
        }

        let summary = format!(
            "Audit complete: {} packages checked, {} vulnerabilities found ({} critical, {} high, {} medium, {} low)",
            lockfile.packages.len(),
            total,
            critical,
            high,
            medium,
            low,
        );

        AuditReport {
            total,
            critical,
            high,
            medium,
            low,
            findings,
            summary,
        }
    }

    fn load_advisories() -> HashMap<String, Vec<Vulnerability>> {
        let mut db = HashMap::new();

        db.insert(
            "lodash".to_string(),
            vec![Vulnerability {
                id: "GHSA-p6mc-m468-83gw".into(),
                severity: Severity::Critical,
                package: "lodash".into(),
                version_range: "<4.17.21".into(),
                title: "Prototype Pollution in lodash".into(),
                description: "Versions of lodash prior to 4.17.21 are vulnerable to prototype pollution via the zipObjectDeep, assignMergeValue, and set functions.".into(),
                references: vec!["https://github.com/advisories/GHSA-p6mc-m468-83gw".into()],
                cvss_score: 9.1,
                patched_versions: vec!["4.17.21".into()],
            }],
        );

        db.insert(
            "minimist".to_string(),
            vec![Vulnerability {
                id: "CVE-2020-7598".into(),
                severity: Severity::High,
                package: "minimist".into(),
                version_range: "<1.2.3".into(),
                title: "Prototype Pollution in minimist".into(),
                description: "minimist before 1.2.3 could be tricked into adding or modifying properties of Object.prototype using a constructor or __proto__ payload.".into(),
                references: vec!["https://nvd.nist.gov/vuln/detail/CVE-2020-7598".into()],
                cvss_score: 7.8,
                patched_versions: vec!["1.2.3".into(), "0.2.4".into()],
            }],
        );

        db.insert(
            "axios".to_string(),
            vec![
                Vulnerability {
                    id: "CVE-2023-45857".into(),
                    severity: Severity::High,
                    package: "axios".into(),
                    version_range: "<1.6.0".into(),
                    title: "Server-Side Request Forgery in axios".into(),
                    description: "An attacker can cause a redirect to be followed cross-origin, leading to SSRF.".into(),
                    references: vec!["https://nvd.nist.gov/vuln/detail/CVE-2023-45857".into()],
                    cvss_score: 7.5,
                    patched_versions: vec!["1.6.0".into()],
                },
                Vulnerability {
                    id: "CVE-2024-23338".into(),
                    severity: Severity::Medium,
                    package: "axios".into(),
                    version_range: ">=1.3.0 <1.6.8".into(),
                    title: "Prototype Pollution in axios".into(),
                    description: "axios before 1.6.8 is vulnerable to prototype pollution via the merge function.".into(),
                    references: vec!["https://nvd.nist.gov/vuln/detail/CVE-2024-23338".into()],
                    cvss_score: 5.5,
                    patched_versions: vec!["1.6.8".into()],
                },
            ],
        );

        db
    }
}

impl Default for AuditDatabase {
    fn default() -> Self {
        Self::new()
    }
}

fn version_matches_range(version: &str, range: &str) -> bool {
    use semver::{Version, VersionReq};
    let ver = match Version::parse(version.trim_start_matches('^').trim_start_matches('~').trim_start_matches('=')) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let req = match VersionReq::parse(range.trim()) {
        Ok(r) => r,
        Err(_) => {
            if range.starts_with('<') || range.starts_with('>') || range.starts_with('=') {
                let cleaned = range.trim();
                let cmp = if cleaned.starts_with("<=") { Some("<=") }
                    else if cleaned.starts_with(">=") { Some(">=") }
                    else if cleaned.starts_with('<') { Some("<") }
                    else if cleaned.starts_with('>') { Some(">") }
                    else if cleaned.starts_with("==") { Some("==") }
                    else if cleaned.starts_with('=') { Some("=") }
                    else { None };
                if let Some(op) = cmp {
                    let v_str = cleaned.trim_start_matches(op).trim();
                    if let Ok(other) = Version::parse(v_str) {
                        return match op {
                            "<" => ver < other,
                            "<=" => ver <= other,
                            ">" => ver > other,
                            ">=" => ver >= other,
                            "==" | "=" => ver == other,
                            _ => false,
                        };
                    }
                }
                false
            } else {
                false
            }
        }
    };
    req.matches(&ver)
}

pub fn generate_report(report: &AuditReport, format: &str) -> String {
    match format {
        "json" => serde_json::to_string_pretty(report).unwrap_or_default(),
        "table" | _ => {
            let mut out = String::new();
            out.push_str(&format!("\n=== Audit Report ===\n\n{}", report.summary));
            out.push_str(&format!("\n  Total:      {}", report.total));
            out.push_str(&format!("\n  Critical:   {}", report.critical));
            out.push_str(&format!("\n  High:       {}", report.high));
            out.push_str(&format!("\n  Medium:     {}", report.medium));
            out.push_str(&format!("\n  Low:        {}", report.low));
            out.push_str("\n\n  Findings:\n");
            for finding in &report.findings {
                out.push_str(&format!("\n    {}@{} [{}]", finding.package, finding.version, finding.highest_severity.as_str()));
                for vuln in &finding.vulnerabilities {
                    out.push_str(&format!("\n      - [{}] {}: {}", vuln.id, vuln.severity.as_str(), vuln.title));
                    out.push_str(&format!("\n        Description: {}", vuln.description));
                    out.push_str(&format!("\n        CVSS Score: {}", vuln.cvss_score));
                    out.push_str(&format!("\n        Patched in: {}", vuln.patched_versions.join(", ")));
                }
            }
            out
        }
    }
}
