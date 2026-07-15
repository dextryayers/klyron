use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    pub project_name: String,
    pub deploy_target: DeployTarget,
    pub region: String,
    pub memory: usize,
    pub timeout: u32,
    pub environment: HashMap<String, String>,
    pub secrets: Vec<String>,
    pub vpc_config: Option<VpcConfig>,
    pub health_check: HealthCheckConfig,
    pub scaling: ScalingConfig,
    pub domains: Vec<String>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeployTarget {
    AwsLambda,
    AwsEcs,
    AwsEks,
    GcpCloudRun,
    GcpGke,
    GcpFunctions,
    AzureFunctions,
    AzureAci,
    AzureAks,
    Heroku,
    Netlify,
    Vercel,
    CloudflareWorkers,
    Kubernetes,
    Docker,
}

impl DeployTarget {
    pub fn name(&self) -> &str {
        match self {
            Self::AwsLambda => "aws-lambda",
            Self::AwsEcs => "aws-ecs",
            Self::AwsEks => "aws-eks",
            Self::GcpCloudRun => "gcp-cloud-run",
            Self::GcpGke => "gcp-gke",
            Self::GcpFunctions => "gcp-functions",
            Self::AzureFunctions => "azure-functions",
            Self::AzureAci => "azure-aci",
            Self::AzureAks => "azure-aks",
            Self::Heroku => "heroku",
            Self::Netlify => "netlify",
            Self::Vercel => "vercel",
            Self::CloudflareWorkers => "cloudflare-workers",
            Self::Kubernetes => "kubernetes",
            Self::Docker => "docker",
        }
    }

    pub fn is_serverless(&self) -> bool {
        matches!(
            self,
            Self::AwsLambda
                | Self::GcpFunctions
                | Self::AzureFunctions
                | Self::CloudflareWorkers
                | Self::Netlify
                | Self::Vercel
        )
    }

    pub fn is_containerized(&self) -> bool {
        matches!(
            self,
            Self::AwsEcs | Self::AwsEks | Self::GcpCloudRun | Self::GcpGke | Self::AzureAci | Self::AzureAks | Self::Docker
        )
    }

    pub fn requires_dockerfile(&self) -> bool {
        self.is_containerized() || matches!(self, Self::Heroku | Self::Kubernetes)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpcConfig {
    pub subnet_ids: Vec<String>,
    pub security_group_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub path: String,
    pub port: u16,
    pub interval: u32,
    pub timeout: u32,
    pub healthy_threshold: u32,
    pub unhealthy_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            path: "/health".into(),
            port: 3000,
            interval: 30,
            timeout: 5,
            healthy_threshold: 2,
            unhealthy_threshold: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    pub min_instances: u32,
    pub max_instances: u32,
    pub target_cpu_utilization: f32,
    pub target_memory_utilization: f32,
    pub scaling_policy: ScalingPolicy,
}

impl Default for ScalingConfig {
    fn default() -> Self {
        Self {
            min_instances: 1,
            max_instances: 10,
            target_cpu_utilization: 70.0,
            target_memory_utilization: 80.0,
            scaling_policy: ScalingPolicy::Cpu,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingPolicy {
    Cpu,
    Memory,
    ConcurrentRequests,
    Schedule,
}

impl DeployConfig {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Cannot read config file: {e}"))?;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "yaml" | "yml" => serde_yaml::from_str(&content)
                .map_err(|e| format!("YAML parse error: {e}")),
            "json" => serde_json::from_str(&content)
                .map_err(|e| format!("JSON parse error: {e}")),
            "toml" => toml::from_str(&content)
                .map_err(|e| format!("TOML parse error: {e}")),
            _ => Err("Unsupported config format. Use .yaml, .json, or .toml".into()),
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let ext = path.extension().and_then(|e| e.str()).unwrap_or("");
        let content = match ext {
            "yaml" | "yml" => serde_yaml::to_string(self)
                .map_err(|e| format!("YAML serialize error: {e}"))?,
            "json" => serde_json::to_string_pretty(self)
                .map_err(|e| format!("JSON serialize error: {e}"))?,
            "toml" => toml::to_string_pretty(self)
                .map_err(|e| format!("TOML serialize error: {e}"))?,
            _ => return Err("Unsupported config format. Use .yaml, .json, or .toml".into()),
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create parent directory: {e}"))?;
        }
        fs::write(path, &content)
            .map_err(|e| format!("Cannot write config file: {e}"))
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.project_name.is_empty() {
            errors.push("project_name is required".into());
        }
        if self.region.is_empty() {
            errors.push("region is required".into());
        }
        if self.memory < 128 {
            errors.push("memory must be at least 128 MB".into());
        }
        if self.timeout < 1 || self.timeout > 900 {
            errors.push("timeout must be between 1 and 900 seconds".into());
        }
        if self.health_check.path.is_empty() {
            errors.push("health_check.path is required".into());
        }
        if self.health_check.port == 0 {
            errors.push("health_check.port must be > 0".into());
        }
        if self.scaling.min_instances > self.scaling.max_instances {
            errors.push("scaling.min_instances must be <= max_instances".into());
        }
        if self.scaling.target_cpu_utilization <= 0.0 || self.scaling.target_cpu_utilization > 100.0 {
            errors.push("scaling.target_cpu_utilization must be between 0 and 100".into());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn detect(project_dir: &Path) -> Result<DeployTarget, String> {
        if project_dir.join("Dockerfile").exists() || project_dir.join("Dockerfile.multi").exists() {
            if project_dir.join("docker-compose.yml").exists() || project_dir.join("docker-compose.yaml").exists() {
                return Ok(DeployTarget::Docker);
            }
            if project_dir.join("k8s").exists() || project_dir.join("kubernetes").exists() || project_dir.join("kustomize").exists() {
                return Ok(DeployTarget::Kubernetes);
            }
            if project_dir.join(".github").exists() || project_dir.join("Jenkinsfile").exists() {
                return Ok(DeployTarget::Docker);
            }
        }
        if project_dir.join("serverless.yml").exists() || project_dir.join("serverless.yaml").exists() {
            return Ok(DeployTarget::AwsLambda);
        }
        if project_dir.join("app.yaml").exists() {
            return Ok(DeployTarget::GcpCloudRun);
        }
        if project_dir.join("function.json").exists() || project_dir.join("host.json").exists() {
            return Ok(DeployTarget::AzureFunctions);
        }
        if project_dir.join("Procfile").exists() {
            return Ok(DeployTarget::Heroku);
        }
        if project_dir.join("netlify.toml").exists() {
            return Ok(DeployTarget::Netlify);
        }
        if project_dir.join("vercel.json").exists() {
            return Ok(DeployTarget::Vercel);
        }
        if project_dir.join("wrangler.toml").exists() {
            return Ok(DeployTarget::CloudflareWorkers);
        }
        Ok(DeployTarget::Docker)
    }

    pub fn generate_default(project_name: &str, target: DeployTarget) -> Self {
        let (region, memory, timeout) = match &target {
            DeployTarget::AwsLambda => ("us-east-1", 128, 30),
            DeployTarget::AwsEcs => ("us-east-1", 512, 120),
            DeployTarget::AwsEks => ("us-east-1", 512, 120),
            DeployTarget::GcpCloudRun => ("us-central1", 256, 300),
            DeployTarget::GcpGke => ("us-central1", 512, 120),
            DeployTarget::GcpFunctions => ("us-central1", 256, 60),
            DeployTarget::AzureFunctions => ("eastus", 128, 30),
            DeployTarget::AzureAci => ("eastus", 512, 120),
            DeployTarget::AzureAks => ("eastus", 512, 120),
            DeployTarget::Heroku => ("us", 512, 30),
            DeployTarget::Netlify => ("us", 128, 30),
            DeployTarget::Vercel => ("iad1", 128, 30),
            DeployTarget::CloudflareWorkers => ("weur", 128, 30),
            DeployTarget::Kubernetes => ("us-east-1", 512, 120),
            DeployTarget::Docker => ("us-east-1", 512, 120),
        };
        Self {
            project_name: project_name.to_string(),
            deploy_target: target,
            region: region.to_string(),
            memory,
            timeout,
            environment: HashMap::from([("NODE_ENV".into(), "production".into())]),
            secrets: Vec::new(),
            vpc_config: None,
            health_check: HealthCheckConfig::default(),
            scaling: ScalingConfig::default(),
            domains: Vec::new(),
            tags: HashMap::new(),
        }
    }
}
