use crate::pack::PackConfig;
use crate::PmError;
use std::path::PathBuf;
use std::time::Instant;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publish_config_defaults() {
        let config = PublishConfig::default();
        assert_eq!(config.root, PathBuf::from("."));
        assert_eq!(config.registry_url, "https://registry.npmjs.org");
        assert_eq!(config.tag, "latest");
        assert_eq!(config.access, "public");
        assert!(!config.dry_run);
        assert!(config.token.is_none());
    }

    #[test]
    fn test_publish_report_creation() {
        let report = PublishReport {
            package_name: "my-pkg".into(),
            version: "1.0.0".into(),
            tarball_size: 1024,
            registry_url: "https://registry.npmjs.org".into(),
            duration_ms: 200,
        };
        assert_eq!(report.package_name, "my-pkg");
        assert_eq!(report.version, "1.0.0");
        assert_eq!(report.tarball_size, 1024);
    }

    #[test]
    fn test_publish_config_custom() {
        let config = PublishConfig {
            root: "/tmp/pkg".into(),
            registry_url: "https://custom.registry".into(),
            token: Some("mytoken".into()),
            tag: "next".into(),
            access: "restricted".into(),
            dry_run: true,
        };
        assert_eq!(config.tag, "next");
        assert_eq!(config.access, "restricted");
        assert!(config.dry_run);
        assert_eq!(config.token.as_deref(), Some("mytoken"));
    }

    #[test]
    fn test_package_metadata_extraction() {
        let pkg_json = serde_json::json!({
            "name": "test-pkg",
            "version": "2.0.0",
        });
        let name = pkg_json["name"].as_str().unwrap();
        let version = pkg_json["version"].as_str().unwrap();
        assert_eq!(name, "test-pkg");
        assert_eq!(version, "2.0.0");
    }
}

/// Configuration for publishing
#[derive(Debug, Clone)]
pub struct PublishConfig {
    pub root: PathBuf,
    pub registry_url: String,
    pub token: Option<String>,
    pub tag: String,
    pub access: String,
    pub dry_run: bool,
}

impl Default for PublishConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            registry_url: "https://registry.npmjs.org".into(),
            token: None,
            tag: "latest".into(),
            access: "public".into(),
            dry_run: false,
        }
    }
}

/// Result of a publish operation
#[derive(Debug, Clone)]
pub struct PublishReport {
    pub package_name: String,
    pub version: String,
    pub tarball_size: u64,
    pub registry_url: String,
    pub duration_ms: u64,
}

/// Publish a package to a registry
pub async fn publish(config: &PublishConfig) -> Result<PublishReport, PmError> {
    let start = Instant::now();

    let pkg_json_path = config.root.join("package.json");
    let pkg_content = std::fs::read_to_string(&pkg_json_path)
        .map_err(|e| PmError::IoError(format!("Cannot read package.json: {e}")))?;
    let pkg: serde_json::Value = serde_json::from_str(&pkg_content)
        .map_err(|e| PmError::IoError(format!("Invalid package.json: {e}")))?;

    let package_name = pkg["name"].as_str()
        .ok_or_else(|| PmError::PackageNotFound("Missing 'name' in package.json".into()))?
        .to_string();
    let version = pkg["version"].as_str()
        .ok_or_else(|| PmError::VersionNotFound("Missing 'version' in package.json".into()))?
        .to_string();

    let pack_config = PackConfig {
        root: config.root.clone(),
        sign: true,
        ..Default::default()
    };
    let tarball = crate::pack::pack(&pack_config)?;
    let tarball_size = tarball.len() as u64;

    if config.dry_run {
        return Ok(PublishReport {
            package_name,
            version,
            tarball_size,
            registry_url: config.registry_url.clone(),
            duration_ms: start.elapsed().as_millis() as u64,
        });
    }

    let client = reqwest::Client::new();
    let url = format!("{}/{}/{}", config.registry_url.trim_end_matches('/'), &package_name, &version);

    let mut req = client.put(&url)
        .header("Content-Type", "application/octet-stream")
        .body(tarball);

    if let Some(ref token) = config.token {
        req = req.header("Authorization", format!("Bearer {}", token));
    }

    let response = req.send().await
        .map_err(|e| PmError::IoError(format!("Upload failed: {e}")))?;

    if !response.status().is_success() {
        return Err(PmError::IoError(format!(
            "Upload returned status: {}",
            response.status()
        )));
    }

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(PublishReport {
        package_name,
        version,
        tarball_size,
        registry_url: config.registry_url.clone(),
        duration_ms,
    })
}
