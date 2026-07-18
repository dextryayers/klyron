pub mod apply;
pub mod check;

use serde::{Deserialize, Serialize};

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

pub use check::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_channel_stable() {
        assert_eq!(UpdateChannel::Stable.as_str(), "stable");
    }

    #[test]
    fn test_update_channel_nightly() {
        assert_eq!(UpdateChannel::Nightly.as_str(), "nightly");
    }

    #[test]
    fn test_update_channel_canary() {
        assert_eq!(UpdateChannel::Canary.as_str(), "canary");
    }

    #[test]
    fn test_update_channel_from_str() {
        assert_eq!(UpdateChannel::from_str("nightly"), UpdateChannel::Nightly);
        assert_eq!(UpdateChannel::from_str("canary"), UpdateChannel::Canary);
        assert_eq!(UpdateChannel::from_str("stable"), UpdateChannel::Stable);
        assert_eq!(UpdateChannel::from_str("unknown"), UpdateChannel::Stable);
    }

    #[test]
    fn test_update_status_fields() {
        let status = UpdateStatus {
            current_version: "0.1.0".into(),
            latest_version: "0.2.0".into(),
            has_update: true,
            channel: UpdateChannel::Stable,
            download_url: Some("https://example.com/klyron".into()),
            checksum: Some("abc123".into()),
            rollout_percentage: 100,
        };
        assert!(status.has_update);
        assert_eq!(status.current_version, "0.1.0");
        assert_eq!(status.latest_version, "0.2.0");
    }

    #[test]
    fn test_update_status_no_update() {
        let status = UpdateStatus {
            current_version: "1.0.0".into(),
            latest_version: "1.0.0".into(),
            has_update: false,
            channel: UpdateChannel::Nightly,
            download_url: None,
            checksum: None,
            rollout_percentage: 50,
        };
        assert!(!status.has_update);
        assert!(status.download_url.is_none());
    }

    #[test]
    fn test_release_info() {
        let release = ReleaseInfo {
            tag_name: "v1.0.0".into(),
            html_url: "https://github.com/release".into(),
            body: Some("Release notes".into()),
            prerelease: false,
            assets: vec![ReleaseAsset {
                name: "klyron-linux-x64.tar.gz".into(),
                browser_download_url: "https://example.com/download".into(),
                size: 1024000,
            }],
        };
        assert_eq!(release.tag_name, "v1.0.0");
        assert!(!release.prerelease);
        assert_eq!(release.assets.len(), 1);
    }

    #[test]
    fn test_release_asset() {
        let asset = ReleaseAsset {
            name: "klyron-macos-arm64.tar.gz".into(),
            browser_download_url: "https://example.com/klyron-mac.tar.gz".into(),
            size: 2048000,
        };
        assert!(asset.name.contains("macos"));
        assert!(asset.size > 0);
    }
}
