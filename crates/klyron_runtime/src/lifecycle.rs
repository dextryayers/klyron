use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleMetrics {
    pub uptime_seconds: u64,
    pub total_restarts: u64,
    pub total_crashes: u64,
    pub active_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeState {
    Init,
    Running,
    Draining,
    Shutdown,
    Crashed,
}

pub struct RuntimeLifecycle {
    state: Mutex<RuntimeState>,
    start_time: Instant,
    restarts: AtomicU64,
    crashes: AtomicU64,
    shutdown_flag: AtomicBool,
    name: String,
    health_check_fn: Mutex<Option<Box<dyn Fn() -> Result<bool> + Send>>>,
    shutdown_hooks: Mutex<Vec<Box<dyn FnOnce() + Send>>>,
}

impl RuntimeLifecycle {
    pub fn new(name: &str) -> Self {
        Self {
            state: Mutex::new(RuntimeState::Init),
            start_time: Instant::now(),
            restarts: AtomicU64::new(0),
            crashes: AtomicU64::new(0),
            shutdown_flag: AtomicBool::new(false),
            name: name.to_string(),
            health_check_fn: Mutex::new(None),
            shutdown_hooks: Mutex::new(Vec::new()),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn state(&self) -> RuntimeState {
        *self.state.lock()
    }

    pub fn set_state(&self, new_state: RuntimeState) {
        let mut state = self.state.lock();
        info!("Runtime '{}' state: {:?} -> {:?}", self.name, *state, new_state);
        *state = new_state;
    }

    pub fn mark_running(&self) {
        self.set_state(RuntimeState::Running);
    }

    pub fn start_draining(&self) {
        self.set_state(RuntimeState::Draining);
    }

    pub fn start_shutdown(&self) {
        self.shutdown_flag.store(true, Ordering::SeqCst);
        self.set_state(RuntimeState::Shutdown);
    }

    pub fn mark_crashed(&self) {
        self.crashes.fetch_add(1, Ordering::SeqCst);
        self.set_state(RuntimeState::Crashed);
    }

    pub fn record_restart(&self) {
        self.restarts.fetch_add(1, Ordering::SeqCst);
    }

    pub fn is_shutting_down(&self) -> bool {
        self.shutdown_flag.load(Ordering::SeqCst)
    }

    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn metrics(&self) -> LifecycleMetrics {
        LifecycleMetrics {
            uptime_seconds: self.uptime().as_secs(),
            total_restarts: self.restarts.load(Ordering::SeqCst),
            total_crashes: self.crashes.load(Ordering::SeqCst),
            active_count: if *self.state.lock() == RuntimeState::Running { 1 } else { 0 },
        }
    }

    pub fn set_health_check<F>(&self, check: F)
    where
        F: Fn() -> Result<bool> + Send + 'static,
    {
        *self.health_check_fn.lock() = Some(Box::new(check));
    }

    pub fn is_healthy(&self) -> bool {
        if *self.state.lock() != RuntimeState::Running {
            return false;
        }
        if let Some(ref check) = *self.health_check_fn.lock() {
            check().unwrap_or(false)
        } else {
            true
        }
    }

    pub fn add_shutdown_hook<F>(&self, hook: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.shutdown_hooks.lock().push(Box::new(hook));
    }

    pub fn shutdown(&self) {
        info!("Runtime '{}' shutting down...", self.name);
        self.start_shutdown();
        let hooks = self.shutdown_hooks.lock().drain(..).collect::<Vec<_>>();
        for hook in hooks {
            hook();
        }
        info!("Runtime '{}' shutdown complete", self.name);
    }

    pub fn restart(&self) -> Result<()> {
        info!("Runtime '{}' restarting...", self.name);
        self.record_restart();
        self.set_state(RuntimeState::Running);
        Ok(())
    }
}

pub fn watch_lifecycle(lifecycle: Arc<RuntimeLifecycle>, check_interval: Duration) {
    tokio::spawn(async move {
        loop {
            if lifecycle.is_shutting_down() {
                break;
            }
            if !lifecycle.is_healthy() {
                warn!("Runtime '{}' health check failed", lifecycle.name());
                lifecycle.mark_crashed();
            }
            tokio::time::sleep(check_interval).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_initial_state() {
        let lc = RuntimeLifecycle::new("test");
        assert_eq!(lc.state(), RuntimeState::Init);
        assert!(!lc.is_shutting_down());
    }

    #[test]
    fn test_lifecycle_transitions() {
        let lc = RuntimeLifecycle::new("test");
        lc.mark_running();
        assert_eq!(lc.state(), RuntimeState::Running);
        assert!(lc.is_healthy());
        lc.start_shutdown();
        assert_eq!(lc.state(), RuntimeState::Shutdown);
        assert!(lc.is_shutting_down());
    }

    #[test]
    fn test_lifecycle_metrics() {
        let lc = RuntimeLifecycle::new("test");
        lc.mark_running();
        let m = lc.metrics();
        assert_eq!(m.total_restarts, 0);
        assert_eq!(m.total_crashes, 0);
        lc.record_restart();
        lc.mark_crashed();
        let m = lc.metrics();
        assert_eq!(m.total_restarts, 1);
        assert_eq!(m.total_crashes, 1);
    }

    #[test]
    fn test_shutdown_hooks() {
        let lc = RuntimeLifecycle::new("test");
        let flag = Arc::new(AtomicBool::new(false));
        let f = flag.clone();
        lc.add_shutdown_hook(move || {
            f.store(true, Ordering::SeqCst);
        });
        lc.shutdown();
        assert!(flag.load(Ordering::SeqCst));
    }

    #[test]
    fn test_health_check() {
        let lc = RuntimeLifecycle::new("test");
        lc.mark_running();
        lc.set_health_check(|| Ok(true));
        assert!(lc.is_healthy());
        lc.set_health_check(|| Ok(false));
        assert!(!lc.is_healthy());
    }
}
