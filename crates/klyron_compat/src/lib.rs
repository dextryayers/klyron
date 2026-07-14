/// Compatibility checker for Klyron — detects framework usage, version mismatches, and compat issues.
use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Overall compatibility status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompatStatus {
    Compatible,
    Partial,
    Incompatible,
    Unknown,
}

/// A single compatibility check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatCheck {
    pub name: String,
    pub status: CompatStatus,
    pub message: String,
}

/// Framework or runtime target for compatibility checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameworkTarget {
    React,
    Next,
    Astro,
    Nest,
    Prisma,
    Node,
}

/// Full compatibility report for a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatReport {
    pub framework: String,
    pub version: Option<String>,
    pub checks: Vec<CompatCheck>,
    pub overall: CompatStatus,
    pub summary: String,
}

/// Compatibility checker.
pub struct CompatChecker;

impl CompatChecker {
    /// Create a new `CompatChecker`.
    pub fn new() -> Self {
        Self
    }

    /// Run a general compatibility check on `dir`.
    pub fn check_project(dir: &Path) -> Result<CompatReport> {
        let pkg = Self::read_package_json(dir)?;
        let mut checks = Vec::new();

        // Check module format
        let module_type = pkg
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("commonjs");
        checks.push(CompatCheck {
            name: "module_format".into(),
            status: CompatStatus::Compatible,
            message: format!("Project uses {module_type} module format"),
        });

        // Detect framework
        let (framework, version) = Self::detect_framework(&pkg);
        checks.push(CompatCheck {
            name: "framework_detection".into(),
            status: CompatStatus::Compatible,
            message: format!("Detected framework: {framework:?}"),
        });

        // Check engines field
        if let Some(engines) = pkg.get("engines").and_then(|v| v.as_object()) {
            for (key, val) in engines {
                let range = val.as_str().unwrap_or("*");
                let status = Self::check_semver_range(range);
                checks.push(CompatCheck {
                    name: format!("engine_{key}"),
                    status,
                    message: format!("Engine {key} requires {range}"),
                });
            }
        }

        // Check for ESM/CJS interop issues
        let has_exports = pkg.get("exports").is_some();
        let has_main = pkg.get("main").is_some();
        if has_exports && !has_main {
            checks.push(CompatCheck {
                name: "exports_field".into(),
                status: CompatStatus::Compatible,
                message: "Package uses 'exports' field (modern resolution)".into(),
            });
        }

        // Check for native Node.js API usage patterns by scanning source files
        let native_checks = Self::check_native_usage(dir);
        checks.extend(native_checks);

        // Build final summary
        let overall = Self::aggregate_status(&checks);
        let summary = format!(
            "Found {} checks: {} compatible, {} partial, {} incompatible",
            checks.len(),
            checks.iter().filter(|c| c.status == CompatStatus::Compatible).count(),
            checks.iter().filter(|c| c.status == CompatStatus::Partial).count(),
            checks.iter().filter(|c| c.status == CompatStatus::Incompatible).count(),
        );

        Ok(CompatReport {
            framework,
            version,
            checks,
            overall,
            summary,
        })
    }

    /// Run compatibility check for a specific framework target.
    pub fn check_framework(dir: &Path, target: FrameworkTarget) -> Result<CompatReport> {
        let pkg = Self::read_package_json(dir)?;
        let mut checks = Vec::new();
        let framework_name = format!("{target:?}");

        let (detected_framework, version) = Self::detect_framework(&pkg);
        let has_framework = detected_framework == framework_name;

        checks.push(CompatCheck {
            name: format!("{target:?}_detected"),
            status: if has_framework {
                CompatStatus::Compatible
            } else {
                CompatStatus::Incompatible
            },
            message: if has_framework {
                format!("{target:?} detected at version {:?}", version.as_deref().unwrap_or("unknown"))
            } else {
                format!("{target:?} not found in dependencies")
            },
        });

        // Framework-specific checks
        match target {
            FrameworkTarget::Next => {
                // Check for pages or app directory
                let has_pages = dir.join("pages").is_dir();
                let has_app = dir.join("app").is_dir();
                if has_pages || has_app {
                    checks.push(CompatCheck {
                        name: "route_structure".into(),
                        status: CompatStatus::Compatible,
                        message: "Next.js route structure detected".into(),
                    });
                }
            }
            FrameworkTarget::React => {
                let has_jsx = Self::glob_has_files(dir, &["*.jsx", "*.tsx", "*.js"]);
                checks.push(CompatCheck {
                    name: "jsx_files".into(),
                    status: CompatStatus::Compatible,
                    message: if has_jsx {
                        "JSX/TSX files found".into()
                    } else {
                        "No JSX/TSX files found".into()
                    },
                });
            }
            FrameworkTarget::Astro => {
                let has_astro = Self::glob_has_files(dir, &["*.astro"]);
                checks.push(CompatCheck {
                    name: "astro_files".into(),
                    status: if has_astro { CompatStatus::Compatible } else { CompatStatus::Unknown },
                    message: if has_astro {
                        ".astro files found".into()
                    } else {
                        "No .astro files found".into()
                    },
                });
            }
            FrameworkTarget::Nest => {
                let has_nest = Self::glob_has_files(dir, &["*.module.ts", "*.controller.ts"]);
                checks.push(CompatCheck {
                    name: "nest_structure".into(),
                    status: if has_nest { CompatStatus::Compatible } else { CompatStatus::Unknown },
                    message: if has_nest {
                        "NestJS module/controller structure detected".into()
                    } else {
                        "No NestJS structure detected".into()
                    },
                });
            }
            FrameworkTarget::Prisma => {
                let has_schema = dir.join("prisma/schema.prisma").exists();
                checks.push(CompatCheck {
                    name: "prisma_schema".into(),
                    status: if has_schema { CompatStatus::Compatible } else { CompatStatus::Unknown },
                    message: if has_schema {
                        "Prisma schema found".into()
                    } else {
                        "No prisma/schema.prisma found".into()
                    },
                });
            }
            FrameworkTarget::Node => {
                // Node is always compatible
                checks.push(CompatCheck {
                    name: "node_runtime".into(),
                    status: CompatStatus::Compatible,
                    message: "Node.js runtime is always compatible".into(),
                });
            }
        }

        let overall = Self::aggregate_status(&checks);
        let summary = format!(
            "Framework check for {target:?}: {overall:?} — {} checks performed",
            checks.len()
        );

        Ok(CompatReport {
            framework: framework_name,
            version: version.or_else(|| {
                pkg.get("version").and_then(|v| v.as_str()).map(String::from)
            }),
            checks,
            overall,
            summary,
        })
    }

    /// Check Node.js API compatibility by scanning for common Node built-in usage.
    pub fn check_node_compat(dir: &Path) -> Result<CompatReport> {
        let mut checks = Vec::new();

        let node_apis = [
            ("fs", "fs"),
            ("path", "path"),
            ("os", "os"),
            ("crypto", "crypto"),
            ("http", "http"),
            ("https", "https"),
            ("child_process", "child_process"),
            ("stream", "stream"),
            ("events", "events"),
            ("buffer", "buffer"),
        ];

        for (name, module) in &node_apis {
            let pattern = format!("require('{module}')");
            let pattern2 = format!("from '{module}'");
            let pattern3 = format!("from \"{module}\"");

            let found = Self::search_files(dir, &[&pattern, &pattern2, &pattern3]);
            checks.push(CompatCheck {
                name: format!("node_{name}"),
                status: if found { CompatStatus::Partial } else { CompatStatus::Compatible },
                message: if found {
                    format!("Uses Node.js '{module}' module — may need polyfill")
                } else {
                    format!("No usage of '{module}' detected")
                },
            });
        }

        // Check ESM/CJS format
        let pkg = Self::read_package_json(dir).ok();
        let module_type = pkg
            .as_ref()
            .and_then(|p| p.get("type").and_then(|v| v.as_str()))
            .unwrap_or("commonjs")
            .to_string();
        let esm_import = Self::search_files(dir, &["import ", "export "]);
        checks.push(CompatCheck {
            name: "module_syntax".into(),
            status: CompatStatus::Compatible,
            message: format!(
                "Package type: {module_type}, ESM syntax detected: {esm_import}"
            ),
        });

        let overall = Self::aggregate_status(&checks);
        let summary = format!(
            "Node.js compat check: {overall:?} — {} API checks performed",
            checks.len()
        );

        Ok(CompatReport {
            framework: "Node.js".into(),
            version: None,
            checks,
            overall,
            summary,
        })
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn read_package_json(dir: &Path) -> Result<serde_json::Value> {
        let path = dir.join("package.json");
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))
    }

    fn detect_framework(pkg: &serde_json::Value) -> (String, Option<String>) {
        let deps = Self::merge_deps(pkg);
        let frameworks: &[(&str, &[&str])] = &[
            ("Next", &["next"]),
            ("React", &["react"]),
            ("Astro", &["astro"]),
            ("Nest", &["@nestjs/core"]),
            ("Prisma", &["@prisma/client", "prisma"]),
            ("Node", &[]),
        ];

        for (name, packages) in frameworks {
            for pkg_name in *packages {
                if let Some(ver) = deps.get(*pkg_name) {
                    return (name.to_string(), Some(ver.clone()));
                }
            }
        }

        ("Node".into(), None)
    }

    fn merge_deps(pkg: &serde_json::Value) -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();
        if let Some(deps) = pkg.get("dependencies").and_then(|v| v.as_object()) {
            for (k, v) in deps {
                map.insert(k.clone(), v.as_str().unwrap_or("").to_string());
            }
        }
        if let Some(deps) = pkg.get("devDependencies").and_then(|v| v.as_object()) {
            for (k, v) in deps {
                map.insert(k.clone(), v.as_str().unwrap_or("").to_string());
            }
        }
        map
    }

    fn check_semver_range(range: &str) -> CompatStatus {
        let range = range.trim();
        if range == "*" || range == "x" {
            return CompatStatus::Compatible;
        }
        if let Some(caret) = range.strip_prefix('^') {
            let major: u64 = caret.split('.').next().unwrap_or("0").parse().unwrap_or(0);
            if major >= 18 {
                return CompatStatus::Compatible;
            }
            return CompatStatus::Partial;
        }
        if let Some(tilde) = range.strip_prefix('~') {
            let parts: Vec<&str> = tilde.split('.').collect();
            if parts.len() >= 2 {
                if let Ok(major) = parts[0].parse::<u64>() {
                    if major >= 18 {
                        return CompatStatus::Compatible;
                    }
                }
            }
        }
        if let Some(ver) = range.strip_prefix(">=") {
            let major: u64 = ver.trim().split('.').next().unwrap_or("0").parse().unwrap_or(0);
            if major >= 18 {
                return CompatStatus::Compatible;
            }
        }
        CompatStatus::Partial
    }

    fn check_native_usage(dir: &Path) -> Vec<CompatCheck> {
        let mut checks = Vec::new();
        let native_patterns = [
            ("napi", "napi"),
            ("node-gyp", "node-gyp"),
            ("bindings", "bindings"),
            ("ffi", "ffi"),
            ("neon", "neon"),
        ];

        for (name, keyword) in &native_patterns {
            let found = Self::search_files(dir, &[keyword]);
            checks.push(CompatCheck {
                name: format!("native_{name}"),
                status: if found {
                    CompatStatus::Partial
                } else {
                    CompatStatus::Compatible
                },
                message: if found {
                    format!("Native '{name}' usage detected — may require native build tools")
                } else {
                    format!("No native '{name}' usage detected")
                },
            });
        }
        checks
    }

    fn search_files(dir: &Path, patterns: &[&str]) -> bool {
        let walker = match dir.read_dir() {
            Ok(entries) => entries,
            Err(_) => return false,
        };
        for entry in walker.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path
                    .file_name()
                    .is_some_and(|n| n != "node_modules" && n != ".git" && n != "target")
                {
                    if Self::search_files(&path, patterns) {
                        return true;
                    }
                }
            } else if let Some(ext) = path.extension() {
                if matches!(ext.to_str(), Some("js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs")) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        for pat in patterns {
                            if content.contains(pat) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn glob_has_files(dir: &Path, _patterns: &[&str]) -> bool {
        // Simple directory scan for matching extensions
        let walker = match dir.read_dir() {
            Ok(entries) => entries,
            Err(_) => return false,
        };
        for entry in walker.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if Self::glob_has_files(&path, _patterns) {
                    return true;
                }
            } else if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy();
                for pat in _patterns {
                    let pat_ext = pat.trim_start_matches("*.");
                    if ext_str == pat_ext {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn aggregate_status(checks: &[CompatCheck]) -> CompatStatus {
        let mut has_incompatible = false;
        let mut has_partial = false;
        let mut has_unknown = false;
        let mut has_compatible = false;

        for c in checks {
            match c.status {
                CompatStatus::Incompatible => has_incompatible = true,
                CompatStatus::Partial => has_partial = true,
                CompatStatus::Unknown => has_unknown = true,
                CompatStatus::Compatible => has_compatible = true,
            }
        }

        if has_incompatible {
            CompatStatus::Incompatible
        } else if has_partial {
            CompatStatus::Partial
        } else if has_unknown && !has_compatible {
            CompatStatus::Unknown
        } else {
            CompatStatus::Compatible
        }
    }
}

impl Default for CompatChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("klyron_compat_test_{}_{}", std::process::id(), id));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_package_json(dir: &Path, content: &str) {
        fs::write(dir.join("package.json"), content).unwrap();
    }

    #[test]
    fn test_check_project_compatible() {
        let dir = temp_dir();
        write_package_json(
            &dir,
            r#"{"name":"test","type":"module","dependencies":{"next":"^14.0.0"}}"#,
        );
        let report = CompatChecker::check_project(&dir).expect("Check failed");
        assert_eq!(report.framework, "Next");
        assert_eq!(report.overall, CompatStatus::Compatible);
    }

    #[test]
    fn test_check_framework_react() {
        let dir = temp_dir();
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/App.jsx"), "import React from 'react'").unwrap();
        write_package_json(
            &dir,
            r#"{"name":"test","dependencies":{"react":"^18.2.0"}}"#,
        );
        let report = CompatChecker::check_framework(&dir, FrameworkTarget::React)
            .expect("Check failed");
        assert_eq!(report.framework, "React");
    }

    #[test]
    fn test_check_node_compat() {
        let dir = temp_dir();
        let src = dir.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("server.js"), "const fs = require('fs')").unwrap();
        write_package_json(&dir, r#"{"name":"test"}"#);

        let report = CompatChecker::check_node_compat(&dir).expect("Check failed");
        // Should detect fs usage as Partial
        let fs_check = report.checks.iter().find(|c| c.name == "node_fs");
        assert!(fs_check.is_some());
        assert_eq!(fs_check.unwrap().status, CompatStatus::Partial);
    }

    #[test]
    fn test_framework_detection_none() {
        let dir = temp_dir();
        write_package_json(&dir, r#"{"name":"test"}"#);
        let report = CompatChecker::check_project(&dir).expect("Check failed");
        assert_eq!(report.framework, "Node");
    }

    #[test]
    fn test_serialization() {
        let report = CompatReport {
            framework: "React".into(),
            version: Some("18.2.0".into()),
            checks: vec![CompatCheck {
                name: "test".into(),
                status: CompatStatus::Compatible,
                message: "All good".into(),
            }],
            overall: CompatStatus::Compatible,
            summary: "OK".into(),
        };
        let json = serde_json::to_string(&report).unwrap();
        let back: CompatReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.framework, "React");
    }
}
