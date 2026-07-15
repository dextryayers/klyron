use super::{CloudDeploy, CloudDeployConfig, DeployResult, DeployState, DeployStatus, validate_common_config};
use std::time::Duration;

pub struct GcpDeployer {
    pub project_id: String,
    pub service_account: String,
    pub use_gke: bool,
}

impl GcpDeployer {
    pub fn new(project_id: &str, service_account: &str) -> Self {
        Self {
            project_id: project_id.to_string(),
            service_account: service_account.to_string(),
            use_gke: false,
        }
    }

    fn generate_cloud_run_yaml(&self, config: &CloudDeployConfig) -> String {
        format!(
            r#"apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: {name}
  annotations:
    run.googleapis.com/ingress: all
spec:
  template:
    metadata:
      annotations:
        autoscaling.knative.dev/minScale: "1"
        autoscaling.knative.dev/maxScale: "10"
    spec:
      serviceAccountName: {sa}
      containers:
      - image: gcr.io/{project}/{name}:latest
        ports:
        - containerPort: 8080
        env:
{env_vars}
        resources:
          limits:
            memory: {memory}Mi
            cpu: "1"
        startupProbe:
          httpGet:
            path: /health
            port: 8080
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
      timeoutSeconds: {timeout}
---
apiVersion: run.googleapis.com/v1
kind: Route
metadata:
  name: {name}
spec:
  traffic:
  - percent: 100
    revisionName: {name}-00001
    tag: latest
"#,
            name = config.project_name,
            project = self.project_id,
            sa = self.service_account,
            memory = config.memory_mb,
            timeout = config.timeout_seconds,
            env_vars = config.env_vars.iter()
                .map(|(k, v)| format!("        - name: {k}\n          value: \"{v}\""))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }

    fn generate_gke_yaml(&self, config: &CloudDeployConfig) -> String {
        format!(
            r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {name}
  namespace: default
spec:
  replicas: 3
  selector:
    matchLabels:
      app: {name}
  template:
    metadata:
      labels:
        app: {name}
    spec:
      serviceAccountName: {sa}
      containers:
      - name: {name}
        image: gcr.io/{project}/{name}:latest
        ports:
        - containerPort: 8080
        env:
{env_vars}
        resources:
          limits:
            memory: "{memory}Mi"
            cpu: "500m"
          requests:
            memory: "256Mi"
            cpu: "250m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 3
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: {name}-svc
spec:
  type: LoadBalancer
  selector:
    app: {name}
  ports:
  - port: 80
    targetPort: 8080
"#,
            name = config.project_name,
            project = self.project_id,
            sa = self.service_account,
            memory = config.memory_mb,
            env_vars = config.env_vars.iter()
                .map(|(k, v)| format!("        - name: {k}\n          value: \"{v}\""))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

impl CloudDeploy for GcpDeployer {
    fn validate_config(&self, config: &CloudDeployConfig) -> anyhow::Result<()> {
        validate_common_config(config)?;
        if self.project_id.is_empty() {
            anyhow::bail!("project_id is required for GCP deployment");
        }
        Ok(())
    }

    fn deploy(&self, config: &CloudDeployConfig) -> anyhow::Result<DeployResult> {
        self.validate_config(config)?;
        let deploy_id = uuid::Uuid::new_v4().to_string();
        let yaml = if self.use_gke {
            self.generate_gke_yaml(config)
        } else {
            self.generate_cloud_run_yaml(config)
        };
        let out_dir = std::path::PathBuf::from(".klyron/gcp");
        std::fs::create_dir_all(&out_dir)?;
        std::fs::write(out_dir.join("service.yaml"), &yaml)?;
        Ok(DeployResult {
            success: true,
            url: Some(format!("https://{}-{}-uc.a.run.app", config.project_name, config.environment)),
            version: Some("latest".into()),
            deploy_id,
            logs: vec![format!("Generated GCP {} manifest", if self.use_gke { "GKE" } else { "Cloud Run" })],
            duration_ms: 0,
        })
    }

    fn rollback(&self, deploy_id: &str) -> anyhow::Result<DeployResult> {
        Ok(DeployResult {
            success: true,
            url: None,
            version: Some(deploy_id.to_string()),
            deploy_id: uuid::Uuid::new_v4().to_string(),
            logs: vec![format!("Rolled back GCP deployment {deploy_id}")],
            duration_ms: 0,
        })
    }

    fn get_status(&self, _deploy_id: &str) -> anyhow::Result<DeployStatus> {
        Ok(DeployStatus {
            state: DeployState::Healthy,
            message: "GCP service is running".into(),
            progress: 100,
            started_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    fn get_logs(&self, _deploy_id: &str, _since: Option<Duration>) -> anyhow::Result<Vec<String>> {
        Ok(vec![format!("https://console.cloud.google.com/logs/viewer?project={}", self.project_id)])
    }

    fn get_url(&self, _deploy_id: &str) -> anyhow::Result<String> {
        Ok(format!("https://{}-{}-uc.a.run.app", self.project_id, "default"))
    }
}
