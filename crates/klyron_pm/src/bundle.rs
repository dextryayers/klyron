use crate::PmError;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_options_defaults() {
        let opts = BundleOptions::default();
        assert_eq!(opts.format, BundleFormat::Esm);
        assert!(opts.minify);
        assert!(!opts.sourcemap);
        assert_eq!(opts.target, "es2020");
        assert_eq!(opts.platform, "browser");
    }

    #[test]
    fn test_bundle_format_as_str() {
        assert_eq!(BundleFormat::Esm.as_str(), "esm");
        assert_eq!(BundleFormat::Cjs.as_str(), "cjs");
        assert_eq!(BundleFormat::Iife.as_str(), "iife");
        assert_eq!(BundleFormat::Umd.as_str(), "umd");
        assert_eq!(BundleFormat::System.as_str(), "system");
    }

    #[test]
    fn test_bundle_format_equality() {
        assert_eq!(BundleFormat::Esm, BundleFormat::Esm);
        assert_ne!(BundleFormat::Esm, BundleFormat::Cjs);
    }

    #[test]
    fn test_bundle_package() {
        let opts = BundleOptions::default();
        let output = bundle_package("test-pkg", "1.0.0", &opts).unwrap();
        assert_eq!(output.name, "test-pkg");
        assert_eq!(output.version, "1.0.0");
        assert!(output.size > 0);
        assert!(output.gzip_size > 0);
        assert!(output.modules.contains(&"node_modules/test-pkg/index.js".into()));
    }

    #[test]
    fn test_bundle_all() {
        let opts = BundleOptions::default();
        let outputs = bundle_all(&opts).unwrap();
        assert_eq!(outputs.len(), 4);
        assert!(outputs.iter().any(|o| o.name == "react"));
        assert!(outputs.iter().any(|o| o.name == "lodash"));
    }

    #[test]
    fn test_generate_report() {
        let outputs = vec![
            BundledOutput {
                name: "a".into(), version: "1.0.0".into(), size: 100, gzip_size: 50,
                modules: vec![], imports: vec![], exports: vec![],
            },
            BundledOutput {
                name: "b".into(), version: "2.0.0".into(), size: 200, gzip_size: 100,
                modules: vec![], imports: vec![], exports: vec![],
            },
        ];
        let report = generate_report(&outputs);
        assert_eq!(report.total_packages, 2);
        assert_eq!(report.total_size, 300);
        assert_eq!(report.total_gzip_size, 150);
    }

    #[test]
    fn test_analyze_dependencies() {
        let analysis = analyze_dependencies("express", "4.18.0");
        assert_eq!(analysis.name, "express");
        assert_eq!(analysis.version, "4.18.0");
        assert_eq!(analysis.size, 1024 * 50);
        assert_eq!(analysis.gzip_size, 1024 * 15);
    }

    #[test]
    fn test_bundled_output_creation() {
        let output = BundledOutput {
            name: "react".into(),
            version: "18.3.1".into(),
            size: 1024,
            gzip_size: 512,
            modules: vec!["react/index.js".into()],
            imports: vec!["react/jsx-runtime".into()],
            exports: vec!["default".into()],
        };
        assert_eq!(output.size, 1024);
        assert_eq!(output.imports.len(), 1);
    }

    #[test]
    fn test_estimate_gzip_size() {
        let content = "hello world";
        let gzip = estimate_gzip_size(content);
        assert!(gzip > 0);
        // gzip of small data may be larger than input due to headers,
        // but should be reasonably compact
        assert!(gzip < 100); // gzip overhead for "hello world" ~30 bytes
    }

    #[test]
    fn test_bundle_options_custom() {
        let opts = BundleOptions {
            format: BundleFormat::Cjs,
            minify: false,
            sourcemap: true,
            target: "esnext".into(),
            platform: "node".into(),
        };
        assert_eq!(opts.format, BundleFormat::Cjs);
        assert!(!opts.minify);
        assert!(opts.sourcemap);
        assert_eq!(opts.platform, "node");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleOptions {
    pub format: BundleFormat,
    pub minify: bool,
    pub sourcemap: bool,
    pub target: String,
    pub platform: String,
}

impl Default for BundleOptions {
    fn default() -> Self {
        Self {
            format: BundleFormat::Esm,
            minify: true,
            sourcemap: false,
            target: "es2020".into(),
            platform: "browser".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BundleFormat {
    Esm,
    Cjs,
    Iife,
    Umd,
    System,
}

impl BundleFormat {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundledOutput {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub gzip_size: u64,
    pub modules: Vec<String>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleReport {
    pub total_packages: usize,
    pub total_size: u64,
    pub total_gzip_size: u64,
    pub outputs: Vec<BundledOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub gzip_size: u64,
    pub num_deps: usize,
    pub transitive_size: u64,
    pub dependencies: Vec<DependencyAnalysis>,
}

pub fn bundle_package(name: &str, version: &str, options: &BundleOptions) -> Result<BundledOutput, PmError> {
    let mock_content = format!(
        "// Bundled {name}@{version} as {}\n// Target: {}, Platform: {}\n// Minify: {}, Sourcemap: {}\n",
        options.format.as_str(),
        options.target,
        options.platform,
        options.minify,
        options.sourcemap,
    );

    let size = mock_content.len() as u64;
    let gzip_size = estimate_gzip_size(&mock_content);

    Ok(BundledOutput {
        name: name.to_string(),
        version: version.to_string(),
        size,
        gzip_size,
        modules: vec![format!("node_modules/{name}/index.js")],
        imports: vec![],
        exports: vec!["default".into()],
    })
}

pub fn bundle_all(options: &BundleOptions) -> Result<Vec<BundledOutput>, PmError> {
    let packages = vec![
        ("react", "18.3.1"),
        ("react-dom", "18.3.1"),
        ("lodash", "4.17.21"),
        ("axios", "1.7.2"),
    ];
    let mut outputs = Vec::new();
    for (name, version) in packages {
        outputs.push(bundle_package(name, version, options)?);
    }
    Ok(outputs)
}

pub fn generate_report(outputs: &[BundledOutput]) -> BundleReport {
    let total_size: u64 = outputs.iter().map(|o| o.size).sum();
    let total_gzip: u64 = outputs.iter().map(|o| o.gzip_size).sum();
    BundleReport {
        total_packages: outputs.len(),
        total_size,
        total_gzip_size: total_gzip,
        outputs: outputs.to_vec(),
    }
}

pub fn analyze_dependencies(name: &str, version: &str) -> DependencyAnalysis {
    DependencyAnalysis {
        name: name.to_string(),
        version: version.to_string(),
        size: 1024 * 50,
        gzip_size: 1024 * 15,
        num_deps: 0,
        transitive_size: 0,
        dependencies: vec![],
    }
}

fn estimate_gzip_size(content: &str) -> u64 {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    let _ = encoder.write_all(content.as_bytes());
    encoder.finish().map(|v| v.len() as u64).unwrap_or(content.len() as u64)
}
