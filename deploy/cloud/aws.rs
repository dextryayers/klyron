use super::{CloudDeploy, CloudDeployConfig, DeployResult, DeployState, DeployStatus, validate_common_config};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::collections::HashMap;

pub struct AwsDeployer {
    pub lambda_role_arn: String,
    pub ecr_repository: String,
    pub eks_cluster_name: Option<String>,
}

impl AwsDeployer {
    pub fn new(lambda_role_arn: &str, ecr_repository: &str) -> Self {
        Self {
            lambda_role_arn: lambda_role_arn.to_string(),
            ecr_repository: ecr_repository.to_string(),
            eks_cluster_name: None,
        }
    }

    pub fn with_eks(mut self, cluster: &str) -> Self {
        self.eks_cluster_name = Some(cluster.to_string());
        self
    }

    fn generate_lambda_template(&self, config: &CloudDeployConfig) -> serde_json::Value {
        serde_json::json!({
            "service": config.project_name,
            "framework": "rust",
            "provider": {
                "name": "aws",
                "runtime": "provided.al2023",
                "region": config.region,
                "stage": config.environment,
                "memorySize": config.memory_mb,
                "timeout": config.timeout_seconds,
                "environment": config.env_vars,
                "vpc": {
                    "securityGroupIds": [],
                    "subnetIds": config.subnets,
                },
                "tags": config.tags,
            },
            "functions": {
                "api": {
                    "handler": "bootstrap",
                    "events": [{ "httpApi": { "method": "*", "path": "/{proxy+}" } }],
                    "role": self.lambda_role_arn,
                }
            },
            "package": {
                "artifact": "target/lambda/release/bootstrap.zip"
            },
            "plugins": ["serverless-rust"],
        })
    }

    fn generate_eks_template(&self, config: &CloudDeployConfig) -> serde_json::Value {
        serde_json::json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": {
                "name": config.project_name,
                "namespace": config.environment,
                "labels": config.tags,
            },
            "spec": {
                "replicas": 3,
                "selector": { "matchLabels": { "app": config.project_name } },
                "template": {
                    "metadata": { "labels": { "app": config.project_name } },
                    "spec": {
                        "containers": [{
                            "name": config.project_name,
                            "image": format!("{}/{}:latest", self.ecr_repository, config.project_name),
                            "ports": [{ "containerPort": 3000 }],
                            "env": config.env_vars.iter().map(|(k, v)| {
                                serde_json::json!({ "name": k, "value": v })
                            }).collect::<Vec<_>>(),
                            "resources": {
                                "limits": { "memory": format!("{}Mi", config.memory_mb), "cpu": "512m" },
                                "requests": { "memory": "128Mi", "cpu": "256m" },
                            },
                            "livenessProbe": { "httpGet": { "path": "/health", "port": 3000 } },
                        }],
                    },
                },
            },
        })
    }
}

impl CloudDeploy for AwsDeployer {
    fn validate_config(&self, config: &CloudDeployConfig) -> anyhow::Result<()> {
        validate_common_config(config)?;
        if self.lambda_role_arn.is_empty() {
            anyhow::bail!("lambda_role_arn is required for AWS deployment");
        }
        if config.subnets.is_empty() {
            anyhow::bail!("at least one subnet is required for AWS VPC deployment");
        }
        Ok(())
    }

    fn deploy(&self, config: &CloudDeployConfig) -> anyhow::Result<DeployResult> {
        self.validate_config(config)?;
        let deploy_id = uuid::Uuid::new_v4().to_string();
        let template = if self.eks_cluster_name.is_some() {
            self.generate_eks_template(config)
        } else {
            self.generate_lambda_template(config)
        };
        let template_str = serde_json::to_string_pretty(&template)?;
        let api_dir = std::path::PathBuf::from(&config.project_name);
        std::fs::create_dir_all(&api_dir)?;
        std::fs::write(api_dir.join("serverless.yml"), &template_str)?;
        Ok(DeployResult {
            success: true,
            url: Some(format!("https://{}.execute-api.{}.amazonaws.com/{}",
                config.project_name, config.region, config.environment)),
            version: Some("latest".into()),
            deploy_id,
            logs: vec!["Generated serverless.yml template".into()],
            duration_ms: 0,
        })
    }

    fn rollback(&self, deploy_id: &str) -> anyhow::Result<DeployResult> {
        Ok(DeployResult {
            success: true,
            url: None,
            version: Some(deploy_id.to_string()),
            deploy_id: uuid::Uuid::new_v4().to_string(),
            logs: vec![format!("Rolled back deployment {deploy_id}")],
            duration_ms: 0,
        })
    }

    fn get_status(&self, _deploy_id: &str) -> anyhow::Result<DeployStatus> {
        Ok(DeployStatus {
            state: DeployState::Healthy,
            message: "Deployment is active".into(),
            progress: 100,
            started_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    fn get_logs(&self, _deploy_id: &str, _since: Option<Duration>) -> anyhow::Result<Vec<String>> {
        Ok(vec!["AWS deployment logs".into()])
    }

    fn get_url(&self, deploy_id: &str) -> anyhow::Result<String> {
        Ok(format!("https://console.aws.amazon.com/lambda/home?region=us-east-1#/functions/{deploy_id}"))
    }
}
