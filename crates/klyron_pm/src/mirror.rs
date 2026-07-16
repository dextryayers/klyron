use crate::PmError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirror_metadata_creation() {
        let mut packages = HashMap::new();
        packages.insert("lodash".into(), MirrorPackage {
            name: "lodash".into(),
            versions: vec!["4.17.21".into()],
            updated_at: "2024-01-01".into(),
        });
        let metadata = MirrorMetadata {
            registry_url: "https://registry.npmjs.org".into(),
            last_sync: "2024-06-15T10:00:00Z".into(),
            packages,
        };
        assert_eq!(metadata.registry_url, "https://registry.npmjs.org");
        assert_eq!(metadata.packages.len(), 1);
        assert_eq!(metadata.packages["lodash"].versions, vec!["4.17.21"]);
    }

    #[test]
    fn test_mirror_package_creation() {
        let pkg = MirrorPackage {
            name: "react".into(),
            versions: vec!["18.0.0".into(), "18.1.0".into()],
            updated_at: "2024-03-01".into(),
        };
        assert_eq!(pkg.name, "react");
        assert_eq!(pkg.versions.len(), 2);
    }

    #[test]
    fn test_mirror_meta_path() {
        let dir = std::path::Path::new("/tmp/mirror");
        let path = mirror_meta_path(dir);
        assert_eq!(path, std::path::PathBuf::from("/tmp/mirror/mirror.json"));
    }

    #[test]
    fn test_url_rewriting() {
        let registry_url = "https://registry.npmjs.org";
        let name = "lodash";
        let version = "4.17.21";
        let tarball_url = format!("{}/{}/-/{}-{}.tgz", registry_url.trim_end_matches('/'), name, name, version);
        assert_eq!(tarball_url, "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz");
    }

    #[test]
    fn test_metadata_url() {
        let registry_url = "https://registry.npmjs.org/";
        let name = "express";
        let metadata_url = format!("{}/{}", registry_url.trim_end_matches('/'), name);
        assert_eq!(metadata_url, "https://registry.npmjs.org/express");
    }

    #[test]
    fn test_chrono_now_iso_format() {
        let iso = chrono_now_iso();
        assert_eq!(iso.len(), 20); // YYYY-MM-DDTHH:MM:SSZ
        assert!(iso.ends_with('Z'));
        assert_eq!(&iso[4..5], "-");
        assert_eq!(&iso[7..8], "-");
        assert_eq!(&iso[13..14], ":");
    }

    #[test]
    fn test_tarball_path_construction() {
        let mirror_dir = std::path::Path::new("/mirror");
        let package_name = "lodash";
        let version = "4.17.21";
        let tarball_path = mirror_dir.join(package_name).join(format!("{package_name}-{version}.tgz"));
        assert_eq!(
            tarball_path.to_string_lossy(),
            "/mirror/lodash/lodash-4.17.21.tgz"
        );
    }

    #[test]
    fn test_install_from_mirror_not_found() {
        let tmp = std::env::temp_dir().join("klyron_mirror_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let result = install_from_mirror("nonexistent", "1.0.0", &tmp, &tmp);
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MirrorMetadata {
    pub registry_url: String,
    pub last_sync: String,
    pub packages: HashMap<String, MirrorPackage>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MirrorPackage {
    pub name: String,
    pub versions: Vec<String>,
    pub updated_at: String,
}

fn mirror_meta_path(mirror_dir: &Path) -> PathBuf {
    mirror_dir.join("mirror.json")
}

pub fn mirror_registry(registry_url: &str, dest_dir: &Path) -> Result<MirrorMetadata, PmError> {
    std::fs::create_dir_all(dest_dir)?;

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| PmError::IoError(format!("HTTP client: {e}")))?;

    let packages_url = format!("{}/-/v1/search?text=&size=250", registry_url.trim_end_matches('/'));
    let resp = client.get(&packages_url)
        .send()
        .map_err(|e| PmError::IoError(format!("Search request failed: {e}")))?;

    let body: serde_json::Value = resp.json()
        .map_err(|e| PmError::IoError(format!("Parse response: {e}")))?;

    let mut packages = HashMap::new();
    if let Some(objects) = body.get("objects").and_then(|o| o.as_array()) {
        for obj in objects {
            if let Some(pkg) = obj.get("package") {
                let name = pkg.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                let version = pkg.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();
                let updated = pkg.get("date").and_then(|d| d.as_str()).unwrap_or("").to_string();

                if !name.is_empty() {
                    let pkg_dir = dest_dir.join(&name);
                    std::fs::create_dir_all(&pkg_dir)?;
                    let tarball_url = format!("{}/{}/-/{}-{}.tgz", registry_url.trim_end_matches('/'), name, name, version);
                    let tarball_path = pkg_dir.join(format!("{name}-{version}.tgz"));
                    if !tarball_path.exists() {
                        if let Ok(tarball_resp) = client.get(&tarball_url).send() {
                            if let Ok(bytes) = tarball_resp.bytes() {
                                let _ = std::fs::write(&tarball_path, &bytes);
                            }
                        }
                    }

                    let meta_path = pkg_dir.join("package.json");
                    if !meta_path.exists() {
                        if let Ok(meta_resp) = client.get(&format!("{}/{}", registry_url.trim_end_matches('/'), name)).send() {
                            if let Ok(meta_body) = meta_resp.bytes() {
                                let _ = std::fs::write(&meta_path, &meta_body);
                            }
                        }
                    }

                    packages.insert(name.clone(), MirrorPackage {
                        name,
                        versions: vec![version],
                        updated_at: updated,
                    });
                }
            }
        }
    }

    let metadata = MirrorMetadata {
        registry_url: registry_url.to_string(),
        last_sync: chrono_now_iso(),
        packages,
    };

    let meta_path = mirror_meta_path(dest_dir);
    std::fs::write(&meta_path, serde_json::to_string_pretty(&metadata)?)?;

    Ok(metadata)
}

pub fn update_mirror(mirror_dir: &Path) -> Result<MirrorMetadata, PmError> {
    let meta_path = mirror_meta_path(mirror_dir);
    let content = std::fs::read_to_string(&meta_path)?;
    let metadata: MirrorMetadata = serde_json::from_str(&content)?;
    mirror_registry(&metadata.registry_url, mirror_dir)
}

pub fn install_from_mirror(package_name: &str, version: &str, mirror_dir: &Path, target_dir: &Path) -> Result<PathBuf, PmError> {
    let tarball_path = mirror_dir.join(package_name).join(format!("{package_name}-{version}.tgz"));
    if !tarball_path.exists() {
        return Err(PmError::PackageNotFound(format!(
            "{package_name}@{version} not found in mirror at {}",
            tarball_path.display()
        )));
    }

    let target_pkg_dir = target_dir.join("node_modules").join(package_name);
    std::fs::create_dir_all(&target_pkg_dir)?;

    let tarball_data = std::fs::read(&tarball_path)?;
    let decoder = flate2::read::GzDecoder::new(&tarball_data[..]);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(&target_pkg_dir)
        .map_err(|e| PmError::IoError(format!("Failed to extract tarball: {e}")))?;

    Ok(target_pkg_dir)
}

fn chrono_now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs();
    let secs_per_day = 86400;
    let days = secs / secs_per_day;
    let time_secs = secs % secs_per_day;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;
    let mut year = 1970i64;
    let mut remaining = days as i64;
    loop {
        let dim = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 { 366 } else { 365 };
        if remaining < dim { break; }
        remaining -= dim;
        year += 1;
    }
    let md = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1;
    for &d in &md {
        if remaining < d { break; }
        remaining -= d;
        month += 1;
    }
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, remaining + 1, hours, minutes, seconds)
}
