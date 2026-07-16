use crate::manifest::{MemoryOp, SandboxConfig, SandboxTestReport};
use anyhow::Result;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

#[derive(Clone)]
pub struct SandboxLimits {
    pub max_memory_bytes: u64,
    pub max_fuel: u64,
    pub max_cpu_ms: u64,
    pub allowed_domains: Option<Vec<String>>,
    pub allowed_paths: Option<Vec<String>>,
    pub allowed_env: Option<Vec<String>>,
}

impl Default for SandboxLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024,
            max_fuel: 1_000_000,
            max_cpu_ms: 5000,
            allowed_domains: None,
            allowed_paths: None,
            allowed_env: None,
        }
    }
}

impl From<SandboxConfig> for SandboxLimits {
    fn from(config: SandboxConfig) -> Self {
        Self {
            max_memory_bytes: config.max_memory_bytes.unwrap_or(64 * 1024 * 1024),
            max_fuel: config.max_fuel.unwrap_or(1_000_000),
            max_cpu_ms: config.max_cpu_ms.unwrap_or(5000),
            allowed_domains: config.allowed_domains,
            allowed_paths: config.allowed_paths,
            allowed_env: config.allowed_env,
        }
    }
}

pub struct Sandbox {
    pub limits: SandboxLimits,
    fuel_consumed: Arc<AtomicU64>,
    start_time: Option<Instant>,
}

impl Sandbox {
    pub fn new(limits: SandboxLimits) -> Self {
        Self {
            limits,
            fuel_consumed: Arc::new(AtomicU64::new(0)),
            start_time: None,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(SandboxLimits::default())
    }

    pub fn from_config(config: SandboxConfig) -> Self {
        Self::new(SandboxLimits::from(config))
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn is_timed_out(&self) -> bool {
        if let Some(start) = self.start_time {
            start.elapsed() > Duration::from_millis(self.limits.max_cpu_ms)
        } else {
            false
        }
    }

    pub fn remaining_time(&self) -> Duration {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed();
            let max = Duration::from_millis(self.limits.max_cpu_ms);
            if elapsed >= max {
                Duration::ZERO
            } else {
                max - elapsed
            }
        } else {
            Duration::from_millis(self.limits.max_cpu_ms)
        }
    }

    pub fn consume_fuel(&self, amount: u64) -> bool {
        let current = self.fuel_consumed.fetch_add(amount, Ordering::SeqCst);
        (current + amount) <= self.limits.max_fuel
    }

    pub fn fuel_remaining(&self) -> u64 {
        self.limits
            .max_fuel
            .saturating_sub(self.fuel_consumed.load(Ordering::SeqCst))
    }

    pub fn check_memory(&self, desired: u64) -> bool {
        desired <= self.limits.max_memory_bytes
    }

    pub fn check_domain_allowed(&self, domain: &str) -> bool {
        match &self.limits.allowed_domains {
            Some(domains) => domains.iter().any(|d| domain.ends_with(d)),
            None => true,
        }
    }

    pub fn check_path_allowed(&self, path: &str) -> bool {
        match &self.limits.allowed_paths {
            Some(paths) => paths.iter().any(|p| path.starts_with(p)),
            None => true,
        }
    }

    pub fn check_env_allowed(&self, var: &str) -> bool {
        match &self.limits.allowed_env {
            Some(vars) => vars.iter().any(|v| var == v),
            None => true,
        }
    }

    pub fn verify_isolation(&self) -> Vec<String> {
        let mut violations = Vec::new();
        if let Some(_start) = self.start_time {
            if self.is_timed_out() {
                violations.push("Sandbox timed out".to_string());
            }
        }
        violations
    }
}

pub struct SandboxTestHarness {
    pub plugin_name: String,
    pub wasm_bytes: Vec<u8>,
    pub config: SandboxConfig,
    pub recorded_calls: Vec<String>,
    pub memory_ops: Vec<MemoryOp>,
    pub timeout: Duration,
}

impl SandboxTestHarness {
    pub fn new(plugin_name: &str, wasm_bytes: Vec<u8>, config: SandboxConfig) -> Self {
        Self {
            plugin_name: plugin_name.to_string(),
            wasm_bytes,
            config,
            recorded_calls: Vec::new(),
            memory_ops: Vec::new(),
            timeout: Duration::from_secs(5),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn record_call(&mut self, call: &str) {
        self.recorded_calls.push(call.to_string());
    }

    pub fn record_memory_op(&mut self, op_type: &str, offset: u64, size: u64) {
        self.memory_ops.push(MemoryOp {
            op_type: op_type.to_string(),
            offset,
            size,
        });
    }

    pub fn run_test<F>(&mut self, hook_name: &str, test_fn: F) -> SandboxTestReport
    where
        F: FnOnce(&[u8]) -> Result<Vec<u8>>,
    {
        let start = Instant::now();
        let mut report = SandboxTestReport {
            plugin_name: self.plugin_name.clone(),
            hook_calls: Vec::new(),
            memory_ops: self.memory_ops.clone(),
            total_fuel_consumed: 0,
            execution_time_ms: 0,
            timed_out: false,
            passed: false,
            error: None,
        };

        let result = match test_fn(&self.wasm_bytes) {
            Ok(data) => {
                report.passed = true;
                report.hook_calls.push(hook_name.to_string());
                Some(data)
            }
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("timed out") || msg.contains("timeout") {
                    report.timed_out = true;
                }
                report.error = Some(msg);
                None
            }
        };

        report.execution_time_ms = start.elapsed().as_millis() as u64;
        report.total_fuel_consumed = 0;

        if report.execution_time_ms > self.timeout.as_millis() as u64 {
            report.timed_out = true;
            report.passed = false;
            report.error = Some("Sandbox timeout exceeded".to_string());
        }

        if let Some(data) = result {
            info!(
                "Sandbox test for {} hook {} returned {} bytes",
                self.plugin_name,
                hook_name,
                data.len()
            );
        }

        report
    }

    pub fn assert_hook_invoked(&self, report: &SandboxTestReport, hook: &str) -> bool {
        report.hook_calls.contains(&hook.to_string())
    }

    pub fn assert_no_errors(&self, report: &SandboxTestReport) -> bool {
        report.error.is_none()
    }

    pub fn assert_within_timeout(&self, report: &SandboxTestReport) -> bool {
        !report.timed_out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_limits_default() {
        let limits = SandboxLimits::default();
        assert_eq!(limits.max_memory_bytes, 64 * 1024 * 1024);
        assert_eq!(limits.max_fuel, 1_000_000);
        assert_eq!(limits.max_cpu_ms, 5000);
        assert!(limits.allowed_domains.is_none());
        assert!(limits.allowed_paths.is_none());
        assert!(limits.allowed_env.is_none());
    }

    #[test]
    fn test_sandbox_limits_custom() {
        let limits = SandboxLimits {
            max_memory_bytes: 128 * 1024 * 1024,
            max_fuel: 500_000,
            max_cpu_ms: 10_000,
            allowed_domains: Some(vec!["example.com".into()]),
            allowed_paths: Some(vec!["/tmp".into()]),
            allowed_env: Some(vec!["HOME".into()]),
        };
        assert_eq!(limits.max_memory_bytes, 128 * 1024 * 1024);
        assert_eq!(limits.max_fuel, 500_000);
        assert_eq!(limits.max_cpu_ms, 10_000);
    }

    #[test]
    fn test_sandbox_new_with_defaults() {
        let sandbox = Sandbox::with_defaults();
        assert_eq!(sandbox.limits.max_memory_bytes, 64 * 1024 * 1024);
        assert_eq!(sandbox.limits.max_fuel, 1_000_000);
        assert_eq!(sandbox.limits.max_cpu_ms, 5000);
    }

    #[test]
    fn test_sandbox_from_config() {
        let config = SandboxConfig {
            max_memory_bytes: Some(32 * 1024 * 1024),
            max_fuel: Some(200_000),
            max_cpu_ms: Some(2000),
            allowed_domains: Some(vec!["test.dev".into()]),
            allowed_paths: None,
            allowed_env: None,
        };
        let sandbox = Sandbox::from_config(config);
        assert_eq!(sandbox.limits.max_memory_bytes, 32 * 1024 * 1024);
        assert_eq!(sandbox.limits.max_fuel, 200_000);
        assert_eq!(sandbox.limits.max_cpu_ms, 2000);
        assert_eq!(sandbox.limits.allowed_domains, Some(vec!["test.dev".into()]));
        assert!(sandbox.limits.allowed_paths.is_none());
    }

    #[test]
    fn test_sandbox_from_config_uses_defaults_when_none() {
        let config = SandboxConfig {
            max_memory_bytes: None,
            max_fuel: None,
            max_cpu_ms: None,
            allowed_domains: None,
            allowed_paths: None,
            allowed_env: None,
        };
        let sandbox = Sandbox::from_config(config);
        assert_eq!(sandbox.limits.max_memory_bytes, 64 * 1024 * 1024);
        assert_eq!(sandbox.limits.max_fuel, 1_000_000);
        assert_eq!(sandbox.limits.max_cpu_ms, 5000);
    }

    #[test]
    fn test_sandbox_start_and_timeout() {
        let mut sandbox = Sandbox::with_defaults();
        assert!(!sandbox.is_timed_out());
        sandbox.start();
        // Should not be timed out immediately
        assert!(!sandbox.is_timed_out());
    }

    #[test]
    fn test_sandbox_remaining_time_before_start() {
        let sandbox = Sandbox::with_defaults();
        let remaining = sandbox.remaining_time();
        assert_eq!(remaining, Duration::from_millis(5000));
    }

    #[test]
    fn test_sandbox_remaining_time_after_start() {
        let mut sandbox = Sandbox::with_defaults();
        sandbox.start();
        let remaining = sandbox.remaining_time();
        assert!(remaining <= Duration::from_millis(5000));
        assert!(remaining > Duration::ZERO);
    }

    #[test]
    fn test_consume_fuel_within_limit() {
        let sandbox = Sandbox::with_defaults();
        assert!(sandbox.consume_fuel(500_000));
        assert_eq!(sandbox.fuel_remaining(), 500_000);
    }

    #[test]
    fn test_consume_fuel_exactly_at_limit() {
        let sandbox = Sandbox::with_defaults();
        assert!(sandbox.consume_fuel(1_000_000));
        assert_eq!(sandbox.fuel_remaining(), 0);
    }

    #[test]
    fn test_consume_fuel_exceeds_limit() {
        let sandbox = Sandbox::with_defaults();
        assert!(!sandbox.consume_fuel(1_000_001));
    }

    #[test]
    fn test_consume_fuel_multiple_calls() {
        let sandbox = Sandbox::with_defaults();
        assert!(sandbox.consume_fuel(300_000));
        assert!(sandbox.consume_fuel(300_000));
        assert!(sandbox.consume_fuel(300_000));
        assert_eq!(sandbox.fuel_remaining(), 100_000);
        assert!(!sandbox.consume_fuel(200_000));
        assert_eq!(sandbox.fuel_remaining(), 0);
    }

    #[test]
    fn test_check_memory_within_limit() {
        let sandbox = Sandbox::with_defaults();
        assert!(sandbox.check_memory(64 * 1024 * 1024));
    }

    #[test]
    fn test_check_memory_exceeds_limit() {
        let sandbox = Sandbox::with_defaults();
        assert!(!sandbox.check_memory(64 * 1024 * 1024 + 1));
    }

    #[test]
    fn test_check_memory_zero() {
        let sandbox = Sandbox::with_defaults();
        assert!(sandbox.check_memory(0));
    }

    #[test]
    fn test_domain_allowed_no_restrictions() {
        let sandbox = Sandbox::with_defaults();
        assert!(sandbox.check_domain_allowed("evil.com"));
        assert!(sandbox.check_domain_allowed(""));
    }

    #[test]
    fn test_domain_allowed_with_restrictions() {
        let limits = SandboxLimits {
            allowed_domains: Some(vec!["example.com".into(), "trusted.org".into()]),
            ..SandboxLimits::default()
        };
        let sandbox = Sandbox::new(limits);
        assert!(sandbox.check_domain_allowed("sub.example.com"));
        assert!(sandbox.check_domain_allowed("trusted.org"));
        assert!(!sandbox.check_domain_allowed("evil.com"));
    }

    #[test]
    fn test_domain_allowed_exact_match() {
        let limits = SandboxLimits {
            allowed_domains: Some(vec!["example.com".into()]),
            ..SandboxLimits::default()
        };
        let sandbox = Sandbox::new(limits);
        assert!(sandbox.check_domain_allowed("example.com"));
    }

    #[test]
    fn test_path_allowed_no_restrictions() {
        let sandbox = Sandbox::with_defaults();
        assert!(sandbox.check_path_allowed("/etc/passwd"));
    }

    #[test]
    fn test_path_allowed_with_restrictions() {
        let limits = SandboxLimits {
            allowed_paths: Some(vec!["/tmp".into(), "/var/data".into()]),
            ..SandboxLimits::default()
        };
        let sandbox = Sandbox::new(limits);
        assert!(sandbox.check_path_allowed("/tmp/foo.txt"));
        assert!(sandbox.check_path_allowed("/var/data/db.sqlite"));
        assert!(!sandbox.check_path_allowed("/etc/passwd"));
        assert!(!sandbox.check_path_allowed("/home/user/file"));
    }

    #[test]
    fn test_env_allowed_no_restrictions() {
        let sandbox = Sandbox::with_defaults();
        assert!(sandbox.check_env_allowed("ANYTHING"));
    }

    #[test]
    fn test_env_allowed_with_restrictions() {
        let limits = SandboxLimits {
            allowed_env: Some(vec!["HOME".into(), "PATH".into()]),
            ..SandboxLimits::default()
        };
        let sandbox = Sandbox::new(limits);
        assert!(sandbox.check_env_allowed("HOME"));
        assert!(sandbox.check_env_allowed("PATH"));
        assert!(!sandbox.check_env_allowed("SECRET_KEY"));
    }

    #[test]
    fn test_verify_isolation_not_started() {
        let sandbox = Sandbox::with_defaults();
        let violations = sandbox.verify_isolation();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_verify_isolation_started_not_timed_out() {
        let mut sandbox = Sandbox::with_defaults();
        sandbox.start();
        let violations = sandbox.verify_isolation();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_sandbox_test_harness_new() {
        let config = SandboxConfig::default();
        let harness = SandboxTestHarness::new("test-plugin", vec![1, 2, 3], config);
        assert_eq!(harness.plugin_name, "test-plugin");
        assert_eq!(harness.wasm_bytes, vec![1, 2, 3]);
        assert_eq!(harness.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_sandbox_test_harness_with_timeout() {
        let config = SandboxConfig::default();
        let harness = SandboxTestHarness::new("p", vec![], config)
            .with_timeout(Duration::from_secs(10));
        assert_eq!(harness.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_sandbox_test_harness_record_call() {
        let config = SandboxConfig::default();
        let mut harness = SandboxTestHarness::new("p", vec![], config);
        harness.record_call("on_before_build");
        harness.record_call("on_after_build");
        assert_eq!(harness.recorded_calls.len(), 2);
        assert_eq!(harness.recorded_calls[0], "on_before_build");
    }

    #[test]
    fn test_sandbox_test_harness_record_memory_op() {
        let config = SandboxConfig::default();
        let mut harness = SandboxTestHarness::new("p", vec![], config);
        harness.record_memory_op("read", 0, 256);
        harness.record_memory_op("write", 1024, 512);
        assert_eq!(harness.memory_ops.len(), 2);
        assert_eq!(harness.memory_ops[0].op_type, "read");
        assert_eq!(harness.memory_ops[0].offset, 0);
        assert_eq!(harness.memory_ops[0].size, 256);
    }

    #[test]
    fn test_sandbox_test_harness_assert_hook_invoked() {
        let config = SandboxConfig::default();
        let mut harness = SandboxTestHarness::new("p", vec![], config);
        let report = harness.run_test("on_before_build", |_| Ok(vec![42]));
        assert!(harness.assert_hook_invoked(&report, "on_before_build"));
        assert!(!harness.assert_hook_invoked(&report, "on_after_build"));
    }

    #[test]
    fn test_sandbox_test_harness_assert_no_errors() {
        let config = SandboxConfig::default();
        let mut harness = SandboxTestHarness::new("p", vec![], config);
        let report = harness.run_test("on_before_build", |_| Ok(vec![42]));
        assert!(harness.assert_no_errors(&report));
    }

    #[test]
    fn test_sandbox_test_harness_assert_within_timeout() {
        let config = SandboxConfig::default();
        let mut harness = SandboxTestHarness::new("p", vec![], config);
        let report = harness.run_test("on_before_build", |_| Ok(vec![42]));
        assert!(harness.assert_within_timeout(&report));
    }

    #[test]
    fn test_sandbox_test_harness_run_test_success() {
        let config = SandboxConfig::default();
        let mut harness = SandboxTestHarness::new("p", vec![10, 20, 30], config);
        let report = harness.run_test("on_before_build", |bytes| Ok(bytes.to_vec()));
        assert!(report.passed);
        assert!(report.error.is_none());
        assert!(!report.timed_out);
        assert_eq!(report.hook_calls, vec!["on_before_build"]);
    }

    #[test]
    fn test_sandbox_test_harness_run_test_error() {
        let config = SandboxConfig::default();
        let mut harness = SandboxTestHarness::new("p", vec![], config);
        let report = harness.run_test("on_before_build", |_| {
            anyhow::bail!("something went wrong")
        });
        assert!(!report.passed);
        assert_eq!(report.error, Some("something went wrong".to_string()));
        assert!(!report.timed_out);
    }

    #[test]
    fn test_sandbox_test_harness_run_test_timeout_error() {
        let config = SandboxConfig::default();
        let mut harness = SandboxTestHarness::new("p", vec![], config);
        let report = harness.run_test("on_before_build", |_| {
            anyhow::bail!("timed out")
        });
        assert!(!report.passed);
        assert!(report.timed_out);
    }

    #[test]
    fn test_sandbox_test_harness_run_test_long_execution() {
        let config = SandboxConfig::default();
        let mut harness = SandboxTestHarness::new("p", vec![], config)
            .with_timeout(Duration::from_millis(1));
        let report = harness.run_test("on_before_build", |_| {
            std::thread::sleep(Duration::from_millis(10));
            Ok(vec![1])
        });
        assert!(!report.passed);
        assert!(report.timed_out);
        assert_eq!(report.error, Some("Sandbox timeout exceeded".to_string()));
    }
}
