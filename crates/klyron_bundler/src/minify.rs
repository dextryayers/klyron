use std::path::{Path};
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result};

use crate::{BundleOutput, BundleResult, BundlerError};

pub struct Minifier;

impl Minifier {
    pub fn new() -> Self {
        Self
    }

    pub fn minify_file(file: &Path) -> Result<BundleResult> {
        let start = Instant::now();
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("js");

        let result = match ext {
            "js" | "mjs" | "cjs" => Self::minify_js(file, None)?,
            "css" => Self::minify_css(file)?,
            "html" | "htm" => Self::minify_html(file)?,
            _ => {
                let data = std::fs::read(file)?;
                let size = data.len() as u64;
                BundleResult {
                    duration: start.elapsed(),
                    output_files: vec![file.to_path_buf()],
                    outputs: vec![BundleOutput {
                        path: file.to_path_buf(),
                        size,
                        kind: ext.to_string(),
                        integrity: crate::compute_hash(&data),
                    }],
                    size_bytes: size,
                    warnings: Vec::new(),
                    chunks: None,
                    css_files: Vec::new(),
                }
            }
        };

        Ok(result)
    }

    pub fn minify_js(file: &Path, config: Option<&str>) -> Result<BundleResult> {
        let start = Instant::now();
        let output = file.with_extension("min.js");

        let cfg = config.unwrap_or(
            r#"{"jsc":{"minify":{"compress":{"unused":true},"mangle":true},"parser":{"syntax":"ecmascript"},"target":"es2020"}}"#
        );

        let args = vec![
            file.to_string_lossy().to_string(),
            "-o".into(),
            output.to_string_lossy().to_string(),
            "--config".into(),
            cfg.into(),
        ];

        let cmd = if cfg!(target_os = "windows") { "swc.exe" } else { "swc" };

        let output_cmd = Command::new(cmd)
            .args(&args)
            .output()
            .with_context(|| format!("failed to execute minifier for {}", file.display()))?;

        let duration = start.elapsed();

        if !output_cmd.status.success() {
            let stderr = String::from_utf8_lossy(&output_cmd.stderr);
            return Err(BundlerError::MinificationError(stderr.to_string()).into());
        }

        let data = std::fs::read(&output)?;
        let integrity = crate::compute_hash(&data);
        let size = data.len() as u64;

        Ok(BundleResult {
            duration,
            output_files: vec![output.clone()],
            outputs: vec![BundleOutput {
                path: output,
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

    pub fn minify_css(file: &Path) -> Result<BundleResult> {
        let start = Instant::now();
        let data = std::fs::read_to_string(file)?;

        let minified = data
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.starts_with("/*") && !l.is_empty())
            .collect::<Vec<_>>()
            .join("");

        let output = file.with_extension("min.css");
        std::fs::write(&output, &minified)?;

        let size = minified.len() as u64;
        let integrity = crate::compute_hash(minified.as_bytes());

        Ok(BundleResult {
            duration: start.elapsed(),
            output_files: vec![output.clone()],
            outputs: vec![BundleOutput {
                path: output,
                size,
                kind: "css".into(),
                integrity,
            }],
            size_bytes: size,
            warnings: Vec::new(),
            chunks: None,
            css_files: Vec::new(),
        })
    }

    pub fn minify_html(file: &Path) -> Result<BundleResult> {
        let start = Instant::now();
        let data = std::fs::read_to_string(file)?;

        let minified = data
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("");

        let output = file.with_extension("min.html");
        std::fs::write(&output, &minified)?;

        let size = minified.len() as u64;
        let integrity = crate::compute_hash(minified.as_bytes());

        Ok(BundleResult {
            duration: start.elapsed(),
            output_files: vec![output.clone()],
            outputs: vec![BundleOutput {
                path: output,
                size,
                kind: "html".into(),
                integrity,
            }],
            size_bytes: size,
            warnings: Vec::new(),
            chunks: None,
            css_files: Vec::new(),
        })
    }

    pub fn minify_with_callback(file: &Path, cb: &crate::MinifierCallback) -> Result<BundleResult> {
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
        let integrity = crate::compute_hash(&data);
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
}

impl Default for Minifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join("klyron_test_minifier");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_minify_css() {
        let dir = temp_dir();
        let file = dir.join("styles.css");
        fs::write(&file, "body { color: red; }\n  /* comment */\n  .cls { background: blue; }").unwrap();
        let result = Minifier::minify_css(&file).unwrap();
        assert_eq!(result.outputs[0].kind, "css");
        assert!(result.size_bytes > 0);
    }

    #[test]
    fn test_minify_html() {
        let dir = temp_dir();
        let file = dir.join("index.html");
        fs::write(&file, "<html>\n  <head>\n    <title>Test</title>\n  </head>\n</html>").unwrap();
        let result = Minifier::minify_html(&file).unwrap();
        assert_eq!(result.outputs[0].kind, "html");
    }

    #[test]
    fn test_minify_file_unknown_extension() {
        let dir = temp_dir();
        let file = dir.join("data.json");
        fs::write(&file, r#"{"key": "value"}"#).unwrap();
        let result = Minifier::minify_file(&file).unwrap();
        assert_eq!(result.outputs[0].kind, "json");
    }
}
