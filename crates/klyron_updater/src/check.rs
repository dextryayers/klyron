use std::path::Path;

use anyhow::{Context, Result};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::{ReleaseInfo, UpdateChannel, UpdateStatus};

pub struct Updater {
    pub current_version: String,
    pub channel: UpdateChannel,
    pub repo_owner: String,
    pub repo_name: String,
    pub backups_dir: std::path::PathBuf,
    pub install_path: std::path::PathBuf,
    pub client: reqwest::Client,
    pub downloaded_bytes: Arc<AtomicU64>,
    pub is_downloading: Arc<AtomicBool>,
    pub rollout_percentage: u8,
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
            .find(|a| {
                a.name.contains(std::env::consts::OS)
                    && a.name.contains(std::env::consts::ARCH)
            })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_updater_constructor() {
        let updater = Updater::new(
            "0.1.0",
            "dextryayers",
            "klyron",
            Path::new("/usr/local/bin/klyron"),
            UpdateChannel::Stable,
        );
        assert_eq!(updater.current_version(), "0.1.0");
        assert_eq!(updater.channel(), UpdateChannel::Stable);
        assert!(!updater.is_downloading());
    }

    #[test]
    fn test_updater_rollout_percentage() {
        let mut updater = Updater::new(
            "0.1.0", "owner", "repo",
            Path::new("/tmp/klyron"),
            UpdateChannel::Canary,
        );
        updater.set_rollout_percentage(25);
        assert_eq!(updater.rollout_percentage, 25);
        updater.set_rollout_percentage(150);
        assert_eq!(updater.rollout_percentage, 100);
    }

    #[test]
    fn test_updater_download_progress() {
        let updater = Updater::new(
            "0.1.0", "owner", "repo",
            Path::new("/tmp/klyron"),
            UpdateChannel::Stable,
        );
        assert_eq!(updater.download_progress(), 0);
    }
}
