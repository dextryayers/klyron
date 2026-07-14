use anyhow::{Context, Result};
use glob::glob;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

pub struct Bundler;

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

#[derive(Debug, Clone)]
pub struct BundleOptions {
    pub entry: String,
    pub out_dir: String,
    pub minify: bool,
    pub sourcemap: bool,
    pub format: OutputFormat,
}

impl Default for BundleOptions {
    fn default() -> Self {
        Self {
            entry: "src/index.js".into(),
            out_dir: "dist".into(),
            minify: true,
            sourcemap: false,
            format: OutputFormat::Esm,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Esm,
    Cjs,
    Iife,
}

#[derive(Debug, Clone)]
pub struct BundleResult {
    pub duration: Duration,
    pub output_files: Vec<PathBuf>,
    pub size_bytes: u64,
    pub warnings: Vec<String>,
}

impl Bundler {
    pub fn new() -> Self {
        Self
    }

    pub fn detect(dir: &Path) -> Option<BundlerKind> {
        let configs: [(&str, BundlerKind); 11] = [
            ("vite.config.*", BundlerKind::Vite),
            ("next.config.*", BundlerKind::Turbopack),
            ("esbuild.config.*", BundlerKind::Esbuild),
            ("esbuild.*", BundlerKind::Esbuild),
            ("webpack.config.*", BundlerKind::Webpack),
            ("rollup.config.*", BundlerKind::Rollup),
            ("rollup.config.*", BundlerKind::Rollup),
            ("parcelrc", BundlerKind::Parcel),
            (".parcelrc", BundlerKind::Parcel),
            ("rsbuild.config.*", BundlerKind::Rsbuild),
            (".swcrc", BundlerKind::SWC),
        ];
        for (pattern, kind) in &configs {
            let pattern_path = dir.join(pattern).to_string_lossy().to_string();
            if glob(&pattern_path).ok().and_then(|mut g| g.next()).is_some() {
                return Some(*kind);
            }
        }
        None
    }

    pub fn bundle(dir: &Path, kind: BundlerKind, opts: BundleOptions) -> Result<BundleResult> {
        let (cmd, args) = Self::get_build_command(kind, &opts);
        let start = Instant::now();
        let output = Command::new(&cmd)
            .args(&args)
            .current_dir(dir)
            .output()
            .with_context(|| format!("failed to execute bundler command: {}", cmd))?;
        let duration = start.elapsed();
        let mut warnings = Vec::new();
        let stderr = String::from_utf8_lossy(&output.stderr);
        for line in stderr.lines() {
            if line.to_lowercase().contains("warn") {
                warnings.push(line.to_string());
            }
        }
        let out_dir = dir.join(&opts.out_dir);
        let mut output_files = Vec::new();
        let mut size_bytes = 0u64;
        if out_dir.exists() {
            Self::collect_files(&out_dir, &mut output_files, &mut size_bytes);
        }
        Ok(BundleResult {
            duration,
            output_files,
            size_bytes,
            warnings,
        })
    }

    fn collect_files(dir: &Path, files: &mut Vec<PathBuf>, size: &mut u64) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::collect_files(&path, files, size);
                } else if let Ok(meta) = path.metadata() {
                    files.push(path);
                    *size += meta.len();
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
            BundlerKind::Esbuild => {
                let mut args = vec![
                    "--bundle".into(),
                    opts.entry.clone(),
                    format!("--outdir={}", opts.out_dir),
                ];
                if opts.minify {
                    args.push("--minify".into());
                }
                if opts.sourcemap {
                    args.push("--sourcemap".into());
                }
                match opts.format {
                    OutputFormat::Esm => args.push("--format=esm".into()),
                    OutputFormat::Cjs => args.push("--format=cjs".into()),
                    OutputFormat::Iife => args.push("--format=iife".into()),
                }
                ("esbuild".into(), args)
            }
            BundlerKind::Vite => {
                let mut args = vec!["build".into(), "--outDir".into(), opts.out_dir.clone()];
                if opts.minify {
                    args.push("--minify".into());
                }
                if opts.sourcemap {
                    args.push("--sourcemap".into());
                }
                match opts.format {
                    OutputFormat::Esm => {}
                    OutputFormat::Cjs => {
                        args.push("--ssr".into());
                    }
                    OutputFormat::Iife => {}
                }
                ("vite".into(), args)
            }
            BundlerKind::Webpack => {
                let mut args = vec![
                    "--mode".into(),
                    if opts.minify {
                        "production"
                    } else {
                        "development"
                    }
                    .into(),
                ];
                if opts.sourcemap {
                    args.push("--devtool".into());
                    args.push("source-map".into());
                }
                ("webpack".into(), args)
            }
            BundlerKind::Rollup => {
                let mut args = vec!["-c".into()];
                if opts.sourcemap {
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
                if !opts.sourcemap {
                    args.push("--no-source-maps".into());
                }
                ("parcel".into(), args)
            }
            BundlerKind::Turbopack => {
                let args = vec!["build".into()];
                ("next".into(), args)
            }
            BundlerKind::Rsbuild => {
                let mut args = vec!["build".into()];
                if opts.sourcemap {
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
                if opts.sourcemap {
                    args.push("-s".into());
                }
                ("swc".into(), args)
            }
        }
    }
}

impl Default for Bundler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_detect_vite() {
        let dir = std::env::temp_dir().join("klyron_test_bundler_vite_1");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("vite.config.ts"), "").unwrap();
        assert_eq!(Bundler::detect(&dir), Some(BundlerKind::Vite));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_webpack() {
        let dir = std::env::temp_dir().join("klyron_test_bundler_webpack");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("webpack.config.js"), "").unwrap();
        assert_eq!(Bundler::detect(&dir), Some(BundlerKind::Webpack));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_esbuild() {
        let dir = std::env::temp_dir().join("klyron_test_bundler_esbuild");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("esbuild.config.mjs"), "").unwrap();
        assert_eq!(Bundler::detect(&dir), Some(BundlerKind::Esbuild));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_parcel() {
        let dir = std::env::temp_dir().join("klyron_test_bundler_parcel");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(".parcelrc"), "{}").unwrap();
        assert_eq!(Bundler::detect(&dir), Some(BundlerKind::Parcel));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_none() {
        let dir = std::env::temp_dir().join("klyron_test_bundler_none");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        assert_eq!(Bundler::detect(&dir), None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_get_config_path_vite() {
        let dir = std::env::temp_dir().join("klyron_test_config_vite");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("vite.config.ts"), "").unwrap();
        let cfg = Bundler::get_config_path(&dir, BundlerKind::Vite);
        assert!(cfg.is_some());
        assert!(cfg.unwrap().ends_with("vite.config.ts"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_get_config_path_swc() {
        let dir = std::env::temp_dir().join("klyron_test_config_swc");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
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
            sourcemap: true,
            format: OutputFormat::Esm,
        };
        let (cmd, args) = Bundler::get_build_command(BundlerKind::Esbuild, &opts);
        assert_eq!(cmd, "esbuild");
        assert!(args.contains(&"--bundle".into()));
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
            sourcemap: false,
            format: OutputFormat::Cjs,
        };
        let (cmd, args) = Bundler::get_build_command(BundlerKind::Vite, &opts);
        assert_eq!(cmd, "vite");
        assert!(args.contains(&"--ssr".into()));
    }

    #[test]
    fn test_bundle_options_defaults() {
        let opts = BundleOptions::default();
        assert_eq!(opts.entry, "src/index.js");
        assert_eq!(opts.out_dir, "dist");
        assert!(opts.minify);
        assert!(!opts.sourcemap);
        assert_eq!(opts.format, OutputFormat::Esm);
    }
}
