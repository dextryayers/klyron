use crate::PmError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
