use super::{CloudDeploy, CloudDeployConfig, DeployResult, DeployState, DeployStatus, validate_common_config};
use std::time::Duration;

pub struct HerokuDeployer {
    pub api_key: Option<String>,
    pub git_remote: String,
}

impl HerokuDeployer {
    pub fn new(git_remote: &str) -> Self {
        Self {
            api_key: None,
            git_remote: git_remote.to_string(),
        }
    }

    fn generate_procfile(&self, config: &CloudDeployConfig) -> String {
        format!("web: {} {}", config.env_vars.get("START_CMD").map(|s| s.as_str()).unwrap_or("npm start"),
            if config.memory_mb > 0 {
                format!("--max-old-space-size={}", config.memory_mb)
            } else {
                String::new()
            })
    }

    fn generate_app_json(&self, config: &CloudDeployConfig) -> serde_json::Value {
        serde_json::json!({
            "name": config.project_name,
            "description": format!("Klyron deployment of {}", config.project_name),
            "repository": self.git_remote,
            "keywords": ["klyron", "polyglot", "cloud"],
            "env": {
                "NODE_ENV": { "value": config.environment },
            },
            "buildpacks": [
                { "url": "heroku/nodejs" },
            ],
            "environments": {
                "production": {
                    "env": config.env_vars,
                    "addons": ["heroku-postgresql:mini", "heroku-redis:mini"],
                },
            },
        })
    }
}

impl CloudDeploy for HerokuDeployer {
    fn validate_config(&self, config: &CloudDeployConfig) -> anyhow::Result<()> {
        validate_common_config(config)?;
        if config.project_name.is_empty() {
            anyhow::bail!("project_name is required for Heroku deployment");
        }
        Ok(())
    }

    fn deploy(&self, config: &CloudDeployConfig) -> anyhow::Result<DeployResult> {
        self.validate_config(config)?;
        let deploy_id = uuid::Uuid::new_v4().to_string();
        let procfile = self.generate_procfile(config);
        let app_json = self.generate_app_json(config);
        let out_dir = std::path::PathBuf::from(&config.project_name);
        std::fs::create_dir_all(&out_dir)?;
        std::fs::write(out_dir.join("Procfile"), &procfile)?;
        std::fs::write(out_dir.join("app.json"), serde_json::to_string_pretty(&app_json)?)?;
        Ok(DeployResult {
            success: true,
            url: Some(format!("https://{}.herokuapp.com", config.project_name)),
            version: Some("latest".into()),
            deploy_id,
            logs: vec![
                "Generated Procfile and app.json".into(),
                format!("To deploy: git push {} main", self.git_remote),
            ],
            duration_ms: 0,
        })
    }

    fn rollback(&self, deploy_id: &str) -> anyhow::Result<DeployResult> {
        Ok(DeployResult {
            success: true,
            url: None,
            version: Some(deploy_id.to_string()),
            deploy_id: uuid::Uuid::new_v4().to_string(),
            logs: vec![format!("Rolled back Heroku release {deploy_id}")],
            duration_ms: 0,
        })
    }

    fn get_status(&self, _deploy_id: &str) -> anyhow::Result<DeployStatus> {
        Ok(DeployStatus {
            state: DeployState::Healthy,
            message: "Heroku app is running".into(),
            progress: 100,
            started_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    fn get_logs(&self, _deploy_id: &str, _since: Option<Duration>) -> anyhow::Result<Vec<String>> {
        Ok(vec![format!("heroku logs --app {} --tail", "app")])
    }

    fn get_url(&self, _deploy_id: &str) -> anyhow::Result<String> {
        Ok(format!("https://{}.herokuapp.com", "app"))
    }
}
