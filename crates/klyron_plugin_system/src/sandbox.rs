use anyhow::Result;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::manifest::SandboxConfig;

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

pub struct PluginSandbox {
    limits: SandboxLimits,
    fuel_consumed: Arc<AtomicU64>,
    start_time: parking_lot::RwLock<Option<Instant>>,
}

impl PluginSandbox {
    pub fn new(limits: SandboxLimits) -> Self {
        Self {
            limits,
            fuel_consumed: Arc::new(AtomicU64::new(0)),
            start_time: parking_lot::RwLock::new(None),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(SandboxLimits::default())
    }

    pub fn from_config(config: SandboxConfig) -> Self {
        Self::new(SandboxLimits::from(config))
    }

    pub fn limits(&self) -> &SandboxLimits {
        &self.limits
    }

    pub fn start_execution(&self) {
        *self.start_time.write() = Some(Instant::now());
    }

    pub fn reset(&self) {
        self.fuel_consumed.store(0, Ordering::SeqCst);
        *self.start_time.write() = None;
    }

    pub fn is_timed_out(&self) -> bool {
        if let Some(start) = *self.start_time.read() {
            start.elapsed() > Duration::from_millis(self.limits.max_cpu_ms)
        } else {
            false
        }
    }

    pub fn remaining_time(&self) -> Duration {
        if let Some(start) = *self.start_time.read() {
            let elapsed = start.elapsed();
            let max = Duration::from_millis(self.limits.max_cpu_ms);
            if elapsed >= max { Duration::ZERO } else { max - elapsed }
        } else {
            Duration::from_millis(self.limits.max_cpu_ms)
        }
    }

    pub fn consume_fuel(&self, amount: u64) -> bool {
        let current = self.fuel_consumed.fetch_add(amount, Ordering::SeqCst);
        (current + amount) <= self.limits.max_fuel
    }

    pub fn fuel_remaining(&self) -> u64 {
        self.limits.max_fuel.saturating_sub(self.fuel_consumed.load(Ordering::SeqCst))
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
        if self.is_timed_out() {
            violations.push("Sandbox timed out".to_string());
        }
        violations
    }
}
