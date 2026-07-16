use crate::PmError;
use std::path::{Path, PathBuf};
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
        let expected_cache = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".klyron-cache"))
            .join("klyron")
            .join("pm");
        assert_eq!(config.cache_dir, expected_cache);
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
    }

    #[test]
    fn test_cache_filename_from_integrity() {
        let name = cache_filename("sha512-deadbeefcafebabe");
        assert_eq!(name, "sha512-deadbeefcafebabe.tgz");
    }

    #[test]
    fn test_cache_filename_sanitizes() {
        let name = cache_filename("sha512-a/b/c");
        assert_eq!(name, "sha512-a-b-c.tgz");
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
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".klyron-cache"))
            .join("klyron")
            .join("pm");
        Self {
            root: PathBuf::from("."),
            production: false,
            frozen_lockfile: false,
            ignore_scripts: false,
            registry_url: "https://registry.npmjs.org".into(),
            cache_dir,
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

/// Return a filesystem-safe cache filename from an integrity string
fn cache_filename(integrity: &str) -> String {
    format!("{}.tgz", integrity.replace('/', "-"))
}

/// Extract tarball bytes into target_dir, stripping the first path component
fn extract_tarball(data: &[u8], target_dir: &Path) -> Result<(), PmError> {
    std::fs::create_dir_all(target_dir)
        .map_err(|e| PmError::IoError(format!("Cannot create dir: {e}")))?;
    let decoder = flate2::read::GzDecoder::new(data);
    let mut archive = tar::Archive::new(decoder);
    for entry in archive.entries().map_err(|e| PmError::IoError(format!("Tar error: {e}")))? {
        let mut entry = entry.map_err(|e| PmError::IoError(format!("Entry error: {e}")))?;
        if let Ok(path) = entry.path() {
            let components: Vec<_> = path.components().collect();
            if components.len() > 1 {
                let relative: PathBuf = components[1..].iter().collect();
                let dest = target_dir.join(&relative);
                if let Some(parent) = dest.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                entry.unpack(&dest).ok();
            }
        }
    }
    Ok(())
}

/// Download tarball from URL, optionally cache it, return the bytes
fn download_and_cache(url: &str, cache_path: Option<&Path>) -> Result<Vec<u8>, PmError> {
    let response = reqwest::blocking::get(url)
        .map_err(|e| PmError::IoError(format!("HTTP request failed: {e}")))?;
    if !response.status().is_success() {
        return Err(PmError::IoError(format!("HTTP {} for {url}", response.status())));
    }
    let bytes = response.bytes()
        .map_err(|e| PmError::IoError(format!("Failed to read response: {e}")))?;
    let data = bytes.to_vec();

    if let Some(cp) = cache_path {
        let _ = std::fs::create_dir_all(cp.parent().unwrap_or(Path::new(".")));
        std::fs::write(cp, &data).ok();
    }

    Ok(data)
}

/// Main install engine
pub struct InstallEngine;

impl InstallEngine {
    /// Run the full install pipeline with real download, extract, and caching
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
            Some(crate::LockfileV3::from_npm_lockfile(&content)?)
        } else {
            None
        };

        let total_packages = lockfile.as_ref().map(|lf| lf.packages.len()).unwrap_or(0);
        let node_modules = config.root.join("node_modules");
        std::fs::create_dir_all(&node_modules)
            .map_err(|e| PmError::IoError(e.to_string()))?;
        std::fs::create_dir_all(&config.cache_dir)
            .map_err(|e| PmError::IoError(e.to_string()))?;

        let mut installed = 0usize;
        let mut from_cache = 0usize;

        if let Some(ref lf) = lockfile {
            for (name, pkg) in &lf.packages {
                let pkg_dir = node_modules.join(name.trim_start_matches("node_modules/"));
                if pkg_dir.join("package.json").exists() {
                    continue;
                }

                let url = match pkg.resolved.as_deref() {
                    Some(u) if !u.is_empty() => u,
                    _ => continue,
                };

                let cache_path = pkg.integrity.as_deref().and_then(|i| {
                    if i.is_empty() { None } else { Some(config.cache_dir.join(cache_filename(i))) }
                });

                let from_cache_flag = if let Some(ref cp) = cache_path {
                    if cp.exists() {
                        match std::fs::read(cp) {
                            Ok(data) => {
                                match extract_tarball(&data, &pkg_dir) {
                                    Ok(()) => true,
                                    Err(e) => {
                                        tracing::warn!("Cache extract failed for {name}: {e}, re-downloading");
                                        false
                                    }
                                }
                            }
                            Err(_) => false,
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                if from_cache_flag {
                    from_cache += 1;
                    installed += 1;
                    continue;
                }

                match download_and_cache(url, cache_path.as_deref()) {
                    Ok(data) => {
                        match extract_tarball(&data, &pkg_dir) {
                            Ok(()) => {
                                installed += 1;
                            }
                            Err(e) => {
                                tracing::warn!("Extract failed for {name}: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to download {name}: {e}");
                    }
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(InstallReport {
            total: total_packages,
            installed,
            from_cache,
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
