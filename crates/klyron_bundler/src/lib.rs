use anyhow::{Context, Result};
use glob::glob;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use thiserror::Error;
use walkdir::WalkDir;

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
  #[error("Tree-shaking analysis error: {0}")]
  TreeShakingError(String),
}

impl From<std::io::Error> for BundlerError {
  fn from(e: std::io::Error) -> Self {
    Self::BuildFailed(e.to_string())
  }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourcemapMode {
  None,
  Inline,
  External,
}

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
  pub css_entry: Option<String>,
  pub minifier_callback: Option<MinifierCallback>,
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
      css_entry: None,
      minifier_callback: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct MinifierCallback {
  pub program: String,
  pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Chunk {
  pub name: String,
  pub path: PathBuf,
  pub size: u64,
  pub entry: bool,
  pub imports: Vec<String>,
  pub exports: Vec<String>,
  pub modules: Vec<String>,
  pub dynamic: bool,
}

#[derive(Debug, Clone)]
pub struct SplitResult {
  pub chunks: Vec<Chunk>,
  pub total_size: u64,
  pub shared_chunks: Vec<String>,
  pub dynamic_imports: Vec<DynamicImport>,
}

#[derive(Debug, Clone)]
pub struct DynamicImport {
  pub specifier: String,
  pub source_file: String,
  pub line: usize,
  pub chunk_name: String,
}

#[derive(Debug, Clone)]
pub struct ExportInfo {
  pub name: String,
  pub local_name: String,
  pub is_default: bool,
  pub used: bool,
}

#[derive(Debug, Clone)]
pub struct ModuleExports {
  pub file: String,
  pub exports: Vec<ExportInfo>,
  pub re_exports: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BundleResult {
  pub duration: Duration,
  pub output_files: Vec<PathBuf>,
  pub outputs: Vec<BundleOutput>,
  pub size_bytes: u64,
  pub warnings: Vec<String>,
  pub chunks: Option<SplitResult>,
  pub css_files: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct BundleOutput {
  pub path: PathBuf,
  pub size: u64,
  pub kind: String,
  pub integrity: String,
}

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

pub struct Bundler;

impl Bundler {
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

  pub fn bundle_js(dir: &Path, _kind: BundlerKind, mut opts: BundleOptions) -> Result<BundleResult> {
    let start = Instant::now();
    let mut css_files = Vec::new();

    // Collect CSS imports before bundling
    if opts.css_entry.is_some() || opts.tree_shaking {
      let entry_path = dir.join(&opts.entry);
      if entry_path.exists() {
        css_files = Self::collect_css_imports(&entry_path, dir, &mut HashSet::new())?;
      }
    }

    // Handle CSS bundling if entry specified
    if let Some(ref css_entry) = opts.css_entry {
      let css_opts = CssBundleOptions {
        entry: css_entry.clone(),
        out_dir: opts.out_dir.clone(),
        minify: opts.minify,
        sourcemap: opts.sourcemap == SourcemapMode::External || opts.sourcemap == SourcemapMode::Inline,
        targets: vec!["last 2 versions".into()],
      };
      let _css_result = Self::bundle_css(dir, css_opts)?;
    }

    // Apply minifier callback instead of default minification
    if opts.minify && opts.minifier_callback.is_some() {
      opts.minify = false;
    }

    let (cmd, mut args) = Self::build_esbuild_args(&opts);

    if !css_files.is_empty() && opts.splitting {
      args.push("--loader:.css=text".into());
    }

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

    let out_dir = dir.join(&opts.out_dir);
    let mut outputs = Vec::new();
    let mut total_size = 0u64;

    if out_dir.exists() {
      for entry in WalkDir::new(&out_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
          let path = entry.path().to_path_buf();
          let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
          if ext == "css" {
            css_files.push(path.clone());
          }
          let metadata = std::fs::metadata(&path)?;
          let data = std::fs::read(&path)?;
          let integrity = compute_hash(&data);
          outputs.push(BundleOutput {
            size: metadata.len(),
            path: path.clone(),
            kind: ext.to_string(),
            integrity,
          });
          total_size += metadata.len();
        }
      }
    }

    // Run minifier callback if available
    if let Some(ref cb) = opts.minifier_callback {
      for output_file in &outputs {
        if output_file.kind == "js" || output_file.kind == "mjs" {
          let result = Self::minify_with_callback(&output_file.path, dir, cb)?;
          total_size = total_size.saturating_sub(output_file.size).saturating_add(result.size_bytes);
        }
      }
    }

    let chunks = if opts.splitting {
      let entry_files: Vec<PathBuf> = outputs.iter()
        .filter(|o| o.kind == "js" || o.kind == "mjs")
        .map(|o| o.path.clone())
        .collect();
      Some(Self::compute_chunks(&entry_files, dir))
    } else {
      None
    };

    Ok(BundleResult {
      duration,
      output_files: outputs.iter().map(|o| o.path.clone()).collect(),
      outputs,
      size_bytes: total_size,
      warnings,
      chunks,
      css_files,
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
    } else {
      args.push("--tree-shaking=true".into());
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

  fn minify_with_callback(file: &Path, _dir: &Path, cb: &MinifierCallback) -> Result<BundleResult> {
    let start = Instant::now();
    let output = Command::new(&cb.program)
      .args(&cb.args)
      .arg(file.to_string_lossy().to_string())
      .output()
      .with_context(|| format!("minifier callback failed for {}", file.display()))?;
    let duration = start.elapsed();
    if !output.status.success() {
      return Err(BundlerError::MinificationError(
        String::from_utf8_lossy(&output.stderr).to_string(),
      ).into());
    }
    let data = std::fs::read(file)?;
    let integrity = compute_hash(&data);
    let size = data.len() as u64;
    Ok(BundleResult {
      duration,
      output_files: vec![file.to_path_buf()],
      outputs: vec![BundleOutput {
        path: file.to_path_buf(),
        size,
        kind: "js".into(),
        integrity,
      }],
      size_bytes: size,
      warnings: Vec::new(),
      chunks: None,
      css_files: Vec::new(),
    })
  }

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

    let css_paths: Vec<PathBuf> = outputs.iter().map(|o| o.path.clone()).collect();
    let out_paths: Vec<PathBuf> = outputs.iter().map(|o| o.path.clone()).collect();
    Ok(BundleResult {
      duration,
      output_files: out_paths,
      outputs,
      size_bytes: total_size,
      warnings,
      chunks: None,
      css_files: css_paths,
    })
  }

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
      css_files: Vec::new(),
    })
  }

  pub fn compute_chunks(entry_files: &[PathBuf], dir: &Path) -> SplitResult {
    let mut modules_by_chunk: HashMap<String, Vec<String>> = HashMap::new();
    let mut shared = HashSet::new();
    let mut dynamic_imports = Vec::new();
    let dynamic_re = Regex::new(r#"import\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap();
    let named_import_re = Regex::new(r#"import\s+\{?\s*([^}]+)\s*\}?\s*from\s+["']([^"']+)["']"#).unwrap();

    for entry in entry_files {
      let name = entry
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("chunk")
        .to_string();

      let mut modules = Vec::new();
      if entry.exists() {
        if let Ok(content) = std::fs::read_to_string(entry) {
          for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Detect static imports
            if trimmed.starts_with("import ") && trimmed.contains("from") {
              if let Some(caps) = named_import_re.captures(trimmed) {
                let spec = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                if !spec.starts_with('.') && !spec.starts_with('/') {
                  modules.push(spec.to_string());
                }
              }
            }

            if trimmed.starts_with("require(") {
              let module_name = trimmed
                .trim_start_matches("require(")
                .trim_start_matches('"')
                .trim_start_matches('\'');
              let module_name = module_name
                .split('"')
                .next()
                .or_else(|| module_name.split('\'').next())
                .unwrap_or(module_name)
                .trim_end_matches(')')
                .to_string();
              if !module_name.starts_with('.') && !module_name.starts_with('/') {
                modules.push(module_name);
              }
            }

            // Detect dynamic imports for code splitting
            if let Some(caps) = dynamic_re.captures(trimmed) {
              let spec = caps.get(1).map(|m| m.as_str()).to_owned().unwrap_or("");
              let chunk_name = spec
                .trim_end_matches(".js")
                .trim_end_matches(".ts")
                .trim_end_matches(".tsx")
                .trim_end_matches(".jsx")
                .replace('/', "-")
                .to_string();
              dynamic_imports.push(DynamicImport {
                specifier: spec.to_string(),
                source_file: name.clone(),
                line: line_idx + 1,
                chunk_name: chunk_name.clone(),
              });
              modules.push(spec.to_string());
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
      let path = dir.join(format!("{}.js", name));
      let size = std::fs::read_to_string(&path).ok().map(|s| s.len() as u64).unwrap_or(0);
      total_size += size;
      let is_dynamic = dynamic_imports.iter().any(|d| d.chunk_name == *name);
      chunks.push(Chunk {
        name: name.clone(),
        path,
        size,
        entry: true,
        imports: modules.clone(),
        exports: Vec::new(),
        modules: modules.clone(),
        dynamic: is_dynamic,
      });
    }

    // Add dynamic import chunks
    for dyn_import in &dynamic_imports {
      let chunk_name = &dyn_import.chunk_name;
      let chunk_path = dir.join(format!("{}.js", chunk_name));
      if !chunks.iter().any(|c| c.name == *chunk_name) {
        let size = std::fs::read_to_string(&chunk_path).ok().map(|s| s.len() as u64).unwrap_or(0);
        total_size += size;
        chunks.push(Chunk {
          name: chunk_name.clone(),
          path: chunk_path,
          size,
          entry: false,
          imports: vec![dyn_import.specifier.clone()],
          exports: Vec::new(),
          modules: Vec::new(),
          dynamic: true,
        });
      }
    }

    SplitResult {
      shared_chunks: shared.into_iter().collect(),
      chunks,
      total_size,
      dynamic_imports,
    }
  }

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

  pub fn analyze_exports(entry: &Path) -> Result<Vec<ModuleExports>> {
    let mut all_exports = Vec::new();
    let mut visited = HashSet::new();
    Self::analyze_exports_recursive(entry, &mut all_exports, &mut visited)?;
    Ok(all_exports)
  }

  fn analyze_exports_recursive(
    file: &Path,
    all_exports: &mut Vec<ModuleExports>,
    visited: &mut HashSet<PathBuf>,
  ) -> Result<()> {
    if !file.exists() || !visited.insert(file.to_path_buf()) {
      return Ok(());
    }

    let content = std::fs::read_to_string(file)?;
    let mut exports = Vec::new();
    let mut re_exports = Vec::new();

    let export_named_re = Regex::new(r#"export\s+(?:const|function|class|let|var|type|interface)\s+(\w+)"#).unwrap();
    let export_default_re = Regex::new(r#"export\s+default\s+(?:function|class|const)?\s*(\w*)"#).unwrap();
    let export_re_export_re = Regex::new(r#"export\s+\{?\s*([^}]+)\s*\}?\s*from\s+["']([^"']+)["']"#).unwrap();
    let import_re = Regex::new(r#"import\s+\{?\s*([^}]+)\s*\}?\s*from\s+["']([^"']+)["']"#).unwrap();

    for line in content.lines() {
      let trimmed = line.trim();

      if let Some(caps) = export_named_re.captures(trimmed) {
        exports.push(ExportInfo {
          name: caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string(),
          local_name: caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string(),
          is_default: false,
          used: true,
        });
      }

      if let Some(caps) = export_default_re.captures(trimmed) {
        let name = caps.get(1).map(|m| m.as_str()).unwrap_or("default").to_string();
        let name = if name.is_empty() { "default".to_string() } else { name };
        exports.push(ExportInfo {
          name: name.clone(),
          local_name: name,
          is_default: true,
          used: true,
        });
      }

      if let Some(caps) = export_re_export_re.captures(trimmed) {
        let spec = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
        re_exports.push(spec.clone());
        if spec.starts_with('.') {
          let parent = file.parent().unwrap_or(Path::new("."));
          let dep_path = parent.join(&spec);
          let resolved = if dep_path.is_file() {
            dep_path
          } else {
            let with_ext = dep_path.with_extension("js");
            if with_ext.is_file() { with_ext } else { continue; }
          };
          Self::analyze_exports_recursive(&resolved, all_exports, visited)?;
        }
      }

      // Track used imports to mark exports
      if let Some(caps) = import_re.captures(trimmed) {
        let imports_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let spec = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        if spec.starts_with('.') {
          let parent = file.parent().unwrap_or(Path::new("."));
          let dep_path = parent.join(spec);
          let resolved = if dep_path.is_file() {
            dep_path
          } else {
            let with_ext = dep_path.with_extension("js");
            if with_ext.is_file() { with_ext } else { continue; }
          };
          Self::analyze_exports_recursive(&resolved, all_exports, visited)?;

          // Mark specific imports as used
          for import_name in imports_str.split(',').map(|s| s.trim().trim_start_matches("type ")) {
            let import_name = import_name.split(" as ").next().unwrap_or(import_name).trim();
            for module_exports in all_exports.iter_mut() {
              for export in module_exports.exports.iter_mut() {
                if export.name == import_name {
                  export.used = true;
                }
              }
            }
          }
        }
      }
    }

    all_exports.push(ModuleExports {
      file: file.to_string_lossy().to_string(),
      exports,
      re_exports,
    });

    Ok(())
  }

  fn collect_css_imports(file: &Path, dir: &Path, visited: &mut HashSet<PathBuf>) -> Result<Vec<PathBuf>> {
    if !file.exists() || !visited.insert(file.to_path_buf()) {
      return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(file)?;
    let css_import_re = Regex::new(r#"import\s+["']([^"']+\.css)["']"#).unwrap();
    let mut css_files = Vec::new();

    for line in content.lines() {
      if let Some(caps) = css_import_re.captures(line.trim()) {
        let spec = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let css_path = if spec.starts_with('.') {
          let parent = file.parent().unwrap_or(Path::new("."));
          parent.join(spec)
        } else {
          dir.join("node_modules").join(spec)
        };
        if css_path.exists() {
          css_files.push(css_path);
        }
      }

      let js_import_re = Regex::new(r#"import\s+.*from\s+["']([^"']+)["']"#).unwrap();
      if let Some(caps) = js_import_re.captures(line.trim()) {
        let spec = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        if spec.starts_with('.') {
          let parent = file.parent().unwrap_or(Path::new("."));
          let js_path = parent.join(spec);
          let resolved = if js_path.is_file() {
            js_path
          } else {
            let with_ext = js_path.with_extension("js");
            if with_ext.is_file() { with_ext } else { continue; }
          };
          let nested = Self::collect_css_imports(&resolved, dir, visited)?;
          css_files.extend(nested);
        }
      }
    }

    Ok(css_files)
  }

  pub fn generate_sourcemap(js_file: &Path, mode: SourcemapMode) -> Result<PathBuf> {
    match mode {
      SourcemapMode::None => Ok(js_file.to_path_buf()),
      SourcemapMode::Inline => {
        Ok(js_file.to_path_buf())
      }
      SourcemapMode::External => {
        let map_path = js_file.with_extension("js.map");
        if map_path.exists() {
          Ok(map_path)
        } else {
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

pub fn compute_hash(data: &[u8]) -> String {
  let mut hasher = Sha256::new();
  hasher.update(data);
  hex::encode(hasher.finalize())
}

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
  fn test_analyze_exports() {
    let dir = temp_dir("exports_analyze");
    fs::write(dir.join("index.js"), "export const foo = 1;\nexport function bar() {}\nexport default class Baz {}").unwrap();
    let modules = Bundler::analyze_exports(&dir.join("index.js")).unwrap();
    assert!(!modules.is_empty());
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_collect_css_imports() {
    let dir = temp_dir("css_imports");
    fs::write(dir.join("index.js"), r#"import './styles.css';"#).unwrap();
    fs::write(dir.join("styles.css"), "body { color: red; }").unwrap();
    let css = Bundler::collect_css_imports(&dir.join("index.js"), &dir, &mut HashSet::new()).unwrap();
    assert_eq!(css.len(), 1);
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_dynamic_import_detection() {
    let dir = temp_dir("dynamic_imports");
    fs::write(dir.join("main.js"), r#"import('./lazy.js').then(m => m.default());"#).unwrap();
    let split = Bundler::compute_chunks(&[dir.join("main.js")], &dir);
    assert!(split.dynamic_imports.iter().any(|d| d.specifier == "./lazy.js"));
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_minifier_callback() {
    let cb = MinifierCallback {
      program: "terser".into(),
      args: vec!["--compress".into()],
    };
    assert_eq!(cb.program, "terser");
  }

  #[test]
  fn test_bundle_result_with_css() {
    let result = BundleResult {
      duration: Duration::from_secs(0),
      output_files: vec![],
      outputs: vec![],
      size_bytes: 0,
      warnings: vec![],
      chunks: None,
      css_files: vec![PathBuf::from("dist/bundle.css")],
    };
    assert_eq!(result.css_files.len(), 1);
  }

  #[test]
  fn test_chunk_with_dynamic_flag() {
    let chunk = Chunk {
      name: "lazy".into(),
      path: PathBuf::from("lazy.js"),
      size: 512,
      entry: false,
      imports: vec![],
      exports: vec![],
      modules: vec![],
      dynamic: true,
    };
    assert!(chunk.dynamic);
  }

  #[test]
  fn test_export_info() {
    let export = ExportInfo {
      name: "foo".into(),
      local_name: "foo".into(),
      is_default: false,
      used: true,
    };
    assert!(export.used);
    assert!(!export.is_default);
  }

  #[test]
  fn test_module_exports() {
    let me = ModuleExports {
      file: "index.js".into(),
      exports: vec![],
      re_exports: vec![],
    };
    assert_eq!(me.file, "index.js");
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
  fn test_bundler_error_types() {
    let e1 = BundlerError::BuildFailed("fail".into());
    let e2 = BundlerError::ToolNotFound("esbuild".into());
    let e3 = BundlerError::TreeShakingError("unused export".into());
    assert!(e1.to_string().contains("fail"));
    assert!(e2.to_string().contains("esbuild"));
    assert!(e3.to_string().contains("unused"));
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
