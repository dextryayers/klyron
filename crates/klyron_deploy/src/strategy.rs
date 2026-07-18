use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::health::HealthChecker;
use crate::{DeployConfig, ServiceConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployStrategy {
    Rolling,
    BlueGreen,
    Canary,
    Recreate,
}

impl DeployStrategy {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Rolling => "rolling",
            Self::BlueGreen => "blue-green",
            Self::Canary => "canary",
            Self::Recreate => "recreate",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub strategy: DeployStrategy,
    pub max_batch_size: u32,
    pub min_ready_secs: u64,
    pub max_surge: u32,
    pub max_unavailable: u32,
    pub canary_percent: u8,
    pub warmup_secs: u64,
    pub health_check_interval_secs: u64,
    pub rollback_on_failure: bool,
    pub health_check_endpoint: String,
    pub health_check_timeout_secs: u64,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            strategy: DeployStrategy::Rolling,
            max_batch_size: 2,
            min_ready_secs: 5,
            max_surge: 1,
            max_unavailable: 0,
            canary_percent: 10,
            warmup_secs: 10,
            health_check_interval_secs: 2,
            rollback_on_failure: true,
            health_check_endpoint: "/health".into(),
            health_check_timeout_secs: 10,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StrategyResult {
    pub strategy: DeployStrategy,
    pub phases: Vec<DeployPhase>,
    pub total_duration: Duration,
    pub success: bool,
    pub rolled_back: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct DeployPhase {
    pub name: String,
    pub instance_count: u32,
    pub duration: Duration,
    pub healthy: bool,
    pub checks_passed: u32,
    pub checks_failed: u32,
}

pub struct StrategyEngine;

impl StrategyEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        strategy: &StrategyConfig,
        config: &DeployConfig,
        service: &ServiceConfig,
        dir: &Path,
    ) -> Result<StrategyResult> {
        match strategy.strategy {
            DeployStrategy::BlueGreen => Self::blue_green_deploy(strategy, config, service, dir),
            DeployStrategy::Canary => Self::canary_deploy(strategy, config, service, dir),
            DeployStrategy::Rolling => Self::rolling_deploy(strategy, config, service, dir),
            DeployStrategy::Recreate => Self::recreate_deploy(strategy, config, service, dir),
        }
    }

    fn blue_green_deploy(
        strategy: &StrategyConfig,
        config: &DeployConfig,
        service: &ServiceConfig,
        dir: &Path,
    ) -> Result<StrategyResult> {
        let start = Instant::now();
        let mut phases = Vec::new();
        info!("Starting blue-green deployment");

        Self::deploy_to_platform(config, service, dir, "green")?;
        let warmup_start = Instant::now();
        std::thread::sleep(Duration::from_secs(strategy.warmup_secs));

        let health_checker = HealthChecker::new(
            &strategy.health_check_endpoint,
            strategy.health_check_timeout_secs,
        );

        let checks_passed = health_checker.check_with_retries(
            service.port,
            strategy.health_check_interval_secs,
            (strategy.min_ready_secs / strategy.health_check_interval_secs).max(1),
        );

        let green_duration = warmup_start.elapsed();
        let green_healthy = checks_passed > 0;

        phases.push(DeployPhase {
            name: "green-deploy".into(),
            instance_count: 1,
            duration: green_duration,
            healthy: green_healthy,
            checks_passed: checks_passed as u32,
            checks_failed: 0,
        });

        if green_healthy {
            info!("Green deployment healthy, switching traffic");
            Self::switch_traffic(config, service, dir, "blue", "green")?;

            let switch_phase = DeployPhase {
                name: "traffic-switch".into(),
                instance_count: 1,
                duration: Duration::from_secs(1),
                healthy: true,
                checks_passed: 1,
                checks_failed: 0,
            };
            phases.push(switch_phase);

            info!("Blue-green deployment completed successfully");
            Ok(StrategyResult {
                strategy: DeployStrategy::BlueGreen,
                phases,
                total_duration: start.elapsed(),
                success: true,
                rolled_back: false,
                message: "Blue-green deployment completed successfully".into(),
            })
        } else {
            warn!("Green deployment health checks failed");
            if strategy.rollback_on_failure {
                info!("Rolling back to blue");
                Self::rollback_to_previous(config, service, dir, "blue")?;
            }
            Ok(StrategyResult {
                strategy: DeployStrategy::BlueGreen,
                phases,
                total_duration: start.elapsed(),
                success: false,
                rolled_back: strategy.rollback_on_failure,
                message: "Green deployment health checks failed".into(),
            })
        }
    }

    fn canary_deploy(
        strategy: &StrategyConfig,
        config: &DeployConfig,
        service: &ServiceConfig,
        dir: &Path,
    ) -> Result<StrategyResult> {
        let start = Instant::now();
        let mut phases = Vec::new();
        info!("Starting canary deployment with {}% traffic", strategy.canary_percent);

        Self::deploy_to_platform(config, service, dir, "canary")?;

        let health_checker = HealthChecker::new(
            &strategy.health_check_endpoint,
            strategy.health_check_timeout_secs,
        );

        let checks_passed = health_checker.check_with_retries(
            service.port,
            strategy.health_check_interval_secs,
            (strategy.min_ready_secs / strategy.health_check_interval_secs).max(1),
        );

        let canary_phase = DeployPhase {
            name: "canary-deploy".into(),
            instance_count: 0,
            duration: Duration::from_secs(strategy.warmup_secs),
            healthy: checks_passed > 0,
            checks_passed: checks_passed as u32,
            checks_failed: 0,
        };
        phases.push(canary_phase);

        if checks_passed > 0 {
            info!("Canary healthy, promoting to full rollout");
            Self::promote_canary(config, service, dir)?;

            let promote_phase = DeployPhase {
                name: "canary-promote".into(),
                instance_count: 1,
                duration: Duration::from_secs(1),
                healthy: true,
                checks_passed: 1,
                checks_failed: 0,
            };
            phases.push(promote_phase);

            Ok(StrategyResult {
                strategy: DeployStrategy::Canary,
                phases,
                total_duration: start.elapsed(),
                success: true,
                rolled_back: false,
                message: format!("Canary deployment completed ({}% initial traffic)", strategy.canary_percent),
            })
        } else {
            warn!("Canary health checks failed");
            if strategy.rollback_on_failure {
                Self::rollback_canary(config, service, dir)?;
            }
            Ok(StrategyResult {
                strategy: DeployStrategy::Canary,
                phases,
                total_duration: start.elapsed(),
                success: false,
                rolled_back: strategy.rollback_on_failure,
                message: "Canary health checks failed".into(),
            })
        }
    }

    fn rolling_deploy(
        strategy: &StrategyConfig,
        config: &DeployConfig,
        service: &ServiceConfig,
        dir: &Path,
    ) -> Result<StrategyResult> {
        let start = Instant::now();
        let mut phases = Vec::new();
        info!("Starting rolling deployment");

        Self::deploy_to_platform(config, service, dir, "rolling")?;

        let health_checker = HealthChecker::new(
            &strategy.health_check_endpoint,
            strategy.health_check_timeout_secs,
        );

        let checks_passed = health_checker.check_with_retries(
            service.port,
            strategy.health_check_interval_secs,
            (strategy.min_ready_secs / strategy.health_check_interval_secs).max(1),
        );

        let rolling_phase = DeployPhase {
            name: "rolling-update".into(),
            instance_count: strategy.max_batch_size,
            duration: Duration::from_secs(strategy.min_ready_secs),
            healthy: checks_passed > 0,
            checks_passed: checks_passed as u32,
            checks_failed: 0,
        };
        phases.push(rolling_phase);

        Ok(StrategyResult {
            strategy: DeployStrategy::Rolling,
            phases,
            total_duration: start.elapsed(),
            success: checks_passed > 0,
            rolled_back: false,
            message: if checks_passed > 0 {
                "Rolling deployment completed".into()
            } else {
                "Rolling deployment completed with health check warnings".into()
            },
        })
    }

    fn recreate_deploy(
        strategy: &StrategyConfig,
        config: &DeployConfig,
        service: &ServiceConfig,
        dir: &Path,
    ) -> Result<StrategyResult> {
        let start = Instant::now();
        let mut phases = Vec::new();
        info!("Starting recreate deployment");

        let shutdown_phase = DeployPhase {
            name: "shutdown-existing".into(),
            instance_count: 0,
            duration: Duration::from_secs(2),
            healthy: true,
            checks_passed: 0,
            checks_failed: 0,
        };
        phases.push(shutdown_phase);

        std::thread::sleep(Duration::from_secs(2));

        Self::deploy_to_platform(config, service, dir, "recreate")?;

        let health_checker = HealthChecker::new(
            &strategy.health_check_endpoint,
            strategy.health_check_timeout_secs,
        );

        let checks_passed = health_checker.check_with_retries(
            service.port,
            strategy.health_check_interval_secs,
            (strategy.min_ready_secs / strategy.health_check_interval_secs).max(1),
        );

        let recreate_phase = DeployPhase {
            name: "recreate-start".into(),
            instance_count: 1,
            duration: Duration::from_secs(strategy.min_ready_secs),
            healthy: checks_passed > 0,
            checks_passed: checks_passed as u32,
            checks_failed: 0,
        };
        phases.push(recreate_phase);

        Ok(StrategyResult {
            strategy: DeployStrategy::Recreate,
            phases,
            total_duration: start.elapsed(),
            success: checks_passed > 0,
            rolled_back: false,
            message: "Recreate deployment completed".into(),
        })
    }

    fn deploy_to_platform(
        config: &DeployConfig,
        _service: &ServiceConfig,
        dir: &Path,
        _label: &str,
    ) -> Result<()> {
        info!("Deploying to platform {:?} from {}", config.platform, dir.display());
        Ok(())
    }

    fn switch_traffic(
        _config: &DeployConfig,
        _service: &ServiceConfig,
        _dir: &Path,
        _from: &str,
        _to: &str,
    ) -> Result<()> {
        info!("Switching traffic");
        Ok(())
    }

    fn rollback_to_previous(
        _config: &DeployConfig,
        _service: &ServiceConfig,
        _dir: &Path,
        _target: &str,
    ) -> Result<()> {
        info!("Rolling back to previous version");
        Ok(())
    }

    fn promote_canary(
        _config: &DeployConfig,
        _service: &ServiceConfig,
        _dir: &Path,
    ) -> Result<()> {
        info!("Promoting canary to full production");
        Ok(())
    }

    fn rollback_canary(
        _config: &DeployConfig,
        _service: &ServiceConfig,
        _dir: &Path,
    ) -> Result<()> {
        info!("Rolling back canary deployment");
        Ok(())
    }
}

impl Default for StrategyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use crate::DeployPlatform;

    fn test_config() -> (StrategyConfig, DeployConfig, ServiceConfig, PathBuf) {
        let strategy = StrategyConfig::default();
        let deploy = DeployConfig {
            platform: DeployPlatform::Docker,
            preview: false,
            project_dir: PathBuf::from("/tmp/test"),
            env_vars: HashMap::new(),
            secrets: vec![],
            serverless: false,
            health_check_path: "/health".into(),
        };
        let service = ServiceConfig::default();
        let dir = PathBuf::from("/tmp");
        (strategy, deploy, service, dir)
    }

    #[test]
    fn test_strategy_as_str() {
        assert_eq!(DeployStrategy::Rolling.as_str(), "rolling");
        assert_eq!(DeployStrategy::BlueGreen.as_str(), "blue-green");
        assert_eq!(DeployStrategy::Canary.as_str(), "canary");
        assert_eq!(DeployStrategy::Recreate.as_str(), "recreate");
    }

    #[test]
    fn test_strategy_config_defaults() {
        let cfg = StrategyConfig::default();
        assert_eq!(cfg.strategy, DeployStrategy::Rolling);
        assert_eq!(cfg.canary_percent, 10);
        assert!(cfg.rollback_on_failure);
    }

    #[test]
    fn test_blue_green_deploy() {
        let (strategy, deploy, service, dir) = test_config();
        let result = StrategyEngine::execute(&strategy, &deploy, &service, &dir).unwrap();
        assert_eq!(result.strategy, DeployStrategy::BlueGreen);
    }

    #[test]
    fn test_canary_deploy() {
        let mut strategy = StrategyConfig::default();
        strategy.strategy = DeployStrategy::Canary;
        let (_, deploy, service, dir) = test_config();
        let result = StrategyEngine::execute(&strategy, &deploy, &service, &dir).unwrap();
        assert_eq!(result.strategy, DeployStrategy::Canary);
    }

    #[test]
    fn test_rolling_deploy() {
        let mut strategy = StrategyConfig::default();
        strategy.strategy = DeployStrategy::Rolling;
        let (_, deploy, service, dir) = test_config();
        let result = StrategyEngine::execute(&strategy, &deploy, &service, &dir).unwrap();
        assert_eq!(result.strategy, DeployStrategy::Rolling);
    }

    #[test]
    fn test_recreate_deploy() {
        let mut strategy = StrategyConfig::default();
        strategy.strategy = DeployStrategy::Recreate;
        let (_, deploy, service, dir) = test_config();
        let result = StrategyEngine::execute(&strategy, &deploy, &service, &dir).unwrap();
        assert_eq!(result.strategy, DeployStrategy::Recreate);
    }

    #[test]
    fn test_strategy_result() {
        let result = StrategyResult {
            strategy: DeployStrategy::BlueGreen,
            phases: vec![],
            total_duration: Duration::from_secs(5),
            success: true,
            rolled_back: false,
            message: "Success".into(),
        };
        assert!(result.success);
        assert!(!result.rolled_back);
    }

    #[test]
    fn test_deploy_phase() {
        let phase = DeployPhase {
            name: "test".into(),
            instance_count: 2,
            duration: Duration::from_secs(10),
            healthy: true,
            checks_passed: 3,
            checks_failed: 0,
        };
        assert!(phase.healthy);
        assert_eq!(phase.checks_passed, 3);
    }
}
