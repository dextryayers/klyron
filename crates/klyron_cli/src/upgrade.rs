use std::path::{Path, PathBuf};

pub struct UpgradeManager {
    current_version: String,
    repo_url: &'static str,
    download_dir: PathBuf,
}

pub struct ReleaseInfo {
    pub version: String,
    pub url: String,
    pub checksum: String,
    pub release_date: String,
    pub changelog: String,
}

impl UpgradeManager {
    pub fn new() -> Self {
        let download_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("klyron")
            .join("updates");
        Self {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            repo_url: "https://api.github.com/repos/dextryayers/klyron",
            download_dir,
        }
    }

    pub fn check_for_updates() -> Result<Option<ReleaseInfo>, String> {
        let url = "https://api.github.com/repos/dextryayers/klyron/releases/latest";
        let resp = ureq::get(url)
            .set("User-Agent", "klyron-updater")
            .set("Accept", "application/vnd.github.v3+json")
            .timeout(std::time::Duration::from_secs(10))
            .call()
            .map_err(|e| format!("Failed to check for updates: {e}"))?;

        let body = resp.into_string().map_err(|e| format!("Failed to read response: {e}"))?;
        let json: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse response: {e}"))?;

        let tag_name = json["tag_name"].as_str().unwrap_or("v0.0.0");
        let version = tag_name.trim_start_matches('v').to_string();
        let current = env!("CARGO_PKG_VERSION");

        if version == current || version_is_lower(&version, current) {
            return Ok(None);
        }

        let assets = json["assets"].as_array().map(|a| {
            a.iter().filter_map(|asset| {
                let name = asset["name"].as_str()?;
                if name.contains(std::env::consts::ARCH) && name.contains(std::env::consts::OS) {
                    Some((
                        asset["browser_download_url"].as_str().unwrap_or("").to_string(),
                        name.to_string(),
                    ))
                } else {
                    None
                }
            }).collect::<Vec<_>>()
        }).unwrap_or_default();

        let (url, _filename) = assets.first().cloned().unwrap_or_else(|| {
            (json["zipball_url"].as_str().unwrap_or("").to_string(), "source.zip".to_string())
        });

        let checksum = json["body"].as_str().unwrap_or("").to_string();
        let release_date = json["published_at"].as_str().unwrap_or("").to_string();
        let changelog = json["body"].as_str().unwrap_or("").to_string();

        Ok(Some(ReleaseInfo {
            version,
            url,
            checksum,
            release_date,
            changelog,
        }))
    }

    pub fn download_update(release: &ReleaseInfo) -> Result<PathBuf, String> {
        let mgr = Self::new();
        std::fs::create_dir_all(&mgr.download_dir).map_err(|e| format!("Cannot create download dir: {e}"))?;

        let filename = release.url.rsplit('/').next().unwrap_or("update.tar.gz");
        let output_path = mgr.download_dir.join(filename);

        let resp = ureq::get(&release.url)
            .set("User-Agent", "klyron-updater")
            .timeout(std::time::Duration::from_secs(120))
            .call()
            .map_err(|e| format!("Download failed: {e}"))?;

        let mut reader = resp.into_reader();
        let mut file = std::fs::File::create(&output_path)
            .map_err(|e| format!("Cannot create file: {e}"))?;
        std::io::copy(&mut reader, &mut file)
            .map_err(|e| format!("Write failed: {e}"))?;

        Ok(output_path)
    }

    pub fn apply_update(download_path: &Path) -> Result<(), String> {
        let backup = Self::backup_current()?;
        let current_exe = std::env::current_exe().map_err(|e| format!("Cannot get current exe: {e}"))?;
        let temp_exe = current_exe.with_extension("new");

        let ext = download_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "gz" | "tgz" | "tar" => {
                let mut archive = flate2::read::GzDecoder::new(std::fs::File::open(download_path)
                    .map_err(|e| format!("Cannot open archive: {e}"))?);
                let mut tar = tar::Archive::new(&mut archive);
                for entry in tar.entries().map_err(|e| format!("Tar error: {e}"))? {
                    let mut entry = entry.map_err(|e| format!("Entry error: {e}"))?;
                    if let Ok(path) = entry.path() {
                        if path.ends_with("klyron") || path.ends_with("klyron.exe") {
                            entry.unpack(&temp_exe).map_err(|e| format!("Cannot unpack: {e}"))?;
                            break;
                        }
                    }
                }
            }
            _ => {
                std::fs::copy(download_path, &temp_exe).map_err(|e| format!("Cannot copy: {e}"))?;
            }
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&temp_exe, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("Cannot set permissions: {e}"))?;
        }

        std::fs::rename(&temp_exe, &current_exe).map_err(|e| format!("Cannot replace binary: {e}"))?;

        let _ = std::fs::remove_file(&backup);
        Ok(())
    }

    pub fn verify_integrity(download_path: &Path, expected_hash: &str) -> bool {
        use sha2::{Sha256, Digest};
        let data = std::fs::read(download_path).ok();
        match data {
            Some(bytes) => {
                let mut hasher = Sha256::new();
                hasher.update(&bytes);
                let computed = hex::encode(hasher.finalize());
                computed == expected_hash
            }
            None => false,
        }
    }

    pub fn rollback() -> Result<(), String> {
        let mgr = Self::new();
        let backup_path = mgr.download_dir.join("klyron.backup");
        if !backup_path.exists() {
            return Err("No backup found to rollback to".into());
        }
        let current_exe = std::env::current_exe().map_err(|e| format!("Cannot get current exe: {e}"))?;
        std::fs::rename(&backup_path, &current_exe).map_err(|e| format!("Rollback failed: {e}"))?;
        Ok(())
    }

    pub fn backup_current() -> Result<PathBuf, String> {
        let mgr = Self::new();
        std::fs::create_dir_all(&mgr.download_dir).map_err(|e| e.to_string())?;
        let current_exe = std::env::current_exe().map_err(|e| format!("Cannot get current exe: {e}"))?;
        let backup_path = mgr.download_dir.join("klyron.backup");
        std::fs::copy(&current_exe, &backup_path).map_err(|e| format!("Backup failed: {e}"))?;
        Ok(backup_path)
    }

    pub fn get_current_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    pub fn get_latest_version() -> Result<String, String> {
        let release = Self::check_for_updates()?;
        Ok(release.map(|r| r.version).unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string()))
    }

    pub fn is_update_available() -> Result<bool, String> {
        Ok(Self::check_for_updates()?.is_some())
    }

    pub fn interactive_upgrade() -> Result<(), String> {
        println!("Current version: v{}", Self::get_current_version());

        let release = match Self::check_for_updates()? {
            Some(r) => r,
            None => {
                println!("You are already running the latest version.");
                return Ok(());
            }
        };

        println!("New version available: v{}", release.version);
        println!("Release date: {}", release.release_date);
        println!("\nChangelog:\n{}", release.changelog);
        println!();

        let _backup = Self::backup_current()?;
        println!("Downloading v{}...", release.version);
        let download_path = Self::download_update(&release)?;
        println!("Downloaded to: {}", download_path.display());

        if !release.checksum.is_empty() {
            println!("Verifying integrity...");
            if Self::verify_integrity(&download_path, &release.checksum) {
                println!("Integrity check passed.");
            } else {
                return Err("Integrity check failed! Aborting.".into());
            }
        }

        println!("Applying update...");
        Self::apply_update(&download_path)?;
        println!("Update applied successfully! Restart klyron to use the new version.");

        Ok(())
    }
}

impl Default for UpgradeManager {
    fn default() -> Self {
        Self::new()
    }
}

fn version_is_lower(a: &str, b: &str) -> bool {
    let a_parts: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
    let b_parts: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();
    for i in 0..a_parts.len().max(b_parts.len()) {
        let av = a_parts.get(i).copied().unwrap_or(0);
        let bv = b_parts.get(i).copied().unwrap_or(0);
        if av != bv { return av < bv; }
    }
    false
}
