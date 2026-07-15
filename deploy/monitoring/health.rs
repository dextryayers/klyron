use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub check_name: String,
    pub status: HealthStatus,
    pub message: String,
    pub latency_ms: u64,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

pub trait HealthCheck: Send + Sync {
    fn check_name(&self) -> &str;
    fn check(&self) -> HealthCheckResult;
    fn severity(&self) -> Severity;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub checks: Vec<HealthCheckResult>,
    pub timestamp: DateTime<Utc>,
    pub overall_latency_ms: u64,
}

impl HealthReport {
    pub fn overall_status(checks: &[HealthCheckResult]) -> HealthStatus {
        let has_critical = checks.iter().any(|c| matches!(c.severity, Severity::Critical) && matches!(c.status, HealthStatus::Unhealthy));
        let has_warning = checks.iter().any(|c| matches!(c.status, HealthStatus::Degraded));
        if has_critical {
            HealthStatus::Unhealthy
        } else if has_warning {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    pub fn from_checks(checks: Vec<HealthCheckResult>, start: Instant) -> Self {
        Self {
            status: Self::overall_status(&checks),
            checks,
            timestamp: Utc::now(),
            overall_latency_ms: start.elapsed().as_millis() as u64,
        }
    }
}

pub struct SystemHealth;

impl SystemHealth {
    pub fn disk_usage(path: &str) -> HealthCheckResult {
        let start = Instant::now();
        let (status, message) = match || -> Result<(u64, u64), String> {
            let usage = fs2::statvfs(path).map_err(|e| e.to_string())?;
            let total = usage.total_space();
            let available = usage.available_space();
            let used_pct = if total > 0 { ((total - available) as f64 / total as f64) * 100.0 } else { 0.0 };
            Ok((total, available))
        }() {
            Ok((total, _avail)) => {
                let used_pct = 0.0;
                if used_pct > 90.0 {
                    (HealthStatus::Unhealthy, format!("Disk usage critical: {used_pct:.1}% (total: {} GB)", total / 1_000_000_000))
                } else if used_pct > 80.0 {
                    (HealthStatus::Degraded, format!("Disk usage warning: {used_pct:.1}% (total: {} GB)", total / 1_000_000_000))
                } else {
                    (HealthStatus::Healthy, format!("Disk usage: {used_pct:.1}% (total: {} GB)", total / 1_000_000_000))
                }
            }
            Err(e) => (HealthStatus::Unhealthy, format!("Disk check failed: {e}")),
        };
        HealthCheckResult {
            check_name: "system.disk".into(),
            status,
            message,
            latency_ms: start.elapsed().as_millis() as u64,
            severity: Severity::Warning,
        }
    }

    pub fn memory_usage() -> HealthCheckResult {
        let start = Instant::now();
        let (status, message) = match sys_info::mem_info() {
            Ok(info) => {
                let total_mb = info.total / 1024;
                let free_mb = info.free / 1024;
                let used_pct = if total_mb > 0 { ((total_mb - free_mb) as f64 / total_mb as f64) * 100.0 } else { 0.0 };
                if used_pct > 95.0 {
                    (HealthStatus::Unhealthy, format!("Memory critical: {used_pct:.1}% used ({free_mb} MB free of {total_mb} MB)"))
                } else if used_pct > 85.0 {
                    (HealthStatus::Degraded, format!("Memory warning: {used_pct:.1}% used ({free_mb} MB free of {total_mb} MB)"))
                } else {
                    (HealthStatus::Healthy, format!("Memory: {used_pct:.1}% used ({free_mb} MB free of {total_mb} MB)"))
                }
            }
            Err(e) => (HealthStatus::Unhealthy, format!("Memory check failed: {e}")),
        };
        HealthCheckResult {
            check_name: "system.memory".into(),
            status,
            message,
            latency_ms: start.elapsed().as_millis() as u64,
            severity: Severity::Warning,
        }
    }

    pub fn cpu_usage() -> HealthCheckResult {
        let start = Instant::now();
        let (status, message) = match sys_info::loadavg() {
            Ok(load) => {
                let cpu_count = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
                let load_pct = (load.one / cpu_count as f64) * 100.0;
                if load_pct > 90.0 {
                    (HealthStatus::Unhealthy, format!("CPU critical: load {:.2} ({} cores)", load.one, cpu_count))
                } else if load_pct > 75.0 {
                    (HealthStatus::Degraded, format!("CPU warning: load {:.2} ({} cores)", load.one, cpu_count))
                } else {
                    (HealthStatus::Healthy, format!("CPU: load {:.2} ({} cores)", load.one, cpu_count))
                }
            }
            Err(e) => (HealthStatus::Unhealthy, format!("CPU check failed: {e}")),
        };
        HealthCheckResult {
            check_name: "system.cpu".into(),
            status,
            message,
            latency_ms: start.elapsed().as_millis() as u64,
            severity: Severity::Warning,
        }
    }

    pub fn uptime() -> HealthCheckResult {
        let start = Instant::now();
        let uptime_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let days = uptime_secs / 86400;
        HealthCheckResult {
            check_name: "system.uptime".into(),
            status: HealthStatus::Healthy,
            message: format!("System uptime: {days} days"),
            latency_ms: start.elapsed().as_millis() as u64,
            severity: Severity::Info,
        }
    }

    pub fn all() -> Vec<HealthCheckResult> {
        vec![
            Self::disk_usage("/"),
            Self::memory_usage(),
            Self::cpu_usage(),
            Self::uptime(),
        ]
    }
}

pub struct ServiceHealth;

impl ServiceHealth {
    pub fn http_endpoint(url: &str, timeout: Duration) -> HealthCheckResult {
        let start = Instant::now();
        let (status, message) = match reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()
            .and_then(|client| client.get(url).send())
        {
            Ok(resp) => {
                let code = resp.status().as_u16();
                if (200..400).contains(&code) {
                    (HealthStatus::Healthy, format!("HTTP {url} returned {code}"))
                } else if (500..600).contains(&code) {
                    (HealthStatus::Unhealthy, format!("HTTP {url} returned {code}"))
                } else {
                    (HealthStatus::Degraded, format!("HTTP {url} returned {code}"))
                }
            }
            Err(e) => (HealthStatus::Unhealthy, format!("HTTP {url} failed: {e}")),
        };
        HealthCheckResult {
            check_name: format!("service.http.{url}"),
            status,
            message,
            latency_ms: start.elapsed().as_millis() as u64,
            severity: Severity::Critical,
        }
    }

    pub fn tcp_port(host: &str, port: u16, timeout: Duration) -> HealthCheckResult {
        let start = Instant::now();
        let addr = format!("{host}:{port}");
        let (status, message) = match std::net::TcpStream::connect_timeout(&addr.parse().unwrap(), timeout) {
            Ok(_) => (HealthStatus::Healthy, format!("TCP {addr} reachable")),
            Err(e) => (HealthStatus::Unhealthy, format!("TCP {addr} unreachable: {e}")),
        };
        HealthCheckResult {
            check_name: format!("service.tcp.{addr}"),
            status,
            message,
            latency_ms: start.elapsed().as_millis() as u64,
            severity: Severity::Critical,
        }
    }

    pub fn process(process_name: &str) -> HealthCheckResult {
        let start = Instant::now();
        let (status, message) = match std::process::Command::new("pgrep")
            .arg("-x")
            .arg(process_name)
            .output()
        {
            Ok(output) if output.status.success() => {
                (HealthStatus::Healthy, format!("Process '{process_name}' is running"))
            }
            Ok(_) => (HealthStatus::Unhealthy, format!("Process '{process_name}' is not running")),
            Err(e) => (HealthStatus::Unhealthy, format!("Process check failed: {e}")),
        };
        HealthCheckResult {
            check_name: format!("service.process.{process_name}"),
            status,
            message,
            latency_ms: start.elapsed().as_millis() as u64,
            severity: Severity::Critical,
        }
    }
}

pub struct DatabaseHealth;

impl DatabaseHealth {
    pub fn connection_pool(uri: &str, max_connections: u32, timeout: Duration) -> HealthCheckResult {
        let start = Instant::now();
        let (status, message) = match std::net::TcpStream::connect_timeout(
            &uri.parse().unwrap_or_else(|_| "127.0.0.1:5432".parse().unwrap()),
            timeout,
        ) {
            Ok(_) => (HealthStatus::Healthy, format!("Database pool reachable ({max_connections} max connections)")),
            Err(e) => (HealthStatus::Unhealthy, format!("Database pool unreachable: {e}")),
        };
        HealthCheckResult {
            check_name: "database.connection_pool".into(),
            status,
            message,
            latency_ms: start.elapsed().as_millis() as u64,
            severity: Severity::Critical,
        }
    }

    pub fn query_latency(uri: &str, query: &str, threshold_ms: u64) -> HealthCheckResult {
        let start = Instant::now();
        let (status, message) = match ureq::post(uri).send_json(serde_json::json!({"query": query})) {
            Ok(_) => {
                let elapsed = start.elapsed().as_millis() as u64;
                if elapsed > threshold_ms {
                    (HealthStatus::Degraded, format!("Query latency high: {elapsed}ms (threshold: {threshold_ms}ms)"))
                } else {
                    (HealthStatus::Healthy, format!("Query latency: {elapsed}ms"))
                }
            }
            Err(e) => (HealthStatus::Unhealthy, format!("Query failed: {e}")),
        };
        HealthCheckResult {
            check_name: "database.query_latency".into(),
            status,
            message,
            latency_ms: start.elapsed().as_millis() as u64,
            severity: Severity::Warning,
        }
    }

    pub fn replication_lag(uri: &str) -> HealthCheckResult {
        let start = Instant::now();
        let (status, message) = match ureq::get(uri).call() {
            Ok(resp) => {
                let lag = resp.header("X-Replication-Lag").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
                if lag > 300 {
                    (HealthStatus::Unhealthy, format!("Replication lag critical: {lag}s"))
                } else if lag > 60 {
                    (HealthStatus::Degraded, format!("Replication lag warning: {lag}s"))
                } else {
                    (HealthStatus::Healthy, format!("Replication lag: {lag}s"))
                }
            }
            Err(e) => (HealthStatus::Unhealthy, format!("Replication check failed: {e}")),
        };
        HealthCheckResult {
            check_name: "database.replication_lag".into(),
            status,
            message,
            latency_ms: start.elapsed().as_millis() as u64,
            severity: Severity::Critical,
        }
    }
}

pub struct CombinedHealthCheck {
    checks: Vec<Box<dyn HealthCheck>>,
}

impl CombinedHealthCheck {
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    pub fn add<C: HealthCheck + 'static>(&mut self, check: C) {
        self.checks.push(Box::new(check));
    }

    pub fn add_boxed(&mut self, check: Box<dyn HealthCheck>) {
        self.checks.push(check);
    }

    pub fn run_all(&self) -> HealthReport {
        let start = Instant::now();
        let results: Vec<HealthCheckResult> = self.checks.iter().map(|c| c.check()).collect();
        HealthReport::from_checks(results, start)
    }
}

impl Default for CombinedHealthCheck {
    fn default() -> Self {
        Self::new()
    }
}
