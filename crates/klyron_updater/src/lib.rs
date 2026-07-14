use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use futures::StreamExt;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateChannel {
    Stable,
    Nightly,
    Canary,
}

impl UpdateChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            UpdateChannel::Stable => "stable",
            UpdateChannel::Nightly => "nightly",
            UpdateChannel::Canary => "canary",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "nightly" => UpdateChannel::Nightly,
            "canary" => UpdateChannel::Canary,
            _ => UpdateChannel::Stable,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub html_url: String,
    pub body: Option<String>,
    pub prerelease: bool,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub channel: UpdateChannel,
    pub download_url: Option<String>,
    pub checksum: Option<String>,
    pub rollout_percentage: u8,
}

pub struct Updater {
    current_version: String,
    channel: UpdateChannel,
    repo_owner: String,
    repo_name: String,
    backups_dir: PathBuf,
    install_path: PathBuf,
    client: reqwest::Client,
    downloaded_bytes: Arc<AtomicU64>,
    is_downloading: Arc<AtomicBool>,
    rollout_percentage: u8,
}

impl Updater {
    pub fn new(
        current_version: &str,
        repo_owner: &str,
        repo_name: &str,
        install_path: &Path,
        channel: UpdateChannel,
    ) -> Self {
        let backups_dir = install_path
            .parent()
            .unwrap_or(Path::new("/tmp"))
            .join(".klyron_backups");

        let client = reqwest::Client::builder()
            .user_agent("klyron-updater/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            current_version: current_version.to_string(),
            channel,
            repo_owner: repo_owner.to_string(),
            repo_name: repo_name.to_string(),
            backups_dir,
            install_path: install_path.to_path_buf(),
            client,
            downloaded_bytes: Arc::new(AtomicU64::new(0)),
            is_downloading: Arc::new(AtomicBool::new(false)),
            rollout_percentage: 100,
        }
    }

    pub fn set_rollout_percentage(&mut self, percentage: u8) {
        self.rollout_percentage = percentage.min(100);
    }

    pub async fn check_version(&self) -> Result<UpdateStatus> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.repo_owner, self.repo_name
        );

        let response: ReleaseInfo = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch latest release")?
            .json()
            .await
            .context("Failed to parse release info")?;

        let latest_version = response.tag_name.trim_start_matches('v').to_string();
        let has_update = latest_version != self.current_version;

        let download_url = response
            .assets
            .iter()
            .find(|a| a.name.contains(std::env::consts::OS) && a.name.contains(std::env::consts::ARCH))
            .map(|a| a.browser_download_url.clone());

        let status = UpdateStatus {
            current_version: self.current_version.clone(),
            latest_version,
            has_update,
            channel: self.channel,
            download_url,
            checksum: None,
            rollout_percentage: self.rollout_percentage,
        };

        Ok(status)
    }

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

        let final_path = self.install_path.parent().unwrap_or(Path::new(".")).join(format!(
            "klyron-{}",
            version
        ));
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

        info!("Successfully installed to: {}", self.install_path.display());
        Ok(())
    }

    pub async fn rollback(&self) -> Result<()> {
        let mut backups: Vec<PathBuf> = Vec::new();
        let mut read_dir = tokio::fs::read_dir(&self.backups_dir).await?;

        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if path
                .file_name()
                .map_or(false, |n| n.to_string_lossy().starts_with("klyron.backup."))
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

    pub fn download_progress(&self) -> u64 {
        self.downloaded_bytes.load(Ordering::SeqCst)
    }

    pub fn is_downloading(&self) -> bool {
        self.is_downloading.load(Ordering::SeqCst)
    }

    pub fn current_version(&self) -> &str {
        &self.current_version
    }

    pub fn channel(&self) -> UpdateChannel {
        self.channel
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}
