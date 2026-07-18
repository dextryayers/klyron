use std::collections::HashMap;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::DockerClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Binds: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub PortBindings: Option<HashMap<String, Vec<PortBinding>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub RestartPolicy: Option<RestartPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub NetworkMode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Env: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub AutoRemove: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortBinding {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub HostIp: Option<String>,
    #[serde(default)]
    pub HostPort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartPolicy {
    #[serde(default)]
    pub Name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub MaximumRetryCount: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContainerConfig {
    #[serde(default)]
    pub Image: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Cmd: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Entrypoint: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Env: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ExposedPorts: Option<HashMap<String, HashMap<(), ()>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub HostConfig: Option<HostConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Labels: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub WorkingDir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerCreateResponse {
    pub Id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Warnings: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSummary {
    pub Id: String,
    #[serde(default)]
    pub Names: Vec<String>,
    #[serde(default)]
    pub Image: String,
    #[serde(default)]
    pub State: String,
    #[serde(default)]
    pub Status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Ports: Option<Vec<PortMapping>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Created: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub IP: Option<String>,
    #[serde(default)]
    pub PrivatePort: u16,
    #[serde(default)]
    pub PublicPort: u16,
    #[serde(default)]
    pub Type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInspectResponse {
    pub Id: String,
    #[serde(default)]
    pub Name: String,
    #[serde(default)]
    pub State: ContainerState,
    #[serde(default)]
    pub Config: ContainerConfig,
    #[serde(default)]
    pub NetworkSettings: NetworkSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContainerState {
    pub Status: String,
    #[serde(default)]
    pub Running: bool,
    #[serde(default)]
    pub Paused: bool,
    #[serde(default)]
    pub ExitCode: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub StartedAt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub FinishedAt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkSettings {
    #[serde(default)]
    pub Networks: HashMap<String, NetworkInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub IPAddress: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Gateway: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub MacAddress: Option<String>,
}

impl DockerClient {
    pub async fn create_container(&self, config: &ContainerConfig) -> anyhow::Result<ContainerCreateResponse> {
        let endpoint = if let Some(ref name) = config.name {
            format!("/containers/create?name={name}")
        } else {
            "/containers/create".to_string()
        };

        let body = serde_json::to_vec(config)?;
        let resp = self.post(&endpoint, &body).await?;
        let result: ContainerCreateResponse = serde_json::from_slice(&resp)
            .context("Failed to parse container create response")?;
        Ok(result)
    }

    pub async fn start_container(&self, container_id: &str) -> anyhow::Result<()> {
        let endpoint = format!("/containers/{container_id}/start");
        let resp = self.post(&endpoint, &[]).await?;
        let status = self.last_status();
        if status != 204 && status != 304 {
            anyhow::bail!(
                "Failed to start container {container_id}: HTTP {status} - {}",
                String::from_utf8_lossy(&resp)
            );
        }
        Ok(())
    }

    pub async fn stop_container(&self, container_id: &str, timeout: Option<u32>) -> anyhow::Result<()> {
        let endpoint = match timeout {
            Some(t) => format!("/containers/{container_id}/stop?t={t}"),
            None => format!("/containers/{container_id}/stop"),
        };
        let resp = self.post(&endpoint, &[]).await?;
        let status = self.last_status();
        if status != 204 && status != 304 {
            anyhow::bail!(
                "Failed to stop container {container_id}: HTTP {status} - {}",
                String::from_utf8_lossy(&resp)
            );
        }
        Ok(())
    }

    pub async fn restart_container(&self, container_id: &str, timeout: Option<u32>) -> anyhow::Result<()> {
        let endpoint = match timeout {
            Some(t) => format!("/containers/{container_id}/restart?t={t}"),
            None => format!("/containers/{container_id}/restart"),
        };
        let resp = self.post(&endpoint, &[]).await?;
        let status = self.last_status();
        if status != 204 {
            anyhow::bail!(
                "Failed to restart container {container_id}: HTTP {status} - {}",
                String::from_utf8_lossy(&resp)
            );
        }
        Ok(())
    }

    pub async fn remove_container(&self, container_id: &str, force: bool) -> anyhow::Result<()> {
        let mut endpoint = format!("/containers/{container_id}");
        if force {
            endpoint.push_str("?force=true");
        }
        let resp = self.delete(&endpoint).await?;
        let status = self.last_status();
        if status != 204 {
            anyhow::bail!(
                "Failed to remove container {container_id}: HTTP {status} - {}",
                String::from_utf8_lossy(&resp)
            );
        }
        Ok(())
    }

    pub async fn list_containers(&self, all: bool) -> anyhow::Result<Vec<ContainerSummary>> {
        let endpoint = if all {
            "/containers/json?all=true"
        } else {
            "/containers/json"
        };
        let resp = self.get(endpoint).await?;
        let containers: Vec<ContainerSummary> = serde_json::from_slice(&resp)
            .context("Failed to parse container list")?;
        Ok(containers)
    }

    pub async fn inspect_container(&self, container_id: &str) -> anyhow::Result<ContainerInspectResponse> {
        let endpoint = format!("/containers/{container_id}/json");
        let resp = self.get(&endpoint).await?;
        let info: ContainerInspectResponse = serde_json::from_slice(&resp)
            .context("Failed to parse container inspect")?;
        Ok(info)
    }

    pub async fn pause_container(&self, container_id: &str) -> anyhow::Result<()> {
        let endpoint = format!("/containers/{container_id}/pause");
        let resp = self.post(&endpoint, &[]).await?;
        let status = self.last_status();
        if status != 204 {
            anyhow::bail!(
                "Failed to pause container {container_id}: HTTP {status}",
            );
        }
        Ok(())
    }

    pub async fn unpause_container(&self, container_id: &str) -> anyhow::Result<()> {
        let endpoint = format!("/containers/{container_id}/unpause");
        let resp = self.post(&endpoint, &[]).await?;
        let status = self.last_status();
        if status != 204 {
            anyhow::bail!(
                "Failed to unpause container {container_id}: HTTP {status}",
            );
        }
        Ok(())
    }

    pub async fn kill_container(&self, container_id: &str, signal: &str) -> anyhow::Result<()> {
        let endpoint = format!("/containers/{container_id}/kill?signal={signal}");
        let resp = self.post(&endpoint, &[]).await?;
        let status = self.last_status();
        if status != 204 {
            anyhow::bail!(
                "Failed to kill container {container_id}: HTTP {status}",
            );
        }
        Ok(())
    }

    pub async fn container_logs(
        &self,
        container_id: &str,
        stdout: bool,
        stderr: bool,
        tail: &str,
    ) -> anyhow::Result<String> {
        let endpoint = format!(
            "/containers/{container_id}/logs?stdout={stdout}&stderr={stderr}&tail={tail}"
        );
        let resp = self.get(&endpoint).await?;
        Ok(String::from_utf8_lossy(&resp).to_string())
    }
}
