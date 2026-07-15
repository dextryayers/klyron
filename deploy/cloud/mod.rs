use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

pub mod aws;
pub mod azure;
pub mod cloudflare;
pub mod gcp;
pub mod heroku;
pub mod netlify;
pub mod vercel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CloudProvider {
    AWS,
    GCP,
    AZURE,
    HEROKU,
    NETLIFY,
    VERCEL,
    CLOUDFLARE,
}

impl CloudProvider {
    pub fn name(&self) -> &str {
        match self {
            Self::AWS => "aws",
            Self::GCP => "gcp",
            Self::AZURE => "azure",
            Self::HEROKU => "heroku",
            Self::NETLIFY => "netlify",
            Self::VERCEL => "vercel",
            Self::CLOUDFLARE => "cloudflare",
        }
    }

    pub fn detect_from_dir(dir: &Path) -> Vec<CloudProvider> {
        let mut detected = Vec::new();
        if dir.join("serverless.yml").exists() || dir.join("serverless.yaml").exists() {
            detected.push(Self::AWS);
        }
        if dir.join("app.yaml").exists() {
            detected.push(Self::GCP);
        }
        if dir.join("azure-functions.json").exists() {
            detected.push(Self::AZURE);
        }
        if dir.join("Procfile").exists() {
            detected.push(Self::HEROKU);
        }
        if dir.join("netlify.toml").exists() {
            detected.push(Self::NETLIFY);
        }
        if dir.join("vercel.json").exists() {
            detected.push(Self::VERCEL);
        }
        if dir.join("wrangler.toml").exists() {
            detected.push(Self::CLOUDFLARE);
        }
        detected
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudDeployConfig {
    pub provider: CloudProvider,
    pub project_name: String,
    pub region: String,
    pub environment: String,
    pub memory_mb: u32,
    pub timeout_seconds: u32,
    pub env_vars: HashMap<String, String>,
    pub secrets: Vec<String>,
    pub vpc_id: Option<String>,
    pub subnets: Vec<String>,
    pub tags: HashMap<String, String>,
}

impl Default for CloudDeployConfig {
    fn default() -> Self {
        Self {
            provider: CloudProvider::AWS,
            project_name: String::new(),
            region: "us-east-1".into(),
            environment: "production".into(),
            memory_mb: 128,
            timeout_seconds: 30,
            env_vars: HashMap::new(),
            secrets: Vec::new(),
            vpc_id: None,
            subnets: Vec::new(),
            tags: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployResult {
    pub success: bool,
    pub url: Option<String>,
    pub version: Option<String>,
    pub deploy_id: String,
    pub logs: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployStatus {
    pub state: DeployState,
    pub message: String,
    pub progress: u8,
    pub started_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployState {
    Pending,
    Building,
    Deploying,
    Healthy,
    RollingBack,
    Failed,
    Cancelled,
}

pub trait CloudDeploy {
    fn validate_config(&self, config: &CloudDeployConfig) -> anyhow::Result<()>;
    fn deploy(&self, config: &CloudDeployConfig) -> anyhow::Result<DeployResult>;
    fn rollback(&self, deploy_id: &str) -> anyhow::Result<DeployResult>;
    fn get_status(&self, deploy_id: &str) -> anyhow::Result<DeployStatus>;
    fn get_logs(&self, deploy_id: &str, since: Option<Duration>) -> anyhow::Result<Vec<String>>;
    fn get_url(&self, deploy_id: &str) -> anyhow::Result<String>;
}

fn validate_common_config(config: &CloudDeployConfig) -> anyhow::Result<()> {
    if config.project_name.is_empty() {
        anyhow::bail!("project_name is required");
    }
    if config.region.is_empty() {
        anyhow::bail!("region is required");
    }
    if config.memory_mb < 128 || config.memory_mb > 10240 {
        anyhow::bail!("memory_mb must be between 128 and 10240");
    }
    if config.timeout_seconds < 1 || config.timeout_seconds > 900 {
        anyhow::bail!("timeout_seconds must be between 1 and 900");
    }
    Ok(())
}
