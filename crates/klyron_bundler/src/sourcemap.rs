use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use crate::{BundlerError, SourcemapMode};

pub struct SourcemapGenerator {
    source_file: PathBuf,
}

impl SourcemapGenerator {
    pub fn new(source_file: &Path) -> Self {
        Self {
            source_file: source_file.to_path_buf(),
        }
    }

    pub fn generate_external(&self) -> Result<PathBuf> {
        let map_path = self.source_file.with_extension("js.map");

        if map_path.exists() {
            return Ok(map_path);
        }

        let args = vec![
            self.source_file.to_string_lossy().to_string(),
            "--sourcemap".into(),
            format!("--outfile={}", map_path.to_string_lossy()),
        ];

        let output = Command::new("esbuild")
            .args(&args)
            .output()
            .with_context(|| format!("sourcemap generation failed for {}", self.source_file.display()))?;

        if !output.status.success() {
            return Err(BundlerError::SourcemapError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ).into());
        }

        Ok(map_path)
    }

    pub fn generate_inline(&self) -> Result<PathBuf> {
        let args = vec![
            self.source_file.to_string_lossy().to_string(),
            "--sourcemap=inline".into(),
            format!("--outfile={}", self.source_file.to_string_lossy()),
        ];

        let output = Command::new("esbuild")
            .args(&args)
            .output()
            .with_context(|| format!("inline sourcemap failed for {}", self.source_file.display()))?;

        if !output.status.success() {
            return Err(BundlerError::SourcemapError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ).into());
        }

        Ok(self.source_file.clone())
    }

    pub fn generate_sourcemap(source_file: &Path, mode: SourcemapMode) -> Result<PathBuf> {
        let sm_gen = Self::new(source_file);
        match mode {
            SourcemapMode::None => Ok(source_file.to_path_buf()),
            SourcemapMode::Inline => sm_gen.generate_inline(),
            SourcemapMode::External => sm_gen.generate_external(),
        }
    }

    pub fn generate_bundle_sourcemap(outputs: &[PathBuf], out_path: &Path) -> Result<PathBuf> {
        let mut combined_map = SourceMapData {
            version: 3,
            file: out_path.file_name().and_then(|n| n.to_str()).unwrap_or("bundle.js").to_string(),
            sources: Vec::new(),
            sources_content: Vec::new(),
            mappings: String::new(),
            names: Vec::new(),
        };

        for file in outputs {
            if file.exists() {
                if let Ok(content) = std::fs::read_to_string(file) {
                    combined_map.sources.push(
                        file.strip_prefix(out_path.parent().unwrap_or(Path::new("")))
                            .unwrap_or(file)
                            .to_string_lossy()
                            .to_string()
                    );
                    combined_map.sources_content.push(content);
                }
            }
        }

        let map_path = out_path.with_extension("js.map");
        let json = serde_json::to_string_pretty(&combined_map)?;
        std::fs::write(&map_path, json)?;

        Ok(map_path)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SourceMapData {
    pub version: u32,
    pub file: String,
    pub sources: Vec<String>,
    #[serde(rename = "sourcesContent")]
    pub sources_content: Vec<String>,
    pub mappings: String,
    pub names: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join("klyron_test_sourcemap");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_sourcemap_data_serialization() {
        let data = SourceMapData {
            version: 3,
            file: "bundle.js".into(),
            sources: vec!["index.js".into()],
            sources_content: vec!["console.log('hello');".into()],
            mappings: "AAAA".into(),
            names: vec![],
        };
        let json = serde_json::to_string_pretty(&data).unwrap();
        assert!(json.contains("bundle.js"));
        assert!(json.contains("sourcesContent"));
    }

    #[test]
    fn test_generate_bundle_sourcemap() {
        let dir = temp_dir();
        let src1 = dir.join("a.js");
        let src2 = dir.join("b.js");
        fs::write(&src1, "const a = 1;").unwrap();
        fs::write(&src2, "const b = 2;").unwrap();
        let out = dir.join("bundle.js");
        fs::write(&out, "const a=1;const b=2;").unwrap();

        let result = SourcemapGenerator::generate_bundle_sourcemap(
            &[src1, src2],
            &out
        ).unwrap();
        assert!(result.exists());
    }

    #[test]
    fn test_sourcemap_mode_behavior() {
        let dir = temp_dir();
        let file = dir.join("test.js");
        fs::write(&file, "console.log('test');").unwrap();
        let result = SourcemapGenerator::generate_sourcemap(&file, SourcemapMode::None).unwrap();
        assert_eq!(result, file);
    }
}
