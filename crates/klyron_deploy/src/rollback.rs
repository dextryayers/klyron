use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRecord {
    pub id: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub strategy: String,
    pub platform: String,
    pub status: DeploymentStatus,
    pub artifact_hash: String,
    pub config_snapshot: HashMap<String, String>,
    pub health_check_passed: bool,
    pub rollback_to: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Pending,
    InProgress,
    Success,
    Failed,
    RolledBack,
    RollbackInProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackState {
    pub deployments: Vec<DeploymentRecord>,
    pub current_active: Option<String>,
    pub rollback_history: Vec<RollbackAction>,
    pub state_file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackAction {
    pub timestamp: DateTime<Utc>,
    pub from_version: String,
    pub to_version: String,
    pub reason: String,
    pub success: bool,
}

pub struct RollbackManager {
    state: RollbackState,
}

impl RollbackManager {
    pub fn new(dir: &Path) -> Result<Self> {
        let state_file = dir.join(".klyron-deploy-state.json");
        let state = if state_file.exists() {
            let content = std::fs::read_to_string(&state_file)
                .context("Failed to read deployment state")?;
            serde_json::from_str(&content).unwrap_or_else(|_| RollbackState {
                deployments: Vec::new(),
                current_active: None,
                rollback_history: Vec::new(),
                state_file: state_file.clone(),
            })
        } else {
            RollbackState {
                deployments: Vec::new(),
                current_active: None,
                rollback_history: Vec::new(),
                state_file,
            }
        };

        Ok(Self { state })
    }

    pub fn record_deployment(
        &mut self,
        version: &str,
        strategy: &str,
        platform: &str,
        config: &HashMap<String, String>,
    ) -> Result<DeploymentRecord> {
        let timestamp = Utc::now();
        let id = Self::generate_deployment_id(version, &timestamp);

        let record = DeploymentRecord {
            id: id.clone(),
            version: version.to_string(),
            timestamp,
            strategy: strategy.to_string(),
            platform: platform.to_string(),
            status: DeploymentStatus::InProgress,
            artifact_hash: String::new(),
            config_snapshot: config.clone(),
            health_check_passed: false,
            rollback_to: self.state.current_active.clone(),
        };

        self.state.deployments.push(record.clone());
        self.state.current_active = Some(id.clone());
        self.save_state()?;

        Ok(record)
    }

    pub fn mark_success(&mut self, id: &str, artifact_hash: &str) -> Result<()> {
        if let Some(record) = self.state.deployments.iter_mut().find(|r| r.id == id) {
            record.status = DeploymentStatus::Success;
            record.artifact_hash = artifact_hash.to_string();
            record.health_check_passed = true;
            self.save_state()?;
            info!("Deployment {} marked as successful", id);
        }
        Ok(())
    }

    pub fn mark_failed(&mut self, id: &str) -> Result<()> {
        if let Some(record) = self.state.deployments.iter_mut().find(|r| r.id == id) {
            record.status = DeploymentStatus::Failed;
            self.save_state()?;
            warn!("Deployment {} marked as failed", id);
        }
        Ok(())
    }

    pub fn rollback(&mut self, id: &str) -> Result<Option<DeploymentRecord>> {
        let target = self.state.deployments.iter()
            .find(|r| r.id == id)
            .cloned();

        if let Some(ref target_record) = target {
            let rollback_to_id = target_record.rollback_to.clone();

            if let Some(prev_id) = rollback_to_id {
                let prev = self.state.deployments.iter()
                    .find(|r| r.id == prev_id)
                    .cloned();

                if let Some(prev_record) = prev {
                    let action = RollbackAction {
                        timestamp: Utc::now(),
                        from_version: target_record.version.clone(),
                        to_version: prev_record.version.clone(),
                        reason: "Manual rollback".into(),
                        success: true,
                    };

                    self.state.rollback_history.push(action);

                    if let Some(current) = self.state.deployments.iter_mut().find(|r| r.id == id) {
                        current.status = DeploymentStatus::RolledBack;
                    }

                    self.state.current_active = Some(prev_id.clone());
                    self.save_state()?;

                    info!("Rolled back from {} to {}", target_record.version, prev_record.version);
                    return Ok(Some(prev_record));
                }
            }
        }

        Ok(None)
    }

    pub fn auto_rollback_on_failure(&mut self, id: &str) -> Result<Option<DeploymentRecord>> {
        warn!("Auto-rollback triggered for deployment {}", id);
        self.mark_failed(id)?;
        self.rollback(id)
    }

    pub fn get_deployment_history(&self) -> &[DeploymentRecord] {
        &self.state.deployments
    }

    pub fn get_current_deployment(&self) -> Option<&DeploymentRecord> {
        let _current_id = self.state.current_active.as_ref()?;
        self.state.deployments.iter().find(|r| Some(&r.id) == self.state.current_active.as_ref())
    }

    pub fn get_rollback_history(&self) -> &[RollbackAction] {
        &self.state.rollback_history
    }

    pub fn list_versions(&self) -> Vec<String> {
        self.state.deployments.iter()
            .map(|r| r.version.clone())
            .collect()
    }

    pub fn cleanup_old_deployments(&mut self, max_to_keep: usize) -> Result<usize> {
        if self.state.deployments.len() <= max_to_keep {
            return Ok(0);
        }

        let to_remove = self.state.deployments.len() - max_to_keep;
        self.state.deployments.drain(..to_remove);
        self.save_state()?;

        Ok(to_remove)
    }

    fn save_state(&self) -> Result<()> {
        if let Some(parent) = self.state.state_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.state)?;
        std::fs::write(&self.state.state_file, content)
            .context("Failed to write deployment state")?;
        Ok(())
    }

    fn generate_deployment_id(version: &str, timestamp: &DateTime<Utc>) -> String {
        let input = format!("{}-{}", version, timestamp.timestamp_nanos_opt().unwrap_or(0));
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let hash = hex::encode(hasher.finalize());
        format!("deploy-{}", &hash[..12])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join("klyron_test_rollback");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_rollback_manager_new() {
        let dir = temp_dir();
        let manager = RollbackManager::new(&dir).unwrap();
        assert!(manager.get_deployment_history().is_empty());
        assert!(manager.get_current_deployment().is_none());
    }

    #[test]
    fn test_record_deployment() {
        let dir = temp_dir();
        let mut manager = RollbackManager::new(&dir).unwrap();
        let config = HashMap::new();
        let record = manager.record_deployment("1.0.0", "rolling", "docker", &config).unwrap();
        assert_eq!(record.version, "1.0.0");
        assert_eq!(record.status, DeploymentStatus::InProgress);
    }

    #[test]
    fn test_mark_success_and_failed() {
        let dir = temp_dir();
        let mut manager = RollbackManager::new(&dir).unwrap();
        let config = HashMap::new();
        let record = manager.record_deployment("1.0.0", "rolling", "docker", &config).unwrap();

        manager.mark_success(&record.id, "abc123").unwrap();
        let current = manager.get_current_deployment().unwrap();
        assert_eq!(current.status, DeploymentStatus::Success);

        let record2 = manager.record_deployment("2.0.0", "blue-green", "docker", &config).unwrap();
        manager.mark_failed(&record2.id).unwrap();
        let failed = manager.state.deployments.iter()
            .find(|r| r.id == record2.id).unwrap();
        assert_eq!(failed.status, DeploymentStatus::Failed);
    }

    #[test]
    fn test_rollback() {
        let dir = temp_dir();
        let mut manager = RollbackManager::new(&dir).unwrap();
        let config = HashMap::new();

        let v1 = manager.record_deployment("1.0.0", "rolling", "docker", &config).unwrap();
        manager.mark_success(&v1.id, "hash1").unwrap();

        let v2 = manager.record_deployment("2.0.0", "rolling", "docker", &config).unwrap();
        manager.mark_success(&v2.id, "hash2").unwrap();

        let rolled_back = manager.rollback(&v2.id).unwrap();
        assert!(rolled_back.is_some());
        assert_eq!(rolled_back.unwrap().version, "1.0.0");
    }

    #[test]
    fn test_auto_rollback_on_failure() {
        let dir = temp_dir();
        let mut manager = RollbackManager::new(&dir).unwrap();
        let config = HashMap::new();

        let v1 = manager.record_deployment("1.0.0", "rolling", "docker", &config).unwrap();
        manager.mark_success(&v1.id, "hash1").unwrap();

        let v2 = manager.record_deployment("2.0.0", "rolling", "docker", &config).unwrap();
        let rolled_back = manager.auto_rollback_on_failure(&v2.id).unwrap();
        assert!(rolled_back.is_some());
    }

    #[test]
    fn test_list_versions() {
        let dir = temp_dir();
        let mut manager = RollbackManager::new(&dir).unwrap();
        let config = HashMap::new();

        manager.record_deployment("1.0.0", "rolling", "docker", &config).unwrap();
        manager.record_deployment("2.0.0", "blue-green", "docker", &config).unwrap();

        let versions = manager.list_versions();
        assert_eq!(versions.len(), 2);
        assert!(versions.contains(&"1.0.0".to_string()));
        assert!(versions.contains(&"2.0.0".to_string()));
    }

    #[test]
    fn test_cleanup_old_deployments() {
        let dir = temp_dir();
        let mut manager = RollbackManager::new(&dir).unwrap();
        let config = HashMap::new();

        for i in 0..5 {
            let record = manager.record_deployment(
                &format!("{}.0.0", i + 1),
                "rolling",
                "docker",
                &config,
            ).unwrap();
            manager.mark_success(&record.id, "hash").unwrap();
        }

        let removed = manager.cleanup_old_deployments(3).unwrap();
        assert_eq!(removed, 2);
        assert_eq!(manager.state.deployments.len(), 3);
    }

    #[test]
    fn test_deployment_status_transitions() {
        let dir = temp_dir();
        let mut manager = RollbackManager::new(&dir).unwrap();
        let config = HashMap::new();

        let record = manager.record_deployment("1.0.0", "canary", "docker", &config).unwrap();
        assert_eq!(record.status, DeploymentStatus::InProgress);

        manager.mark_success(&record.id, "hash").unwrap();
        let current = manager.get_current_deployment().unwrap();
        assert_eq!(current.status, DeploymentStatus::Success);
    }

    #[test]
    fn test_deployment_record_serialization() {
        let record = DeploymentRecord {
            id: "deploy-test-1".into(),
            version: "1.0.0".into(),
            timestamp: Utc::now(),
            strategy: "blue-green".into(),
            platform: "docker".into(),
            status: DeploymentStatus::Success,
            artifact_hash: "abc123".into(),
            config_snapshot: HashMap::new(),
            health_check_passed: true,
            rollback_to: None,
        };
        let json = serde_json::to_string_pretty(&record).unwrap();
        let deserialized: DeploymentRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, "1.0.0");
        assert_eq!(deserialized.strategy, "blue-green");
    }

    #[test]
    fn test_rollback_action_serialization() {
        let action = RollbackAction {
            timestamp: Utc::now(),
            from_version: "2.0.0".into(),
            to_version: "1.0.0".into(),
            reason: "Health check failed".into(),
            success: true,
        };
        let json = serde_json::to_string_pretty(&action).unwrap();
        let deserialized: RollbackAction = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.from_version, "2.0.0");
        assert!(deserialized.success);
    }
}
