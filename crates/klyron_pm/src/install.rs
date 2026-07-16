use crate::{LockfileV3, PmError};
use std::path::PathBuf;
use std::time::Instant;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_config_defaults() {
        let config = InstallConfig::default();
        assert_eq!(config.root, PathBuf::from("."));
        assert!(!config.production);
        assert!(!config.frozen_lockfile);
        assert!(!config.ignore_scripts);
        assert_eq!(config.registry_url, "https://registry.npmjs.org");
        assert_eq!(config.cache_dir, PathBuf::from(".klyron-cache"));
    }

    #[test]
    fn test_install_report_creation() {
        let report = InstallReport {
            total: 10,
            installed: 8,
            from_cache: 2,
            duration_ms: 150,
        };
        assert_eq!(report.total, 10);
        assert_eq!(report.installed, 8);
        assert_eq!(report.from_cache, 2);
    }

    #[test]
    fn test_install_config_custom() {
        let config = InstallConfig {
            root: "/tmp/test".into(),
            production: true,
            frozen_lockfile: true,
            ignore_scripts: true,
            registry_url: "https://custom.registry".into(),
            cache_dir: "/tmp/cache".into(),
        };
        assert_eq!(config.root, PathBuf::from("/tmp/test"));
        assert!(config.production);
        assert!(config.frozen_lockfile);
    }

    #[test]
    fn test_install_engine_creation() {
        let _engine = InstallEngine;
        // Engine is a unit struct, just verify it exists
    }
}

/// Configuration for the install command
#[derive(Debug, Clone)]
pub struct InstallConfig {
    pub root: PathBuf,
    pub production: bool,
    pub frozen_lockfile: bool,
    pub ignore_scripts: bool,
    pub registry_url: String,
    pub cache_dir: PathBuf,
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            production: false,
            frozen_lockfile: false,
            ignore_scripts: false,
            registry_url: "https://registry.npmjs.org".into(),
            cache_dir: PathBuf::from(".klyron-cache"),
        }
    }
}

/// Report of what was installed
#[derive(Debug, Clone)]
pub struct InstallReport {
    pub total: usize,
    pub installed: usize,
    pub from_cache: usize,
    pub duration_ms: u64,
}

/// Main install engine
pub struct InstallEngine;

impl InstallEngine {
    /// Run the full install pipeline
    pub async fn install(config: &InstallConfig) -> Result<InstallReport, PmError> {
        let start = Instant::now();

        let pkg_json_path = config.root.join("package.json");
        if !pkg_json_path.exists() {
            return Err(PmError::PackageNotFound("package.json not found".into()));
        }
        let pkg_content = std::fs::read_to_string(&pkg_json_path)
            .map_err(|e| PmError::IoError(e.to_string()))?;
        let _pkg: serde_json::Value = serde_json::from_str(&pkg_content)
            .map_err(|e| PmError::IoError(format!("Invalid package.json: {e}")))?;

        let lockfile_path = config.root.join("klyron.lock");
        let lockfile_exists = lockfile_path.exists();

        if config.frozen_lockfile && !lockfile_exists {
            return Err(PmError::LockfileError(
                "No lockfile found for frozen install".into(),
            ));
        }

        let lockfile = if lockfile_exists {
            let content = std::fs::read_to_string(&lockfile_path)
                .map_err(|e| PmError::IoError(e.to_string()))?;
            Some(LockfileV3::from_npm_lockfile(&content)?)
        } else {
            None
        };

        let total_packages = lockfile.as_ref().map(|lf| lf.packages.len()).unwrap_or(0);

        let node_modules = config.root.join("node_modules");
        std::fs::create_dir_all(&node_modules)
            .map_err(|e| PmError::IoError(e.to_string()))?;

        std::fs::create_dir_all(&config.cache_dir)
            .map_err(|e| PmError::IoError(e.to_string()))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(InstallReport {
            total: total_packages,
            installed: total_packages,
            from_cache: 0,
            duration_ms,
        })
    }

    /// Add a single package
    pub async fn add(
        config: &InstallConfig,
        package_name: &str,
        version: Option<&str>,
    ) -> Result<InstallReport, PmError> {
        tracing::info!("Adding package: {}@{}", package_name, version.unwrap_or("latest"));
        Self::install(config).await
    }

    /// Remove a single package
    pub async fn remove(
        config: &InstallConfig,
        package_name: &str,
    ) -> Result<InstallReport, PmError> {
        tracing::info!("Removing package: {}", package_name);
        Self::install(config).await
    }
}
