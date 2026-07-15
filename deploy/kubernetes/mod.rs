pub mod helm;
pub mod kustomize;
pub mod manifests;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sDeploymentConfig {
    pub name: String,
    pub namespace: String,
    pub replicas: u32,
    pub image: String,
    pub image_pull_policy: String,
    pub container_port: u16,
    pub service_port: u16,
    pub cpu_request: String,
    pub cpu_limit: String,
    pub memory_request: String,
    pub memory_limit: String,
    pub env_vars: HashMap<String, String>,
    pub secrets: Vec<String>,
    pub config_maps: Vec<String>,
    pub health_check_path: String,
    pub ingress_host: Option<String>,
    pub ingress_tls_secret: Option<String>,
    pub hpa_min_replicas: u32,
    pub hpa_max_replicas: u32,
    pub hpa_target_cpu: u32,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub node_selector: HashMap<String, String>,
    pub tolerations: Vec<Toleration>,
    pub affinity: Option<Affinity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toleration {
    pub key: String,
    pub operator: String,
    pub value: String,
    pub effect: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affinity {
    pub node_affinity: Option<HashMap<String, String>>,
    pub pod_anti_affinity: Option<HashMap<String, String>>,
}

impl Default for K8sDeploymentConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            namespace: "default".into(),
            replicas: 3,
            image: String::new(),
            image_pull_policy: "Always".into(),
            container_port: 3000,
            service_port: 80,
            cpu_request: "256m".into(),
            cpu_limit: "512m".into(),
            memory_request: "256Mi".into(),
            memory_limit: "512Mi".into(),
            env_vars: HashMap::new(),
            secrets: Vec::new(),
            config_maps: Vec::new(),
            health_check_path: "/health".into(),
            ingress_host: None,
            ingress_tls_secret: None,
            hpa_min_replicas: 2,
            hpa_max_replicas: 10,
            hpa_target_cpu: 70,
            labels: HashMap::new(),
            annotations: HashMap::new(),
            node_selector: HashMap::new(),
            tolerations: Vec::new(),
            affinity: None,
        }
    }
}

impl K8sDeploymentConfig {
    pub fn new(name: &str, image: &str) -> Self {
        let mut cfg = Self::default();
        cfg.name = name.to_string();
        cfg.image = image.to_string();
        cfg
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.name.is_empty() {
            anyhow::bail!("deployment name is required");
        }
        if self.image.is_empty() {
            anyhow::bail!("container image is required");
        }
        if self.replicas == 0 {
            anyhow::bail!("replicas must be > 0");
        }
        if self.container_port == 0 {
            anyhow::bail!("container_port must be > 0");
        }
        Ok(())
    }
}
