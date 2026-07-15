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
