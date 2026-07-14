use anyhow::{Context, Result};
use glob::glob;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use thiserror::Error;
use walkdir::WalkDir;

// ── Errors ───────────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum BundlerError {
  #[error("Build failed: {0}")]
  BuildFailed(String),
  #[error("Entry not found: {0}")]
  EntryNotFound(String),
  #[error("Output directory error: {0}")]
  OutputError(String),
  #[error("Minification error: {0}")]
  MinificationError(String),
  #[error("Code splitting error: {0}")]
  CodeSplittingError(String),
  #[error("Sourcemap error: {0}")]
  SourcemapError(String),
  #[error("CSS bundle error: {0}")]
  CssBundleError(String),
  #[error("Tool not found: {0}")]
  ToolNotFound(String),
}

impl From<std::io::Error> for BundlerError {
  fn from(e: std::io::Error) -> Self {
    Self::BuildFailed(e.to_string())
  }
}

// ── Bundler Kind ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundlerKind {
  Esbuild,
  Vite,
  Webpack,
  Rollup,
  Parcel,
  Turbopack,
  Rsbuild,
  SWC,
}

// ── Output Format ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
  Esm,
  Cjs,
  Iife,
  Umd,
  System,
}

impl OutputFormat {
  pub fn as_str(&self) -> &str {
    match self {
      Self::Esm => "esm",
      Self::Cjs => "cjs",
      Self::Iife => "iife",
      Self::Umd => "umd",
      Self::System => "system",
    }
  }
}

// ── Sourcemap Mode ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourcemapMode {
  None,
  Inline,
  External,
}

// ── Bundle Options ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BundleOptions {
  pub entry: String,
  pub out_dir: String,
  pub out_file: Option<String>,
  pub minify: bool,
  pub sourcemap: SourcemapMode,
  pub format: OutputFormat,
  pub splitting: bool,
  pub tree_shaking: bool,
  pub platform: String,
  pub external: Vec<String>,
  pub define: HashMap<String, String>,
  pub jsx: String,
}

impl Default for BundleOptions {
  fn default() -> Self {
    Self {
      entry: "src/index.js".into(),
      out_dir: "dist".into(),
      out_file: None,
      minify: true,
      sourcemap: SourcemapMode::External,
      format: OutputFormat::Esm,
      splitting: false,
      tree_shaking: true,
      platform: "browser".into(),
      external: Vec::new(),
      define: HashMap::new(),
      jsx: "automatic".into(),
    }
  }
}

// ── Chunk / Code Splitting ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Chunk {
  pub name: String,
  pub path: PathBuf,
  pub size: u64,
  pub entry: bool,
  pub imports: Vec<String>,
  pub exports: Vec<String>,
  pub modules: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SplitResult {
  pub chunks: Vec<Chunk>,
  pub total_size: u64,
  pub shared_chunks: Vec<String>,
}

// ── Bundle Result ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BundleResult {
  pub duration: Duration,
  pub output_files: Vec<PathBuf>,
  pub outputs: Vec<BundleOutput>,
  pub size_bytes: u64,
  pub warnings: Vec<String>,
  pub chunks: Option<SplitResult>,
}

#[derive(Debug, Clone)]
pub struct BundleOutput {
  pub path: PathBuf,
  pub size: u64,
  pub kind: String,
  pub integrity: String,
}

// ── CSS Bundle Options ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CssBundleOptions {
  pub entry: String,
  pub out_dir: String,
  pub minify: bool,
  pub sourcemap: bool,
  pub targets: Vec<String>,
}

impl Default for CssBundleOptions {
  fn default() -> Self {
    Self {
      entry: "src/styles.css".into(),
      out_dir: "dist".into(),
      minify: true,
      sourcemap: false,
      targets: vec!["last 2 versions".into()],
    }
  }
}

// ── Bundler ──────────────────────────────────────────────────────────────────

pub struct Bundler;

impl Bundler {
  // ── Detect ───────────────────────────────────────────────────────────────

  pub fn detect(dir: &Path) -> Option<BundlerKind> {
    let configs: [(&str, BundlerKind); 12] = [
      ("vite.config.*", BundlerKind::Vite),
      ("vite.config.*", BundlerKind::Vite),
      ("next.config.*", BundlerKind::Turbopack),
      ("esbuild.config.*", BundlerKind::Esbuild),
      ("esbuild.*", BundlerKind::Esbuild),
      ("webpack.config.*", BundlerKind::Webpack),
      ("rollup.config.*", BundlerKind::Rollup),
      ("parcelrc", BundlerKind::Parcel),
      (".parcelrc", BundlerKind::Parcel),
      ("rsbuild.config.*", BundlerKind::Rsbuild),
      (".swcrc", BundlerKind::SWC),
      ("tsconfig.json", BundlerKind::Esbuild),
    ];
    for (pattern, kind) in &configs {
      let pattern_path = dir.join(pattern).to_string_lossy().to_string();
      if glob(&pattern_path).ok().and_then(|mut g| g.next()).is_some() {
        return Some(*kind);
      }
    }
    None
  }

  // ── Bundle JS/TS ─────────────────────────────────────────────────────────

  pub fn bundle_js(dir: &Path, _kind: BundlerKind, opts: BundleOptions) -> Result<BundleResult> {
    let start = Instant::now();
    let (cmd, args) = Self::build_esbuild_args(&opts);

    // Try esbuild first, fallback to other bundlers
    let output = Command::new(&cmd)
      .args(&args)
      .current_dir(dir)
      .output()
      .with_context(|| format!("failed to execute bundler: {cmd}"))?;

    let duration = start.elapsed();

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(BundlerError::BuildFailed(stderr.to_string()).into());
    }

    let warnings = Self::extract_warnings(&output.stderr);

    // Collect output files
    let out_dir = dir.join(&opts.out_dir);
    let mut outputs = Vec::new();
    let mut total_size = 0u64;

    if out_dir.exists() {
      for entry in WalkDir::new(&out_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
          let path = entry.path().to_path_buf();
          let metadata = std::fs::metadata(&path)?;
          let data = std::fs::read(&path)?;
          let integrity = compute_hash(&data);
          let kind = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
          outputs.push(BundleOutput {
            size: metadata.len(),
            path: path.clone(),
            kind: kind.clone(),
            integrity,
          });
          total_size += metadata.len();
        }
      }
    }

    Ok(BundleResult {
      duration,
      output_files: outputs.iter().map(|o| o.path.clone()).collect(),
      outputs,
      size_bytes: total_size,
      warnings,
      chunks: None,
    })
  }

  fn build_esbuild_args(opts: &BundleOptions) -> (String, Vec<String>) {
    let mut args = vec![
      opts.entry.clone(),
      format!("--outdir={}", opts.out_dir),
    ];

    if opts.minify {
      args.push("--minify".into());
    }

    match opts.sourcemap {
      SourcemapMode::Inline => args.push("--sourcemap=inline".into()),
      SourcemapMode::External => args.push("--sourcemap".into()),
      SourcemapMode::None => {}
    }

    args.push(format!("--format={}", opts.format.as_str()));
    args.push(format!("--platform={}", opts.platform));

    if opts.splitting {
      args.push("--splitting".into());
    }

    if !opts.tree_shaking {
      args.push("--tree-shaking=false".into());
    }

    if let Some(ref out_file) = opts.out_file {
      args.push(format!("--outfile={}", out_file));
    }

    for ext in &opts.external {
      args.push(format!("--external:{}", ext));
    }

    for (key, value) in &opts.define {
      args.push(format!("--define:{}={}", key, value));
    }

    args.push(format!("--jsx={}", opts.jsx));

    ("esbuild".into(), args)
  }

  fn extract_warnings(stderr: &[u8]) -> Vec<String> {
    let stderr = String::from_utf8_lossy(stderr);
    let mut warnings = Vec::new();
    for line in stderr.lines() {
      let lower = line.to_lowercase();
      if lower.contains("warn") || lower.contains("warning") {
        warnings.push(line.to_string());
      }
    }
    warnings
  }

  // ── Bundle CSS ──────────────────────────────────────────────────────────

  pub fn bundle_css(dir: &Path, opts: CssBundleOptions) -> Result<BundleResult> {
    let start = Instant::now();
    let out_file = format!("{}/bundle.css", opts.out_dir);

    let mut args = vec![
      "--bundle".into(),
      opts.entry.clone(),
      "-o".into(),
      out_file.clone(),
    ];

    if opts.minify {
      args.push("--minify".into());
    }
    if opts.sourcemap {
      args.push("--sourcemap".into());
    }
    for target in &opts.targets {
      args.push("--targets".into());
      args.push(target.clone());
    }

    let cmd = if cfg!(target_os = "windows") {
      "lightningcss.exe"
    } else {
      "lightningcss"
    };

    let output = Command::new(cmd)
      .args(&args)
      .current_dir(dir)
      .output();

    let (duration, outputs, warnings, total_size) = match output {
      Ok(output) => {
        let duration = start.elapsed();
        let warnings = Self::extract_warnings(&output.stderr);
        let out_dir = dir.join(&opts.out_dir);
        let mut outputs = Vec::new();
        let mut total_size = 0u64;
        if out_dir.exists() {
          for entry in WalkDir::new(&out_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.path().extension().map_or(false, |e| e == "css") {
              let path = entry.path().to_path_buf();
              let data = std::fs::read(&path)?;
              let integrity = compute_hash(&data);
              let size = data.len() as u64;
              outputs.push(BundleOutput {
                path,
                size,
                kind: "css".into(),
                integrity,
              });
              total_size += size;
            }
          }
        }
        (duration, outputs, warnings, total_size)
      }
      Err(_e) => {
        // Fallback: use esbuild for CSS bundling
        let mut css_opts = BundleOptions::default();
        css_opts.entry = opts.entry.clone();
        css_opts.out_dir = opts.out_dir.clone();
        css_opts.minify = opts.minify;
        css_opts.sourcemap = if opts.sourcemap {
          SourcemapMode::External
        } else {
          SourcemapMode::None
        };
        let result = Self::bundle_js(dir, BundlerKind::Esbuild, css_opts)?;
        return Ok(result);
      }
    };

    Ok(BundleResult {
      duration,
      output_files: outputs.iter().map(|o| o.path.clone()).collect(),
      outputs,
      size_bytes: total_size,
      warnings,
      chunks: None,
    })
  }

  // ── Minification (SWC via subprocess) ───────────────────────────────────

  pub fn minify_js(file: &Path, output: &Path, swc_config: Option<&str>) -> Result<BundleResult> {
    let start = Instant::now();
    let mut args = vec![
      file.to_string_lossy().to_string(),
      "-o".into(),
      output.to_string_lossy().to_string(),
    ];

    if let Some(config) = swc_config {
      args.push("--config".into());
      args.push(config.into());
    } else {
      args.push("--config".into());
      args.push(
        r#"{"jsc":{"minify":{"compress":{"unused":true},"mangle":true},"parser":{"syntax":"ecmascript"},"target":"es2020"}}"#.into(),
      );
    }

    let cmd = if cfg!(target_os = "windows") {
      "swc.exe"
    } else {
      "swc"
    };

    let output_cmd = Command::new(cmd)
      .args(&args)
      .output()
      .with_context(|| format!("failed to execute swc minification for {}", file.display()))?;

    let duration = start.elapsed();

    if !output_cmd.status.success() {
      let stderr = String::from_utf8_lossy(&output_cmd.stderr);
      return Err(BundlerError::MinificationError(stderr.to_string()).into());
    }

    let data = std::fs::read(output)?;
    let integrity = compute_hash(&data);
    let size = data.len() as u64;

    Ok(BundleResult {
      duration,
      output_files: vec![output.to_path_buf()],
      outputs: vec![BundleOutput {
        path: output.to_path_buf(),
        size,
        kind: "js".into(),
        integrity,
      }],
      size_bytes: size,
      warnings: Vec::new(),
      chunks: None,
    })
  }

  // ── Code Splitting / Chunk Splitting ─────────────────────────────────────

  pub fn compute_chunks(entry_files: &[PathBuf]) -> SplitResult {
    let mut modules_by_chunk: HashMap<String, Vec<String>> = HashMap::new();
    let mut shared = HashSet::new();

    for entry in entry_files {
      let name = entry
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("chunk")
        .to_string();

      let mut modules = Vec::new();
      if entry.exists() {
        if let Ok(content) = std::fs::read_to_string(entry) {
          for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") || trimmed.starts_with("import(") || trimmed.starts_with("require(") {
              // Extract module name from import
              let module_name = trimmed
                .trim_start_matches("import ")
                .trim_start_matches('(')
                .trim_start_matches('"')
                .trim_start_matches('\'');
              if !module_name.starts_with('.') && !module_name.starts_with('/') {
                let module_name = module_name
                  .split('"')
                  .next()
                  .or_else(|| module_name.split('\'').next())
                  .unwrap_or(module_name)
                  .trim_end_matches(')')
                  .to_string();
                modules.push(module_name);
              }
            }
          }
        }
      }
      modules_by_chunk.insert(name.clone(), modules.clone());
    }

    // Find shared modules
    let mut module_counts: HashMap<String, usize> = HashMap::new();
    for modules in modules_by_chunk.values() {
      for m in modules {
        *module_counts.entry(m.clone()).or_default() += 1;
      }
    }
    for (module, count) in &module_counts {
      if *count > 1 {
        shared.insert(module.clone());
      }
    }

    let mut chunks = Vec::new();
    let mut total_size = 0u64;

    for (name, modules) in &modules_by_chunk {
      let path = PathBuf::from(format!("{}.js", name));
      let size = std::fs::read_to_string(&path).ok().map(|s| s.len() as u64).unwrap_or(0);
      total_size += size;
      chunks.push(Chunk {
        name: name.clone(),
        path,
        size,
        entry: true,
        imports: modules.clone(),
        exports: Vec::new(),
        modules: modules.clone(),
      });
    }

    SplitResult {
      shared_chunks: shared.into_iter().collect(),
      chunks,
      total_size,
    }
  }

  // ── Tree-Shaking Analysis ────────────────────────────────────────────────

  pub fn analyze_tree_shaking(entry: &Path) -> Result<HashMap<String, Vec<String>>> {
    let mut tree = HashMap::new();
    Self::analyze_dependencies(entry, &mut tree, &mut HashSet::new())?;
    Ok(tree)
  }

  fn analyze_dependencies(
    file: &Path,
    tree: &mut HashMap<String, Vec<String>>,
    visited: &mut HashSet<PathBuf>,
  ) -> Result<()> {
    if !file.exists() || !visited.insert(file.to_path_buf()) {
      return Ok(());
    }

    let content = std::fs::read_to_string(file)?;
    let mut deps = Vec::new();

    for line in content.lines() {
      let trimmed = line.trim();
      if let Some(rest) = trimmed.strip_prefix("import ") {
        if let Some(spec) = rest
          .split('"')
          .nth(1)
          .or_else(|| rest.split('\'').nth(1))
        {
          deps.push(spec.to_string());
        }
      } else if let Some(rest) = trimmed.strip_prefix("require(") {
        if let Some(spec) = rest
          .split('"')
          .nth(1)
          .or_else(|| rest.split('\'').nth(1))
        {
          deps.push(spec.to_string());
        }
      }
    }

    tree.insert(file.to_string_lossy().to_string(), deps.clone());

    for dep in &deps {
      if dep.starts_with('.') {
        let parent = file.parent().unwrap_or(Path::new("."));
        let dep_path = parent.join(dep);
        let resolved = if dep_path.is_file() {
          dep_path
        } else {
          // Try with extensions
          let with_ext = dep_path.with_extension("js");
          if with_ext.is_file() {
            with_ext
          } else {
            continue;
          }
        };
        Self::analyze_dependencies(&resolved, tree, visited)?;
      }
    }

    Ok(())
  }

  // ── Sourcemap Generation ─────────────────────────────────────────────────

  pub fn generate_sourcemap(js_file: &Path, mode: SourcemapMode) -> Result<PathBuf> {
    match mode {
      SourcemapMode::None => Ok(js_file.to_path_buf()),
      SourcemapMode::Inline => {
        // Sourcemap already inline from esbuild
        Ok(js_file.to_path_buf())
      }
      SourcemapMode::External => {
        let map_path = js_file.with_extension("js.map");
        if map_path.exists() {
          Ok(map_path)
        } else {
          // Generate one using esbuild
          let args = vec![
            js_file.to_string_lossy().to_string(),
            "--sourcemap".into(),
            format!("--outfile={}", map_path.to_string_lossy()),
          ];
          let output = Command::new("esbuild")
            .args(&args)
            .output()
            .with_context(|| format!("sourcemap generation failed for {}", js_file.display()))?;

          if !output.status.success() {
            return Err(BundlerError::SourcemapError(
              String::from_utf8_lossy(&output.stderr).to_string(),
            ).into());
          }
          Ok(map_path)
        }
      }
    }
  }

  // ── Config Detection ─────────────────────────────────────────────────────

  pub fn get_config_path(dir: &Path, kind: BundlerKind) -> Option<PathBuf> {
    let patterns: &[&str] = match kind {
      BundlerKind::Vite => &["vite.config.*"],
      BundlerKind::Turbopack => &["next.config.*"],
      BundlerKind::Esbuild => &["esbuild.config.*", "esbuild.*"],
      BundlerKind::Webpack => &["webpack.config.*"],
      BundlerKind::Rollup => &["rollup.config.*"],
      BundlerKind::Parcel => &["parcelrc", ".parcelrc"],
      BundlerKind::Rsbuild => &["rsbuild.config.*"],
      BundlerKind::SWC => &[".swcrc"],
    };
    for pattern in patterns {
      let pattern_path = dir.join(pattern).to_string_lossy().to_string();
      if let Ok(matches) = glob(&pattern_path) {
        for entry in matches.flatten() {
          return Some(entry);
        }
      }
    }
    None
  }

  // ── Build Command ────────────────────────────────────────────────────────

  pub fn get_build_command(kind: BundlerKind, opts: &BundleOptions) -> (String, Vec<String>) {
    match kind {
      BundlerKind::Esbuild => Self::build_esbuild_args(opts),
      BundlerKind::Vite => {
        let mut args = vec!["build".into(), "--outDir".into(), opts.out_dir.clone()];
        if opts.minify {
          args.push("--minify".into());
        }
        if opts.sourcemap != SourcemapMode::None {
          args.push("--sourcemap".into());
        }
        match opts.format {
          OutputFormat::Cjs => args.push("--ssr".into()),
          _ => {}
        }
        ("vite".into(), args)
      }
      BundlerKind::Webpack => {
        let mut args = vec![
          "--mode".into(),
          if opts.minify { "production" } else { "development" }.into(),
        ];
        if opts.sourcemap != SourcemapMode::None {
          args.push("--devtool".into());
          args.push("source-map".into());
        }
        ("webpack".into(), args)
      }
      BundlerKind::Rollup => {
        let mut args = vec!["-c".into()];
        if opts.sourcemap != SourcemapMode::None {
          args.push("-m".into());
        }
        ("rollup".into(), args)
      }
      BundlerKind::Parcel => {
        let mut args = vec![
          "build".into(),
          opts.entry.clone(),
          "--dist-dir".into(),
          opts.out_dir.clone(),
        ];
        if !opts.minify {
          args.push("--no-minify".into());
        }
        if opts.sourcemap == SourcemapMode::None {
          args.push("--no-source-maps".into());
        }
        ("parcel".into(), args)
      }
      BundlerKind::Turbopack => {
        ("next".into(), vec!["build".into()])
      }
      BundlerKind::Rsbuild => {
        let mut args = vec!["build".into()];
        if opts.sourcemap != SourcemapMode::None {
          args.push("--sourcemap".into());
        }
        ("rsbuild".into(), args)
      }
      BundlerKind::SWC => {
        let mut args = vec![
          opts.entry.clone(),
          "-d".into(),
          opts.out_dir.clone(),
        ];
        if opts.sourcemap != SourcemapMode::None {
          args.push("-s".into());
        }
        ("swc".into(), args)
      }
    }
  }
}

impl Default for Bundler {
  fn default() -> Self {
    Self
  }
}

// ── Utility ──────────────────────────────────────────────────────────────────

pub fn compute_hash(data: &[u8]) -> String {
  let mut hasher = Sha256::new();
  hasher.update(data);
  hex::encode(hasher.finalize())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;

  fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("klyron_test_bundler_{}", name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
  }

  #[test]
  fn test_detect_vite() {
    let dir = temp_dir("detect_vite");
    fs::write(dir.join("vite.config.ts"), "").unwrap();
    assert_eq!(Bundler::detect(&dir), Some(BundlerKind::Vite));
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_detect_webpack() {
    let dir = temp_dir("detect_webpack");
    fs::write(dir.join("webpack.config.js"), "").unwrap();
    assert_eq!(Bundler::detect(&dir), Some(BundlerKind::Webpack));
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_detect_none() {
    let dir = temp_dir("detect_none");
    assert_eq!(Bundler::detect(&dir), None);
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_bundle_options_defaults() {
    let opts = BundleOptions::default();
    assert_eq!(opts.entry, "src/index.js");
    assert_eq!(opts.out_dir, "dist");
    assert!(opts.minify);
    assert!(opts.tree_shaking);
    assert_eq!(opts.format, OutputFormat::Esm);
  }

  #[test]
  fn test_css_bundle_options_defaults() {
    let opts = CssBundleOptions::default();
    assert_eq!(opts.entry, "src/styles.css");
    assert!(opts.minify);
    assert!(!opts.sourcemap);
  }

  #[test]
  fn test_output_format_as_str() {
    assert_eq!(OutputFormat::Esm.as_str(), "esm");
    assert_eq!(OutputFormat::Cjs.as_str(), "cjs");
    assert_eq!(OutputFormat::Iife.as_str(), "iife");
    assert_eq!(OutputFormat::Umd.as_str(), "umd");
  }

  #[test]
  fn test_compute_hash() {
    let hash = compute_hash(b"hello");
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
  }

  #[test]
  fn test_get_config_path_vite() {
    let dir = temp_dir("config_vite");
    fs::write(dir.join("vite.config.ts"), "").unwrap();
    let cfg = Bundler::get_config_path(&dir, BundlerKind::Vite);
    assert!(cfg.is_some());
    assert!(cfg.unwrap().ends_with("vite.config.ts"));
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_get_config_path_swc() {
    let dir = temp_dir("config_swc");
    fs::write(dir.join(".swcrc"), "{}").unwrap();
    let cfg = Bundler::get_config_path(&dir, BundlerKind::SWC);
    assert!(cfg.is_some());
    assert!(cfg.unwrap().ends_with(".swcrc"));
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_get_build_command_esbuild_esm() {
    let opts = BundleOptions {
      entry: "src/index.ts".into(),
      out_dir: "dist".into(),
      minify: true,
      sourcemap: SourcemapMode::External,
      format: OutputFormat::Esm,
      ..Default::default()
    };
    let (cmd, args) = Bundler::get_build_command(BundlerKind::Esbuild, &opts);
    assert_eq!(cmd, "esbuild");
    assert!(args.contains(&"--minify".into()));
    assert!(args.contains(&"--sourcemap".into()));
    assert!(args.contains(&"--format=esm".into()));
  }

  #[test]
  fn test_get_build_command_vite_cjs() {
    let opts = BundleOptions {
      entry: "src/main.ts".into(),
      out_dir: "build".into(),
      minify: false,
      sourcemap: SourcemapMode::None,
      format: OutputFormat::Cjs,
      ..Default::default()
    };
    let (cmd, args) = Bundler::get_build_command(BundlerKind::Vite, &opts);
    assert_eq!(cmd, "vite");
    assert!(args.contains(&"--ssr".into()));
  }

  #[test]
  fn test_analyze_tree_shaking_simple() {
    let dir = temp_dir("treeshake");
    let entry = dir.join("index.js");
    fs::write(
      &entry,
      "import { foo } from './utils';\nimport bar from 'lodash';",
    )
    .unwrap();
    fs::write(dir.join("utils.js"), "export const foo = 1;").unwrap();
    let tree = Bundler::analyze_tree_shaking(&entry).unwrap();
    assert!(tree.contains_key(&entry.to_string_lossy().to_string()));
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_compute_chunks_empty() {
    let chunks = Bundler::compute_chunks(&[]);
    assert!(chunks.chunks.is_empty());
    assert_eq!(chunks.total_size, 0);
  }

  #[test]
  fn test_sourcemap_modes() {
    assert_eq!(SourcemapMode::None as u8, 0);
    assert_eq!(SourcemapMode::Inline as u8, 1);
    assert_eq!(SourcemapMode::External as u8, 2);
  }

  #[test]
  fn test_bundle_result_defaults() {
    let result = BundleResult {
      duration: Duration::from_secs(0),
      output_files: vec![],
      outputs: vec![],
      size_bytes: 0,
      warnings: vec![],
      chunks: None,
    };
    assert_eq!(result.size_bytes, 0);
    assert!(result.warnings.is_empty());
  }

  #[test]
  fn test_chunk_struct() {
    let chunk = Chunk {
      name: "main".into(),
      path: PathBuf::from("main.js"),
      size: 1024,
      entry: true,
      imports: vec!["lodash".into()],
      exports: vec!["default".into()],
      modules: vec!["src/index.js".into()],
    };
    assert!(chunk.entry);
    assert_eq!(chunk.imports[0], "lodash");
  }

  #[test]
  fn test_split_result() {
    let result = SplitResult {
      chunks: vec![],
      total_size: 0,
      shared_chunks: vec![],
    };
    assert!(result.shared_chunks.is_empty());
  }

  #[test]
  fn test_bundler_error_types() {
    let e1 = BundlerError::BuildFailed("fail".into());
    let e2 = BundlerError::ToolNotFound("esbuild".into());
    assert!(e1.to_string().contains("fail"));
    assert!(e2.to_string().contains("esbuild"));
  }

  #[test]
  fn test_bundle_output() {
    let output = BundleOutput {
      path: PathBuf::from("dist/bundle.js"),
      size: 1024,
      kind: "js".into(),
      integrity: "abc123".into(),
    };
    assert_eq!(output.kind, "js");
    assert_eq!(output.size, 1024);
  }

  #[test]
  fn test_output_format_variants() {
    let variants = [OutputFormat::Esm, OutputFormat::Cjs, OutputFormat::Iife, OutputFormat::Umd, OutputFormat::System];
    assert_eq!(variants.len(), 5);
  }

  #[test]
  fn test_bundler_kind_variants() {
    let kinds = [
      BundlerKind::Esbuild,
      BundlerKind::Vite,
      BundlerKind::Webpack,
      BundlerKind::Rollup,
      BundlerKind::Parcel,
      BundlerKind::Turbopack,
      BundlerKind::Rsbuild,
      BundlerKind::SWC,
    ];
    assert_eq!(kinds.len(), 8);
  }
}
