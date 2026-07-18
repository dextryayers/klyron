use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::resolve::{resolve_deps, DependencyGraph};
use crate::sourcemap::SourcemapGenerator;
use crate::minify::Minifier;
use crate::{BundlerError, BundleOptions, BundleResult, BundleOutput, SourcemapMode, CssBundleOptions, BundlerKind, SplitResult, Chunk, DynamicImport};

pub struct Bundler;

impl Bundler {
    pub fn new() -> Self {
        Self
    }

    pub fn bundle_js(dir: &Path, kind: BundlerKind, opts: BundleOptions) -> Result<BundleResult> {
        let start = Instant::now();
        let out_dir = dir.join(&opts.out_dir);
        let entry_path = dir.join(&opts.entry);

        if !entry_path.exists() {
            return Err(BundlerError::EntryNotFound(opts.entry.clone()).into());
        }

        let dep_graph = resolve_deps(&entry_path)?;

        let (cmd, args) = Self::build_bundler_args(kind, &opts);

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

        let outputs = Self::collect_outputs(&out_dir)?;
        let total_size: u64 = outputs.iter().map(|o| o.size).sum();

        let mut result_outputs = outputs;

        if opts.minify {
            for output_file in &mut result_outputs {
                if output_file.kind == "js" || output_file.kind == "mjs" {
                    let minified = Minifier::minify_file(&output_file.path)?;
                    output_file.size = minified.size_bytes;
                }
            }
        }

        let sourcemap_files = if opts.sourcemap != SourcemapMode::None {
            Self::generate_sourcemaps(&result_outputs, &opts.sourcemap)?
        } else {
            Vec::new()
        };

        let chunks = if opts.splitting {
            let entry_files: Vec<PathBuf> = result_outputs.iter()
                .filter(|o| o.kind == "js" || o.kind == "mjs")
                .map(|o| o.path.clone())
                .collect();
            Some(Self::compute_chunks(&entry_files, dir, &dep_graph))
        } else {
            None
        };

        let all_outputs = result_outputs.into_iter()
            .chain(sourcemap_files.into_iter().map(|p| BundleOutput {
                path: p,
                size: 0,
                kind: "map".into(),
                integrity: String::new(),
            }))
            .collect::<Vec<_>>();

        Ok(BundleResult {
            duration,
            output_files: all_outputs.iter().map(|o| o.path.clone()).collect(),
            outputs: all_outputs,
            size_bytes: total_size,
            warnings,
            chunks,
            css_files: Vec::new(),
        })
    }

    fn build_bundler_args(_kind: BundlerKind, opts: &BundleOptions) -> (String, Vec<String>) {
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

    fn collect_outputs(out_dir: &Path) -> Result<Vec<BundleOutput>> {
        let mut outputs = Vec::new();
        if !out_dir.exists() {
            return Ok(outputs);
        }
        for entry in WalkDir::new(out_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path().to_path_buf();
                if path.extension().map_or(false, |e| e == "map") {
                    continue;
                }
                let metadata = std::fs::metadata(&path)?;
                let data = std::fs::read(&path)?;
                let integrity = compute_hash(&data);
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_string();
                outputs.push(BundleOutput {
                    size: metadata.len(),
                    path,
                    kind: ext,
                    integrity,
                });
            }
        }
        Ok(outputs)
    }

    fn generate_sourcemaps(outputs: &[BundleOutput], mode: &SourcemapMode) -> Result<Vec<PathBuf>> {
        let mut maps = Vec::new();
        for out in outputs {
            if out.kind != "js" && out.kind != "mjs" {
                continue;
            }
            let sm_gen = SourcemapGenerator::new(&out.path);
            let map_path = match mode {
                SourcemapMode::External => sm_gen.generate_external()?,
                SourcemapMode::Inline => sm_gen.generate_inline()?,
                SourcemapMode::None => continue,
            };
            maps.push(map_path);
        }
        Ok(maps)
    }

    fn extract_warnings(stderr: &[u8]) -> Vec<String> {
        let stderr = String::from_utf8_lossy(stderr);
        stderr.lines()
            .filter(|l| l.to_lowercase().contains("warn"))
            .map(|l| l.to_string())
            .collect()
    }

    fn compute_chunks(entry_files: &[PathBuf], dir: &Path, _graph: &DependencyGraph) -> SplitResult {
        use regex::Regex;
        let dynamic_re = Regex::new(r#"import\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap();

        let mut chunks = Vec::new();
        let mut dynamic_imports = Vec::new();
        let mut total_size = 0u64;
        let mut module_counts: HashMap<String, usize> = HashMap::new();
        let mut modules_by_chunk: HashMap<String, Vec<String>> = HashMap::new();

        for entry in entry_files {
            let name = entry.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("chunk")
                .to_string();

            let mut modules = Vec::new();
            if let Ok(content) = std::fs::read_to_string(entry) {
                for (line_idx, line) in content.lines().enumerate() {
                    if let Some(caps) = dynamic_re.captures(line.trim()) {
                        let spec = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                        let chunk_name = spec.trim_end_matches(".js")
                            .trim_end_matches(".ts")
                            .trim_end_matches(".tsx")
                            .trim_end_matches(".jsx")
                            .replace('/', "-");
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
            modules_by_chunk.insert(name.clone(), modules.clone());
            for m in &modules {
                *module_counts.entry(m.clone()).or_default() += 1;
            }
        }

        let shared: Vec<String> = module_counts.iter()
            .filter(|&(_, &c)| c > 1)
            .map(|(k, _)| k.clone())
            .collect();

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

        for dyn_import in &dynamic_imports {
            let chunk_name = &dyn_import.chunk_name;
            if !chunks.iter().any(|c| c.name == *chunk_name) {
                let chunk_path = dir.join(format!("{}.js", chunk_name));
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
            shared_chunks: shared,
            chunks,
            total_size,
            dynamic_imports,
        }
    }

    pub fn bundle_css(dir: &Path, opts: CssBundleOptions) -> Result<BundleResult> {
        let start = Instant::now();
        let args = Self::build_css_args(&opts);

        let cmd = if cfg!(target_os = "windows") { "lightningcss.exe" } else { "lightningcss" };

        let output = Command::new(cmd)
            .args(&args)
            .current_dir(dir)
            .output();

        match output {
            Ok(output) => {
                let duration = start.elapsed();
                let warnings = Self::extract_warnings(&output.stderr);
                let out_dir = dir.join(&opts.out_dir);
                let outputs = Self::collect_outputs(&out_dir)?;
                let total_size: u64 = outputs.iter().map(|o| o.size).sum();
                let css_paths: Vec<PathBuf> = outputs.iter().map(|o| o.path.clone()).collect();

                Ok(BundleResult {
                    duration,
                    output_files: outputs.iter().map(|o| o.path.clone()).collect(),
                    outputs,
                    size_bytes: total_size,
                    warnings,
                    chunks: None,
                    css_files: css_paths,
                })
            }
            Err(_) => {
                let js_opts = BundleOptions {
                    entry: opts.entry,
                    out_dir: opts.out_dir,
                    minify: opts.minify,
                    sourcemap: if opts.sourcemap { SourcemapMode::External } else { SourcemapMode::None },
                    ..Default::default()
                };
                Self::bundle_js(dir, BundlerKind::Esbuild, js_opts)
            }
        }
    }

    fn build_css_args(opts: &CssBundleOptions) -> Vec<String> {
        let mut args = vec![
            "--bundle".into(),
            opts.entry.clone(),
            "-o".into(),
            format!("{}/bundle.css", opts.out_dir),
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
        args
    }

    pub fn detect(dir: &Path) -> Option<BundlerKind> {
        let configs: [(&str, BundlerKind); 11] = [
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
            if glob::glob(&pattern_path).ok().and_then(|mut g| g.next()).is_some() {
                return Some(*kind);
            }
        }
        None
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
            if let Ok(matches) = glob::glob(&pattern_path) {
                for entry in matches.flatten() {
                    return Some(entry);
                }
            }
        }
        None
    }
}

impl Default for Bundler {
    fn default() -> Self {
        Self::new()
    }
}

pub fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}
