pub mod bundle;
pub mod resolve;
pub mod minify;
pub mod sourcemap;

pub use bundle::{Bundler, compute_hash};
pub use resolve::{resolve_deps, resolve_specifier, analyze_exports, DependencyGraph, ModuleNode};

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use thiserror::Error;

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
    fn test_bundle_options_defaults() {
        let opts = BundleOptions::default();
        assert_eq!(opts.entry, "src/index.js");
        assert_eq!(opts.out_dir, "dist");
        assert!(opts.minify);
        assert!(opts.tree_shaking);
        assert_eq!(opts.format, OutputFormat::Esm);
    }

    #[test]
    fn test_output_format_as_str() {
        assert_eq!(OutputFormat::Esm.as_str(), "esm");
        assert_eq!(OutputFormat::Cjs.as_str(), "cjs");
        assert_eq!(OutputFormat::Iife.as_str(), "iife");
        assert_eq!(OutputFormat::Umd.as_str(), "umd");
        assert_eq!(OutputFormat::System.as_str(), "system");
    }

    #[test]
    fn test_compute_hash() {
        let hash = compute_hash(b"hello");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_detect_vite() {
        let dir = temp_dir("detect_vite");
        fs::write(dir.join("vite.config.ts"), "").unwrap();
        assert_eq!(Bundler::detect(&dir), Some(BundlerKind::Vite));
    }

    #[test]
    fn test_detect_webpack() {
        let dir = temp_dir("detect_webpack");
        fs::write(dir.join("webpack.config.js"), "").unwrap();
        assert_eq!(Bundler::detect(&dir), Some(BundlerKind::Webpack));
    }

    #[test]
    fn test_detect_none() {
        let dir = temp_dir("detect_none");
        assert_eq!(Bundler::detect(&dir), None);
    }

    #[test]
    fn test_css_bundle_options_defaults() {
        let opts = CssBundleOptions::default();
        assert_eq!(opts.entry, "src/styles.css");
        assert!(opts.minify);
        assert!(!opts.sourcemap);
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
    fn test_module_exports() {
        let me = ModuleExports {
            file: "index.js".into(),
            exports: vec![],
            re_exports: vec![],
        };
        assert_eq!(me.file, "index.js");
    }
}
