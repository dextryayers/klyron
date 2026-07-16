use std::collections::{BTreeMap, HashMap};
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

pub mod autoloader;
pub mod composer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompatStatus {
  Compatible,
  Partial,
  Incompatible,
  Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatCheck {
  pub name: String,
  pub status: CompatStatus,
  pub message: String,
  pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameworkTarget {
  React,
  Next,
  Astro,
  Nest,
  Prisma,
  Node,
  Vue,
  Svelte,
  Angular,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSupport {
  pub browser: String,
  pub version: String,
  pub supported: bool,
  pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatReport {
  pub framework: String,
  pub version: Option<String>,
  pub checks: Vec<CompatCheck>,
  pub overall: CompatStatus,
  pub summary: String,
  pub browser_matrix: Vec<BrowserSupport>,
  pub suggestions: Vec<String>,
}

struct CaniuseData {
  browsers: HashMap<String, Vec<String>>,
}

impl CaniuseData {
  fn fetch() -> Result<Self> {
    let url = "https://raw.githubusercontent.com/Fyrd/caniuse/main/data.json";
    let response = ureq::get(url)
      .call()
      .context("failed to fetch caniuse data")?;
    let mut body = String::new();
    response.into_body().into_reader().read_to_string(&mut body)
      .context("failed to read caniuse data")?;
    let json: serde_json::Value = serde_json::from_str(&body)
      .context("failed to parse caniuse data")?;

    let mut browsers = HashMap::new();
    if let Some(data) = json.get("data").and_then(|d| d.as_object()) {
      for (_key, _feature) in data {
        if let Some(stats) = _feature.get("stats").and_then(|s| s.as_object()) {
          for (browser, versions) in stats {
            let versions_str = versions.as_str().unwrap_or("");
            let supported: Vec<String> = versions_str
              .split(',')
              .filter(|v| v.contains("y"))
              .map(|v| v.trim().to_string())
              .collect();
            browsers.entry(browser.clone()).or_insert_with(Vec::new).extend(supported);
          }
        }
      }
    }

    Ok(CaniuseData { browsers })
  }

  fn check_feature(&self, _feature: &str) -> Vec<BrowserSupport> {
    let mut matrix = Vec::new();
    for (browser, versions) in &self.browsers {
      if let Some(latest) = versions.last() {
        matrix.push(BrowserSupport {
          browser: browser.clone(),
          version: latest.clone(),
          supported: true,
          notes: None,
        });
      }
    }
    matrix
  }
}

pub struct CompatChecker {
  #[allow(dead_code)]
  cache_dir: Option<std::path::PathBuf>,
}

impl CompatChecker {
  pub fn new() -> Self {
    CompatChecker { cache_dir: None }
  }

  pub fn with_cache(dir: std::path::PathBuf) -> Self {
    CompatChecker { cache_dir: Some(dir) }
  }

  pub fn check_project(dir: &Path) -> Result<CompatReport> {
    let checker = CompatChecker::new();
    checker.check_project_internal(dir)
  }

  fn check_project_internal(&self, dir: &Path) -> Result<CompatReport> {
    let pkg = Self::read_package_json(dir)?;
    let mut checks = Vec::new();
    let mut suggestions = Vec::new();
    let mut browser_matrix = Vec::new();

    let module_type = pkg
      .get("type")
      .and_then(|v| v.as_str())
      .unwrap_or("commonjs");
    checks.push(CompatCheck {
      name: "module_format".into(),
      status: CompatStatus::Compatible,
      message: format!("Project uses {module_type} module format"),
      suggestion: None,
    });

    let (framework, version) = Self::detect_framework(&pkg);
    checks.push(CompatCheck {
      name: "framework_detection".into(),
      status: CompatStatus::Compatible,
      message: format!("Detected framework: {framework}"),
      suggestion: None,
    });

    if let Some(engines) = pkg.get("engines").and_then(|v| v.as_object()) {
      for (key, val) in engines {
        let range = val.as_str().unwrap_or("*");
        let (status, suggestion) = Self::check_engine_range(range);
        checks.push(CompatCheck {
          name: format!("engine_{key}"),
          status,
          message: format!("Engine {key} requires {range}"),
          suggestion,
        });
      }
    }

    let native_checks = self.check_native_usage(dir);
    checks.extend(native_checks);

    let node_api_checks = self.check_node_api_surface(dir);
    checks.extend(node_api_checks);

    let framework_checks = self.check_framework_compat(&pkg);
    checks.extend(framework_checks);

    if let Ok(data) = CaniuseData::fetch() {
      browser_matrix = data.check_feature("es6");
    }

    suggestions.extend(checks.iter().filter_map(|c| c.suggestion.clone()));

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
      browser_matrix,
      suggestions,
    })
  }

  pub fn check_framework(dir: &Path, target: FrameworkTarget) -> Result<CompatReport> {
    let checker = CompatChecker::new();
    checker.check_framework_internal(dir, target)
  }

  fn check_framework_internal(&self, dir: &Path, target: FrameworkTarget) -> Result<CompatReport> {
    let pkg = Self::read_package_json(dir)?;
    let mut checks = Vec::new();
    let mut suggestions = Vec::new();
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
      suggestion: if !has_framework {
        Some(format!("Install {target:?}: npm install {package_name}", package_name = target_package(target)))
      } else {
        None
      },
    });

    match target {
      FrameworkTarget::Next => {
        let has_pages = dir.join("pages").is_dir();
        let has_app = dir.join("app").is_dir();
        let has_router = dir.join("src/app").is_dir();
        if has_pages || has_app || has_router {
          checks.push(CompatCheck {
            name: "route_structure".into(),
            status: CompatStatus::Compatible,
            message: "Next.js route structure detected".into(),
            suggestion: None,
          });
        } else {
          checks.push(CompatCheck {
            name: "route_structure".into(),
            status: CompatStatus::Partial,
            message: "No pages/ or app/ directory found".into(),
            suggestion: Some("Create a pages/ or app/ directory for routing".into()),
          });
        }
      }
      FrameworkTarget::React => {
        let has_jsx = Self::glob_has_files(dir, &["*.jsx", "*.tsx"]);
        checks.push(CompatCheck {
          name: "jsx_files".into(),
          status: CompatStatus::Compatible,
          message: if has_jsx {
            "JSX/TSX files found".into()
          } else {
            "No JSX/TSX files found".into()
          },
          suggestion: if !has_jsx {
            Some("Rename .js files to .jsx for JSX support".into())
          } else {
            None
          },
        });
      }
      FrameworkTarget::Astro => {
        let has_astro = Self::glob_has_files(dir, &["*.astro"]);
        checks.push(CompatCheck {
          name: "astro_files".into(),
          status: if has_astro { CompatStatus::Compatible } else { CompatStatus::Unknown },
          message: if has_astro { ".astro files found".into() } else { "No .astro files found".into() },
          suggestion: None,
        });
      }
      FrameworkTarget::Nest => {
        let has_nest = Self::glob_has_files(dir, &["*.module.ts", "*.controller.ts"]);
        check_framework_structure(&mut checks, has_nest, "NestJS module/controller");
        if !has_nest {
          suggestions.push("Create NestJS modules with @Module() decorator".into());
        }
      }
      FrameworkTarget::Prisma => {
        let has_schema = dir.join("prisma/schema.prisma").exists();
        check_framework_structure(&mut checks, has_schema, "Prisma schema");
        if !has_schema {
          suggestions.push("Run 'npx prisma init' to create schema".into());
        }
      }
      FrameworkTarget::Vue => {
        let has_vue = Self::glob_has_files(dir, &["*.vue"]);
        check_framework_structure(&mut checks, has_vue, "Vue single-file components");
      }
      FrameworkTarget::Svelte => {
        let has_svelte = Self::glob_has_files(dir, &["*.svelte"]);
        check_framework_structure(&mut checks, has_svelte, "Svelte components");
      }
      FrameworkTarget::Angular => {
        let has_angular = Self::glob_has_files(dir, &["*.component.ts", "*.module.ts"]);
        check_framework_structure(&mut checks, has_angular, "Angular components");
      }
      FrameworkTarget::Node => {
        checks.push(CompatCheck {
          name: "node_runtime".into(),
          status: CompatStatus::Compatible,
          message: "Node.js runtime is always compatible".into(),
          suggestion: None,
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
      browser_matrix: vec![],
      suggestions,
    })
  }

  pub fn check_node_compat(dir: &Path) -> Result<CompatReport> {
    let checker = CompatChecker::new();
    checker.check_node_compat_internal(dir)
  }

  fn check_node_compat_internal(&self, dir: &Path) -> Result<CompatReport> {
    let mut checks = Vec::new();
    let mut suggestions = Vec::new();

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
      let patterns = [
        format!("require('{module}')"),
        format!("from '{module}'"),
        format!("from \"{module}\""),
        format!("import * as {name} from '{module}'"),
      ];
      let patterns_refs: Vec<&str> = patterns.iter().map(|s| s.as_str()).collect();
      let found = self.search_files(dir, &patterns_refs);
      checks.push(CompatCheck {
        name: format!("node_{name}"),
        status: if found { CompatStatus::Partial } else { CompatStatus::Compatible },
        message: if found {
          format!("Uses Node.js '{module}' module — may need polyfill in non-Node environments")
        } else {
          format!("No usage of '{module}' detected")
        },
        suggestion: if found {
          Some(format!("Replace '{}' with platform-agnostic alternative or use condition imports", module))
        } else {
          None
        },
      });
    }

    let pkg = Self::read_package_json(dir).ok();
    let module_type = pkg
      .as_ref()
      .and_then(|p| p.get("type").and_then(|v| v.as_str()))
      .unwrap_or("commonjs")
      .to_string();
    let esm_syntax = self.search_files(dir, &["import ", "export "]);
    checks.push(CompatCheck {
      name: "module_syntax".into(),
      status: CompatStatus::Compatible,
      message: format!("Package type: {module_type}, ESM syntax detected: {esm_syntax}"),
      suggestion: if module_type == "commonjs" && esm_syntax {
        Some("Consider adding \"type\": \"module\" to package.json".into())
      } else {
        None
      },
    });

    suggestions.extend(checks.iter().filter_map(|c| c.suggestion.clone()));

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
      browser_matrix: vec![],
      suggestions,
    })
  }

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
      ("Vue", &["vue"]),
      ("Svelte", &["svelte"]),
      ("Angular", &["@angular/core"]),
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

  fn check_engine_range(range: &str) -> (CompatStatus, Option<String>) {
    let range = range.trim();
    if range == "*" || range == "x" {
      return (CompatStatus::Compatible, None);
    }
    if let Ok(req) = VersionReq::parse(range) {
      let ver_18 = semver::Version::new(18, 0, 0);
      let ver_17 = semver::Version::new(17, 0, 0);
      if req.matches(&ver_18) && !req.matches(&ver_17) {
        return (CompatStatus::Compatible, None);
      }
      return (CompatStatus::Partial, Some(format!("Upgrade Node.js to satisfy {}", range)));
    }
    if let Some(caret) = range.strip_prefix('^') {
      let major: u64 = caret.split('.').next().unwrap_or("0").parse().unwrap_or(0);
      if major >= 18 {
        return (CompatStatus::Compatible, None);
      }
      return (CompatStatus::Partial, Some(format!("Upgrade to Node.js 18+ (currently {major}.x)")));
    }
    (CompatStatus::Partial, Some(format!("Consider specifying a semver range like ^18.0.0")))
  }

  fn check_native_usage(&self, dir: &Path) -> Vec<CompatCheck> {
    let mut checks = Vec::new();
    let native_patterns = [
      ("napi", "napi", "Use @klyron/napi or wasm-based alternatives"),
      ("node-gyp", "node-gyp", "Use prebuilt binaries or WASM"),
      ("bindings", "bindings", "Use ESM imports instead of native bindings"),
      ("ffi", "ffi", "Use WebAssembly or klyron FFI"),
      ("neon", "neon", "Consider WASM-based alternatives"),
    ];

    for (name, keyword, suggestion) in &native_patterns {
      let found = self.search_files(dir, &[keyword]);
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
        suggestion: if found { Some(suggestion.to_string()) } else { None },
      });
    }
    checks
  }

  fn check_node_api_surface(&self, dir: &Path) -> Vec<CompatCheck> {
    let mut checks = Vec::new();
    let apis: &[(&str, &[&str])] = &[
      ("worker_threads", &["worker_threads", "Worker"]),
      ("cluster", &["cluster"]),
      ("async_hooks", &["async_hooks"]),
      ("diagnostics_channel", &["diagnostics_channel"]),
    ];

    for (name, patterns) in apis {
      let found = self.search_files(dir, patterns);
      checks.push(CompatCheck {
        name: format!("node_api_{name}"),
        status: if found { CompatStatus::Partial } else { CompatStatus::Compatible },
        message: if found {
          format!("Uses Node.js '{name}' API — may need polyfill")
        } else {
          format!("No usage of '{name}' detected")
        },
        suggestion: if found {
          Some(format!("'{name}' may not be available in all runtimes"))
        } else {
          None
        },
      });
    }
    checks
  }

  fn check_framework_compat(&self, pkg: &serde_json::Value) -> Vec<CompatCheck> {
    let mut checks = Vec::new();
    let deps = Self::merge_deps(pkg);

    let compat_pairs: &[(&str, &[&str])] = &[
      ("react-dom", &["react"]),
      ("next", &["react", "react-dom"]),
      ("@nestjs/core", &["reflect-metadata", "rxjs"]),
    ];

    for (pkg_name, required) in compat_pairs {
      if deps.contains_key(*pkg_name) {
    for req in *required {
      if !deps.contains_key(*req) {
            checks.push(CompatCheck {
              name: format!("{pkg_name}_needs_{req}"),
              status: CompatStatus::Incompatible,
              message: format!("'{pkg_name}' requires '{req}' but it's not installed"),
              suggestion: Some(format!("Install: npm install {req}")),
            });
          }
        }
      }
    }

    checks
  }

  fn search_files(&self, dir: &Path, patterns: &[&str]) -> bool {
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
      let path = entry.path();
      if path.is_dir() {
        if path
          .file_name()
          .is_some_and(|n| n == "node_modules" || n == ".git" || n == "target" || n == ".next")
        {
          continue;
        }
      }
      if path.is_file() {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
          if matches!(ext, "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "vue" | "svelte") {
            if let Ok(content) = std::fs::read_to_string(path) {
              for pat in patterns {
                if content.contains(pat) {
                  return true;
                }
              }
            }
          }
        }
      }
    }
    false
  }

  fn glob_has_files(dir: &Path, patterns: &[&str]) -> bool {
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
      if entry.path().is_file() {
        if let Some(ext) = entry.path().extension() {
          let ext_str = ext.to_string_lossy();
          for pat in patterns {
            let pat_ext = pat.trim_start_matches("*.");
            if ext_str == pat_ext {
              return true;
            }
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

fn check_framework_structure(checks: &mut Vec<CompatCheck>, found: bool, label: &str) {
  checks.push(CompatCheck {
    name: format!("{}_structure", label.to_lowercase().replace(' ', "_")),
    status: if found { CompatStatus::Compatible } else { CompatStatus::Unknown },
    message: if found {
      format!("{label} structure detected")
    } else {
      format!("No {label} structure detected")
    },
    suggestion: if !found {
      Some(format!("Create {label} files"))
    } else {
      None
    },
  });
}

fn target_package(target: FrameworkTarget) -> &'static str {
  match target {
    FrameworkTarget::React => "react react-dom",
    FrameworkTarget::Next => "next",
    FrameworkTarget::Astro => "astro",
    FrameworkTarget::Nest => "@nestjs/core",
    FrameworkTarget::Prisma => "prisma @prisma/client",
    FrameworkTarget::Node => "",
    FrameworkTarget::Vue => "vue",
    FrameworkTarget::Svelte => "svelte",
    FrameworkTarget::Angular => "@angular/core",
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
  fn test_check_engine_range() {
    let (status, _) = CompatChecker::check_engine_range(">=18.0.0");
    assert_eq!(status, CompatStatus::Compatible);
    let (status, _) = CompatChecker::check_engine_range(">=16.0.0");
    assert_eq!(status, CompatStatus::Partial);
  }

  #[test]
  fn test_suggestions_included() {
    let dir = temp_dir();
    let src = dir.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("server.js"), "require('napi')").unwrap();
    write_package_json(&dir, r#"{"name":"test"}"#);
    let report = CompatChecker::check_project(&dir).expect("Check failed");
    assert!(!report.suggestions.is_empty());
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
        suggestion: None,
      }],
      overall: CompatStatus::Compatible,
      summary: "OK".into(),
      browser_matrix: vec![],
      suggestions: vec![],
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: CompatReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.framework, "React");
  }
}
