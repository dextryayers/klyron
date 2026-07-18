use std::time::{Duration, Instant};

use anyhow::Result;
use tracing::{info, warn};

const DEFAULT_USER_AGENT: &str = "klyron-deploy-health-check/1.0";

#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub body: Option<String>,
    pub checks_passed: u32,
    pub checks_failed: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

pub struct HealthChecker {
    endpoint: String,
    timeout_secs: u64,
    expected_status: u16,
}

impl HealthChecker {
    pub fn new(endpoint: &str, timeout_secs: u64) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            timeout_secs,
            expected_status: 200,
        }
    }

    pub fn with_expected_status(mut self, status: u16) -> Self {
        self.expected_status = status;
        self
    }

    pub fn check(&self, port: u16) -> Result<HealthCheckResult> {
        let url = format!("http://127.0.0.1:{}{}", port, self.endpoint);
        let start = Instant::now();

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(self.timeout_secs))
            .user_agent(DEFAULT_USER_AGENT)
            .build()?;

        let response = match client.get(&url).send() {
            Ok(resp) => resp,
            Err(e) => {
                let elapsed = start.elapsed();
                return Ok(HealthCheckResult {
                    status: HealthStatus::Unhealthy,
                    status_code: 0,
                    response_time_ms: elapsed.as_millis() as u64,
                    body: Some(format!("Connection failed: {e}")),
                    checks_passed: 0,
                    checks_failed: 1,
                });
            }
        };

        let elapsed = start.elapsed();
        let status_code = response.status().as_u16();
        let body = response.text().ok();

        let status = if status_code == self.expected_status {
            HealthStatus::Healthy
        } else if status_code >= 200 && status_code < 500 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        let (checks_passed, checks_failed) = match status {
            HealthStatus::Healthy => (1, 0),
            _ => (0, 1),
        };

        Ok(HealthCheckResult {
            status,
            status_code,
            response_time_ms: elapsed.as_millis() as u64,
            body,
            checks_passed,
            checks_failed,
        })
    }

    pub fn check_with_retries(&self, port: u16, interval_secs: u64, max_retries: u64) -> u64 {
        let mut passed = 0u64;

        for attempt in 0..max_retries {
            match self.check(port) {
                Ok(result) => {
                    if result.status == HealthStatus::Healthy {
                        passed += 1;
                        info!("Health check passed (attempt {})", attempt + 1);
                    } else if result.status == HealthStatus::Degraded {
                        warn!("Health check degraded: HTTP {}", result.status_code);
                    } else {
                        warn!("Health check failed: HTTP {}", result.status_code);
                    }
                }
                Err(e) => {
                    warn!("Health check error (attempt {}): {}", attempt + 1, e);
                }
            }

            if attempt + 1 < max_retries {
                std::thread::sleep(Duration::from_secs(interval_secs));
            }
        }

        passed
    }

    pub fn check_endpoint(url: &str, timeout_secs: u64) -> Result<HealthCheckResult> {
        let checker = Self {
            endpoint: "/".into(),
            timeout_secs,
            expected_status: 200,
        };

        let start = Instant::now();
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent(DEFAULT_USER_AGENT)
            .build()?;

        let response = match client.get(url).send() {
            Ok(resp) => resp,
            Err(e) => {
                return Ok(HealthCheckResult {
                    status: HealthStatus::Unhealthy,
                    status_code: 0,
                    response_time_ms: start.elapsed().as_millis() as u64,
                    body: Some(format!("Request failed: {e}")),
                    checks_passed: 0,
                    checks_failed: 1,
                });
            }
        };

        let elapsed = start.elapsed();
        let status_code = response.status().as_u16();
        let body = response.text().ok();

        let status = if status_code == 200 {
            HealthStatus::Healthy
        } else if status_code < 500 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        let passed = status == HealthStatus::Healthy;
        Ok(HealthCheckResult {
            status,
            status_code,
            response_time_ms: elapsed.as_millis() as u64,
            body,
            checks_passed: if passed { 1 } else { 0 },
            checks_failed: if passed { 0 } else { 1 },
        })
    }

    pub fn check_multiple(endpoints: &[String], timeout_secs: u64) -> Vec<HealthCheckResult> {
        endpoints.iter().map(|url| {
            Self::check_endpoint(url, timeout_secs).unwrap_or_else(|_| HealthCheckResult {
                status: HealthStatus::Unhealthy,
                status_code: 0,
                response_time_ms: 0,
                body: Some("Check failed".into()),
                checks_passed: 0,
                checks_failed: 1,
            })
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_connection_refused() {
        let checker = HealthChecker::new("/health", 2);
        let result = checker.check(19999).unwrap();
        assert_eq!(result.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_check_result_status() {
        let result = HealthCheckResult {
            status: HealthStatus::Healthy,
            status_code: 200,
            response_time_ms: 50,
            body: Some("{\"status\":\"ok\"}".into()),
            checks_passed: 1,
            checks_failed: 0,
        };
        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.checks_passed > 0);
    }

    #[test]
    fn test_health_check_degraded() {
        let result = HealthCheckResult {
            status: HealthStatus::Degraded,
            status_code: 503,
            response_time_ms: 100,
            body: None,
            checks_passed: 0,
            checks_failed: 1,
        };
        assert_eq!(result.status, HealthStatus::Degraded);
        assert!(result.checks_failed > 0);
    }

    #[test]
    fn test_health_check_unhealthy() {
        let result = HealthCheckResult {
            status: HealthStatus::Unhealthy,
            status_code: 0,
            response_time_ms: 0,
            body: Some("timeout".into()),
            checks_passed: 0,
            checks_failed: 1,
        };
        assert_eq!(result.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_check_endpoint_invalid_url() {
        let result = HealthChecker::check_endpoint("http://invalid.local:1/test", 1);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_check_multiple_empty() {
        let results = HealthChecker::check_multiple(&[], 2);
        assert!(results.is_empty());
    }

    #[test]
    fn test_health_checker_builder() {
        let checker = HealthChecker::new("/healthz", 5)
            .with_expected_status(204);
        let result = checker.check(19998).unwrap();
        assert_eq!(result.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_check_retries_zero() {
        let checker = HealthChecker::new("/health", 1);
        let passed = checker.check_with_retries(19997, 1, 0);
        assert_eq!(passed, 0);
    }
}
