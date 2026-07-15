use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KustomizeOverlay {
    pub name: String,
    pub bases: Vec<String>,
    pub patches: Vec<KustomizePatch>,
    pub images: Vec<ImageOverride>,
    pub config_map_generator: Vec<ConfigMapGenerator>,
    pub secret_generator: Vec<SecretGenerator>,
    pub name_prefix: String,
    pub common_labels: HashMap<String, String>,
    pub replicas: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KustomizePatch {
    pub path: String,
    pub patch_type: PatchType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatchType {
    StrategicMerge,
    JsonPatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageOverride {
    pub name: String,
    pub new_name: String,
    pub new_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMapGenerator {
    pub name: String,
    pub files: Vec<String>,
    pub literals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretGenerator {
    pub name: String,
    pub files: Vec<String>,
    pub envs: Vec<String>,
    pub literals: Vec<String>,
    pub secret_type: String,
}

impl KustomizeOverlay {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            bases: vec!["../../base".into()],
            patches: Vec::new(),
            images: Vec::new(),
            config_map_generator: Vec::new(),
            secret_generator: Vec::new(),
            name_prefix: String::new(),
            common_labels: HashMap::new(),
            replicas: HashMap::new(),
        }
    }

    fn kustomization_yaml(&self) -> String {
        let mut out = String::from("apiVersion: kustomize.config.k8s.io/v1beta1\nkind: Kustomization\n\n");
        if !self.name_prefix.is_empty() {
            out.push_str(&format!("namePrefix: {}\n\n", self.name_prefix));
        }
        if !self.common_labels.is_empty() {
            out.push_str("commonLabels:\n");
            for (k, v) in &self.common_labels {
                out.push_str(&format!("  {k}: {v}\n"));
            }
            out.push('\n');
        }
        if !self.bases.is_empty() {
            out.push_str("bases:\n");
            for base in &self.bases {
                out.push_str(&format!("  - {base}\n"));
            }
            out.push('\n');
        }
        if !self.replicas.is_empty() {
            out.push_str("replicas:\n");
            for (name, count) in &self.replicas {
                out.push_str(&format!("  - name: {name}\n    count: {count}\n"));
            }
            out.push('\n');
        }
        if !self.images.is_empty() {
            out.push_str("images:\n");
            for img in &self.images {
                out.push_str(&format!("  - name: {}\n    newName: {}\n    newTag: {}\n", img.name, img.new_name, img.new_tag));
            }
            out.push('\n');
        }
        if !self.patches.is_empty() {
            out.push_str("patches:\n");
            for patch in &self.patches {
                let ptype = match patch.patch_type {
                    PatchType::StrategicMerge => "strategic",
                    PatchType::JsonPatch => "json",
                };
                out.push_str(&format!("  - path: {}\n    patchType: {ptype}\n", patch.path));
            }
            out.push('\n');
        }
        if !self.config_map_generator.is_empty() {
            out.push_str("configMapGenerator:\n");
            for cm in &self.config_map_generator {
                out.push_str(&format!("  - name: {}\n", cm.name));
                if !cm.files.is_empty() {
                    out.push_str("    files:\n");
                    for f in &cm.files {
                        out.push_str(&format!("      - {f}\n"));
                    }
                }
                if !cm.literals.is_empty() {
                    out.push_str("    literals:\n");
                    for l in &cm.literals {
                        out.push_str(&format!("      - {l}\n"));
                    }
                }
            }
            out.push('\n');
        }
        if !self.secret_generator.is_empty() {
            out.push_str("secretGenerator:\n");
            for sg in &self.secret_generator {
                out.push_str(&format!("  - name: {}\n", sg.name));
                if !sg.files.is_empty() {
                    out.push_str("    files:\n");
                    for f in &sg.files {
                        out.push_str(&format!("      - {f}\n"));
                    }
                }
                if !sg.envs.is_empty() {
                    out.push_str("    envs:\n");
                    for e in &sg.envs {
                        out.push_str(&format!("      - {e}\n"));
                    }
                }
                if !sg.literals.is_empty() {
                    out.push_str("    literals:\n");
                    for l in &sg.literals {
                        out.push_str(&format!("      - {l}\n"));
                    }
                }
                if !sg.secret_type.is_empty() && sg.secret_type != "Opaque" {
                    out.push_str(&format!("    type: {}\n", sg.secret_type));
                }
            }
            out.push('\n');
        }
        out
    }

    pub fn generate_overlay(&self, output_base: &Path) -> Result<PathBuf, String> {
        let overlay_dir = output_base.join("overlays").join(&self.name);
        fs::create_dir_all(&overlay_dir).map_err(|e| format!("Cannot create overlay directory: {e}"))?;
        fs::write(overlay_dir.join("kustomization.yaml"), &self.kustomization_yaml())
            .map_err(|e| format!("Cannot write kustomization.yaml: {e}"))?;
        Ok(overlay_dir)
    }
}

pub fn generate_dev_overlay(base_dir: &Path) -> Result<KustomizeOverlay, String> {
    let mut overlay = KustomizeOverlay::new("dev");
    overlay.name_prefix = "dev-";
    overlay.common_labels.insert("environment".into(), "dev".into());
    overlay.replicas.insert("klyron".into(), 1);
    overlay.images.push(ImageOverride {
        name: "ghcr.io/klyron/klyron".into(),
        new_name: "ghcr.io/klyron/klyron".into(),
        new_tag: "latest".into(),
    });
    overlay.config_map_generator.push(ConfigMapGenerator {
        name: "klyron-env".into(),
        files: vec![],
        literals: vec!["LOG_LEVEL=debug".into(), "NODE_ENV=development".into()],
    });
    overlay.generate_overlay(base_dir)?;
    Ok(overlay)
}

pub fn generate_staging_overlay(base_dir: &Path) -> Result<KustomizeOverlay, String> {
    let mut overlay = KustomizeOverlay::new("staging");
    overlay.name_prefix = "staging-";
    overlay.common_labels.insert("environment".into(), "staging".into());
    overlay.replicas.insert("klyron".into(), 2);
    overlay.images.push(ImageOverride {
        name: "ghcr.io/klyron/klyron".into(),
        new_name: "ghcr.io/klyron/klyron".into(),
        new_tag: "staging".into(),
    });
    overlay.config_map_generator.push(ConfigMapGenerator {
        name: "klyron-env".into(),
        files: vec![],
        literals: vec!["LOG_LEVEL=debug".into(), "NODE_ENV=staging".into()],
    });
    overlay.patches.push(KustomizePatch {
        path: "resource-limits.yaml".into(),
        patch_type: PatchType::StrategicMerge,
    });
    let patch_content = r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: klyron
spec:
  template:
    spec:
      containers:
        - name: klyron
          resources:
            limits:
              cpu: "500m"
              memory: "512Mi"
            requests:
              cpu: "250m"
              memory: "256Mi"
"#;
    fs::write(overlay_dir_path(base_dir, "staging").join("resource-limits.yaml"), patch_content)
        .map_err(|e| format!("Cannot write resource-limits.yaml: {e}"))?;
    overlay.generate_overlay(base_dir)?;
    Ok(overlay)
}

pub fn generate_prod_overlay(base_dir: &Path) -> Result<KustomizeOverlay, String> {
    let mut overlay = KustomizeOverlay::new("prod");
    overlay.name_prefix = "prod-";
    overlay.common_labels.insert("environment".into(), "production".into());
    overlay.replicas.insert("klyron".into(), 3);
    overlay.images.push(ImageOverride {
        name: "ghcr.io/klyron/klyron".into(),
        new_name: "ghcr.io/klyron/klyron".into(),
        new_tag: "prod".into(),
    });
    overlay.config_map_generator.push(ConfigMapGenerator {
        name: "klyron-env".into(),
        files: vec![],
        literals: vec!["LOG_LEVEL=info".into(), "NODE_ENV=production".into()],
    });
    overlay.patches.push(KustomizePatch {
        path: "resource-limits.yaml".into(),
        patch_type: PatchType::StrategicMerge,
    });
    overlay.patches.push(KustomizePatch {
        path: "hpa.yaml".into(),
        patch_type: PatchType::StrategicMerge,
    });
    overlay.patches.push(KustomizePatch {
        path: "pdb.yaml".into(),
        patch_type: PatchType::StrategicMerge,
    });
    let patch_dir = overlay_dir_path(base_dir, "prod");
    fs::create_dir_all(&patch_dir).map_err(|e| format!("Cannot create patch directory: {e}"))?;

    let resource_patch = r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: klyron
spec:
  template:
    spec:
      containers:
        - name: klyron
          resources:
            limits:
              cpu: "1"
              memory: "1Gi"
            requests:
              cpu: "500m"
              memory: "512Mi"
"#;
    fs::write(patch_dir.join("resource-limits.yaml"), resource_patch)
        .map_err(|e| format!("Cannot write resource-limits.yaml: {e}"))?;

    let hpa_patch = r#"apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: klyron
spec:
  minReplicas: 3
  maxReplicas: 20
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
"#;
    fs::write(patch_dir.join("hpa.yaml"), hpa_patch)
        .map_err(|e| format!("Cannot write hpa.yaml: {e}"))?;

    let pdb_patch = r#"apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: klyron-pdb
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app.kubernetes.io/name: klyron
"#;
    fs::write(patch_dir.join("pdb.yaml"), pdb_patch)
        .map_err(|e| format!("Cannot write pdb.yaml: {e}"))?;

    overlay.generate_overlay(base_dir)?;
    Ok(overlay)
}

fn overlay_dir_path(base_dir: &Path, name: &str) -> PathBuf {
    base_dir.join("overlays").join(name)
}
