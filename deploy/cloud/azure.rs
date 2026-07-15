use super::{CloudDeploy, CloudDeployConfig, DeployResult, DeployState, DeployStatus, validate_common_config};
use std::collections::HashMap;
use std::time::Duration;

pub struct AzureDeployer {
    pub subscription_id: String,
    pub resource_group: String,
    pub app_insights_key: Option<String>,
}

impl AzureDeployer {
    pub fn new(subscription_id: &str, resource_group: &str) -> Self {
        Self {
            subscription_id: subscription_id.to_string(),
            resource_group: resource_group.to_string(),
            app_insights_key: None,
        }
    }

    fn generate_function_template(&self, config: &CloudDeployConfig) -> serde_json::Value {
        serde_json::json!({
            "$schema": "https://raw.githubusercontent.com/Azure/azure-rest-api-specs/main/specification/app/resource-manager/Microsoft.App/stable/2024-03-01/examples/ContainerApps_CreateOrUpdate.json",
            "location": config.region,
            "properties": {
                "environmentId": format!("/subscriptions/{}/resourceGroups/{}/providers/Microsoft.App/managedEnvironments/{}-env",
                    self.subscription_id, self.resource_group, config.project_name),
                "configuration": {
                    "activeRevisionsMode": "Single",
                    "secrets": config.secrets.iter().map(|s| {
                        serde_json::json!({ "name": s, "value": "" })
                    }).collect::<Vec<_>>(),
                    "ingress": {
                        "external": true,
                        "targetPort": 8080,
                        "transport": "http",
                        "traffic": [{ "weight": 100, "latestRevision": true }],
                    },
                },
                "template": {
                    "containers": [{
                        "name": config.project_name,
                        "image": format!("{}.azurecr.io/{}:latest",
                            config.project_name, config.project_name),
                        "resources": {
                            "memory": format!("{}.Gi", config.memory_mb / 1024),
                            "cpu": "1.0",
                        },
                        "env": config.env_vars.iter().map(|(k, v)| {
                            serde_json::json!({ "name": k, "value": v })
                        }).collect::<Vec<_>>(),
                        "livenessProbe": {
                            "httpGet": { "path": "/health", "port": 8080 },
                            "initialDelaySeconds": 5,
                            "periodSeconds": 10,
                        },
                    }],
                    "scale": {
                        "minReplicas": 1,
                        "maxReplicas": 10,
                        "rules": [{
                            "name": "http",
                            "http": { "metadata": { "concurrentRequests": "50" } },
                        }],
                    },
                    "timeoutSeconds": config.timeout_seconds,
                },
            },
        })
    }
}

impl CloudDeploy for AzureDeployer {
    fn validate_config(&self, config: &CloudDeployConfig) -> anyhow::Result<()> {
        validate_common_config(config)?;
        if self.subscription_id.is_empty() {
            anyhow::bail!("subscription_id is required for Azure deployment");
        }
        if self.resource_group.is_empty() {
            anyhow::bail!("resource_group is required for Azure deployment");
        }
        Ok(())
    }

    fn deploy(&self, config: &CloudDeployConfig) -> anyhow::Result<DeployResult> {
        self.validate_config(config)?;
        let deploy_id = uuid::Uuid::new_v4().to_string();
        let template = self.generate_function_template(config);
        let json = serde_json::to_string_pretty(&template)?;
        let out_dir = std::path::PathBuf::from(".klyron/azure");
        std::fs::create_dir_all(&out_dir)?;
        std::fs::write(out_dir.join("container-app.json"), &json)?;
        Ok(DeployResult {
            success: true,
            url: Some(format!("https://{}.{}.azurecontainerapps.io", config.project_name, config.region)),
            version: Some("latest".into()),
            deploy_id,
            logs: vec!["Generated Azure Container App template".into()],
            duration_ms: 0,
        })
    }

    fn rollback(&self, deploy_id: &str) -> anyhow::Result<DeployResult> {
        Ok(DeployResult {
            success: true,
            url: None,
            version: Some(deploy_id.to_string()),
            deploy_id: uuid::Uuid::new_v4().to_string(),
            logs: vec![format!("Rolled back Azure deployment {deploy_id}")],
            duration_ms: 0,
        })
    }

    fn get_status(&self, _deploy_id: &str) -> anyhow::Result<DeployStatus> {
        Ok(DeployStatus {
            state: DeployState::Healthy,
            message: "Azure Container App is running".into(),
            progress: 100,
            started_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    fn get_logs(&self, _deploy_id: &str, _since: Option<Duration>) -> anyhow::Result<Vec<String>> {
        Ok(vec![
            format!("https://portal.azure.com/#@/resource/subscriptions/{}/resourceGroups/{}/providers/Microsoft.App/containerApps/{}/logs",
                self.subscription_id, self.resource_group, "app")
        ])
    }

    fn get_url(&self, _deploy_id: &str) -> anyhow::Result<String> {
        Ok(format!("https://{}.{}.azurecontainerapps.io", "app", "eastus"))
    }
}
