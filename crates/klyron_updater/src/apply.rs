use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use anyhow::{Context, Result};
use futures::StreamExt;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::ReleaseInfo;
use crate::check::Updater;

impl Updater {
    pub async fn download(&self, version: &str) -> Result<PathBuf> {
        self.is_downloading.store(true, Ordering::SeqCst);

        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/tags/v{}",
            self.repo_owner, self.repo_name, version
        );

        let release: ReleaseInfo = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch release info")?
            .json()
            .await?;

        let asset = release
            .assets
            .iter()
            .find(|a| {
                a.name.contains(std::env::consts::OS)
                    && a.name.contains(std::env::consts::ARCH)
            })
            .ok_or_else(|| anyhow::anyhow!("No matching asset found for this platform"))?;

        info!("Downloading {} ({})", asset.name, asset.size);

        let response = self
            .client
            .get(asset.browser_download_url.as_str())
            .send()
            .await
            .context("Failed to download binary")?;

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;

        let tmp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
        let tmp_path = tmp_dir.path().join(&asset.name);

        let mut file = tokio::fs::File::create(&tmp_path)
            .await
            .context("Failed to create temp file")?;

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Error while downloading")?;
            file.write_all(&chunk)
                .await
                .context("Failed to write to temp file")?;
            downloaded += chunk.len() as u64;
            self.downloaded_bytes.store(downloaded, Ordering::SeqCst);

            if total_size > 0 {
                let progress = (downloaded as f64 / total_size as f64) * 100.0;
                info!("Download progress: {:.1}%", progress);
            }
        }

        file.flush().await?;

        let checksum = self.verify_checksum(&tmp_path).await?;
        info!("Checksum verified: {}", checksum);

        self.is_downloading.store(false, Ordering::SeqCst);

        let final_path = self
            .install_path
            .parent()
            .unwrap_or(Path::new("."))
            .join(format!("klyron-{}", version));
        tokio::fs::rename(&tmp_path, &final_path).await?;
        tmp_dir.close().ok();

        let _ = checksum;
        Ok(final_path)
    }

    pub async fn install(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(anyhow::anyhow!("Binary not found at: {}", path.display()));
        }

        info!("Installing from: {}", path.display());

        if self.install_path.exists() {
            let timestamp = chrono_now();
            let backup_name = format!("klyron.backup.{}", timestamp);
            let backup_path = self.backups_dir.join(&backup_name);

            tokio::fs::create_dir_all(&self.backups_dir).await?;
            tokio::fs::rename(&self.install_path, &backup_path)
                .await
                .context("Failed to create backup")?;

            info!("Backed up current binary to: {}", backup_path.display());
        }

        tokio::fs::rename(path, &self.install_path)
            .await
            .context("Failed to install binary")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perm = std::fs::Permissions::from_mode(0o755);
            tokio::fs::set_permissions(&self.install_path, perm).await?;
        }

        info!(
            "Successfully installed to: {}",
            self.install_path.display()
        );
        Ok(())
    }

    pub async fn rollback(&self) -> Result<()> {
        let mut backups: Vec<PathBuf> = Vec::new();
        let mut read_dir = tokio::fs::read_dir(&self.backups_dir).await?;

        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if path
                .file_name()
                .map_or(false, |n| {
                    n.to_string_lossy().starts_with("klyron.backup.")
                })
            {
                backups.push(path);
            }
        }

        backups.sort();
        backups.reverse();

        let latest_backup = backups
            .first()
            .ok_or_else(|| anyhow::anyhow!("No backup found for rollback"))?;

        info!("Rolling back to: {}", latest_backup.display());
        tokio::fs::rename(latest_backup, &self.install_path)
            .await
            .context("Failed to restore backup")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perm = std::fs::Permissions::from_mode(0o755);
            tokio::fs::set_permissions(&self.install_path, perm).await?;
        }

        info!("Rollback successful");
        Ok(())
    }

    pub async fn verify_checksum(&self, path: &Path) -> Result<String> {
        let data = tokio::fs::read(path)
            .await
            .context("Failed to read file for checksum")?;

        let hash = Sha256::digest(&data);
        let checksum = format!("{:x}", hash);

        let checksum_path = path.with_extension("sha256");
        if checksum_path.exists() {
            let expected = tokio::fs::read_to_string(&checksum_path)
                .await
                .context("Failed to read checksum file")?;
            let expected = expected.trim();
            if checksum != expected {
                return Err(anyhow::anyhow!(
                    "Checksum mismatch: got {} expected {}",
                    checksum,
                    expected
                ));
            }
        }

        Ok(checksum)
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UpdateChannel;

    #[test]
    fn test_chrono_now_format() {
        let now = chrono_now();
        assert!(!now.is_empty());
        assert!(now.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_update_channel_equality() {
        assert_eq!(UpdateChannel::Stable, UpdateChannel::Stable);
        assert_ne!(UpdateChannel::Stable, UpdateChannel::Nightly);
    }
}
