use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::DockerClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSummary {
    pub Id: String,
    #[serde(default)]
    pub RepoTags: Vec<String>,
    #[serde(default)]
    pub RepoDigests: Vec<String>,
    #[serde(default)]
    pub Created: String,
    #[serde(default)]
    pub Size: i64,
    #[serde(default)]
    pub VirtualSize: i64,
    #[serde(default)]
    pub Labels: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInspect {
    pub Id: String,
    #[serde(default)]
    pub RepoTags: Vec<String>,
    #[serde(default)]
    pub RepoDigests: Vec<String>,
    #[serde(default)]
    pub Parent: String,
    #[serde(default)]
    pub Comment: String,
    #[serde(default)]
    pub Created: String,
    #[serde(default)]
    pub Container: String,
    #[serde(default)]
    pub DockerVersion: String,
    #[serde(default)]
    pub Architecture: String,
    #[serde(default)]
    pub Os: String,
    #[serde(default)]
    pub Size: i64,
    #[serde(default)]
    pub VirtualSize: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateImageResponse {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub progress: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
}

impl DockerClient {
    pub async fn list_images(&self) -> anyhow::Result<Vec<ImageSummary>> {
        let resp = self.get("/images/json").await?;
        let images: Vec<ImageSummary> =
            serde_json::from_slice(&resp).context("Failed to parse image list")?;
        Ok(images)
    }

    pub async fn pull_image(&self, image: &str, tag: &str) -> anyhow::Result<()> {
        let endpoint = format!("/images/create?fromImage={image}&tag={tag}");
        let resp = self.post(&endpoint, &[]).await?;

        let status = self.last_status();
        if status != 200 {
            anyhow::bail!(
                "Failed to pull image {image}:{tag}: HTTP {status} - {}",
                String::from_utf8_lossy(&resp)
            );
        }

        for line in resp.split(|&b| b == b'\n') {
            if line.is_empty() {
                continue;
            }
            if let Ok(evt) = serde_json::from_slice::<CreateImageResponse>(line) {
                if let Some(err) = evt.error {
                    anyhow::bail!("Pull error for {image}:{tag}: {err}");
                }
            }
        }
        Ok(())
    }

    pub async fn remove_image(&self, image_id: &str, force: bool) -> anyhow::Result<()> {
        let mut endpoint = format!("/images/{image_id}");
        if force {
            endpoint.push_str("?force=true");
        }
        let resp = self.delete(&endpoint).await?;
        let status = self.last_status();
        if status != 200 {
            anyhow::bail!(
                "Failed to remove image {image_id}: HTTP {status} - {}",
                String::from_utf8_lossy(&resp)
            );
        }
        Ok(())
    }

    pub async fn inspect_image(&self, image_id: &str) -> anyhow::Result<ImageInspect> {
        let endpoint = format!("/images/{image_id}/json");
        let resp = self.get(&endpoint).await?;
        let info: ImageInspect =
            serde_json::from_slice(&resp).context("Failed to parse image inspect")?;
        Ok(info)
    }

    pub async fn tag_image(&self, image_id: &str, repo: &str, tag: &str) -> anyhow::Result<()> {
        let endpoint = format!("/images/{image_id}/tag?repo={repo}&tag={tag}");
        let resp = self.post(&endpoint, &[]).await?;
        let status = self.last_status();
        if status != 201 {
            anyhow::bail!(
                "Failed to tag image {image_id}: HTTP {status} - {}",
                String::from_utf8_lossy(&resp)
            );
        }
        Ok(())
    }

    pub async fn prune_images(&self) -> anyhow::Result<serde_json::Value> {
        let resp = self.post("/images/prune?filters={}", &[]).await?;
        let result: serde_json::Value =
            serde_json::from_slice(&resp).context("Failed to parse prune response")?;
        Ok(result)
    }

    pub async fn search_images(&self, term: &str) -> anyhow::Result<serde_json::Value> {
        let endpoint = format!("/images/search?term={term}");
        let resp = self.get(&endpoint).await?;
        let result: serde_json::Value =
            serde_json::from_slice(&resp).context("Failed to parse search results")?;
        Ok(result)
    }

    pub async fn build_image(
        &self,
        tar_archive: &[u8],
        tag: &str,
        dockerfile: &str,
    ) -> anyhow::Result<()> {
        let endpoint = format!("/build?t={tag}&dockerfile={dockerfile}");
        let resp = self.post(&endpoint, tar_archive).await?;

        let status = self.last_status();
        if status != 200 {
            anyhow::bail!(
                "Failed to build image: HTTP {status} - {}",
                String::from_utf8_lossy(&resp)
            );
        }
        Ok(())
    }
}
