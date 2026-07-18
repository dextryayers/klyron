use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::lifecycle::RuntimeLifecycle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionPolicy {
    pub max_restarts: u32,
    pub restart_window: Duration,
    pub restart_delay: Duration,
    pub enable_crash_dumps: bool,
}

impl Default for SupervisionPolicy {
    fn default() -> Self {
        Self {
            max_restarts: 5,
            restart_window: Duration::from_secs(60),
            restart_delay: Duration::from_millis(500),
            enable_crash_dumps: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStatus {
    pub pid: Option<u32>,
    pub name: String,
    pub state: String,
    pub uptime_seconds: u64,
    pub restarts: u64,
}

struct ManagedProcess {
    lifecycle: Arc<RuntimeLifecycle>,
    policy: SupervisionPolicy,
    start_times: Vec<Instant>,
}

pub struct Supervisor {
    processes: Mutex<HashMap<String, ManagedProcess>>,
    total_restarts: AtomicU64,
}

impl Supervisor {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
            total_restarts: AtomicU64::new(0),
        }
    }

    pub fn register(&self, name: &str, lifecycle: Arc<RuntimeLifecycle>) {
        self.register_with_policy(name, lifecycle, SupervisionPolicy::default());
    }

    pub fn register_with_policy(
        &self,
        name: &str,
        lifecycle: Arc<RuntimeLifecycle>,
        policy: SupervisionPolicy,
    ) {
        let mut procs = self.processes.lock();
        procs.insert(name.to_string(), ManagedProcess {
            lifecycle,
            policy,
            start_times: Vec::new(),
        });
        info!("Supervisor registered process: {}", name);
    }

    pub fn unregister(&self, name: &str) {
        let mut procs = self.processes.lock();
        procs.remove(name);
        info!("Supervisor unregistered process: {}", name);
    }

    pub fn list_processes(&self) -> Vec<ProcessStatus> {
        let procs = self.processes.lock();
        procs.iter().map(|(name, mp)| {
            let lc = &mp.lifecycle;
            ProcessStatus {
                pid: None,
                name: name.clone(),
                state: format!("{:?}", lc.state()),
                uptime_seconds: lc.uptime().as_secs(),
                restarts: mp.start_times.len() as u64,
            }
        }).collect()
    }

    pub fn supervise(&self, name: &str) -> Result<()> {
        let mut procs = self.processes.lock();
        let mp = procs.get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Process '{}' not registered", name))?;

        let now = Instant::now();
        mp.start_times.retain(|t| now.duration_since(*t) < mp.policy.restart_window);

        if mp.start_times.len() >= mp.policy.max_restarts as usize {
            anyhow::bail!(
                "Process '{}' exceeded max restarts ({}) in window {:?}",
                name, mp.policy.max_restarts, mp.policy.restart_window
            );
        }

        mp.start_times.push(now);
        self.total_restarts.fetch_add(1, Ordering::SeqCst);
        mp.lifecycle.record_restart();
        mp.lifecycle.mark_running();
        info!("Supervisor restarted process: {}", name);
        Ok(())
    }

    pub fn report_crash(&self, name: &str) {
        let mut procs = self.processes.lock();
        if let Some(mp) = procs.get_mut(name) {
            mp.lifecycle.mark_crashed();
            error!("Supervisor detected crash in process: {}", name);
        }
    }

    pub fn shutdown_all(&self) {
        info!("Supervisor shutting down all processes...");
        let procs = self.processes.lock();
        for (name, mp) in procs.iter() {
            info!("Shutting down process: {}", name);
            mp.lifecycle.shutdown();
        }
    }

    pub fn shutdown(&self, name: &str) -> Result<()> {
        let procs = self.processes.lock();
        let mp = procs.get(name)
            .ok_or_else(|| anyhow::anyhow!("Process '{}' not registered", name))?;
        mp.lifecycle.shutdown();
        Ok(())
    }

    pub fn total_restarts(&self) -> u64 {
        self.total_restarts.load(Ordering::SeqCst)
    }

    pub fn process_count(&self) -> usize {
        self.processes.lock().len()
    }
}

impl Default for Supervisor {
    fn default() -> Self {
        Self::new()
    }
}

pub fn supervise_process(lifecycle: Arc<RuntimeLifecycle>, _policy: SupervisionPolicy) {
    let lc = lifecycle.clone();
    tokio::spawn(async move {
        loop {
            if lc.is_shutting_down() {
                break;
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::RuntimeState;

    #[test]
    fn test_supervisor_register() {
        let sup = Supervisor::new();
        let lc = Arc::new(RuntimeLifecycle::new("proc1"));
        sup.register("proc1", lc);
        assert_eq!(sup.process_count(), 1);
    }

    #[test]
    fn test_supervisor_list() {
        let sup = Supervisor::new();
        let lc = Arc::new(RuntimeLifecycle::new("proc1"));
        sup.register("proc1", lc.clone());
        lc.mark_running();
        let list = sup.list_processes();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "proc1");
    }

    #[test]
    fn test_supervisor_unregister() {
        let sup = Supervisor::new();
        let lc = Arc::new(RuntimeLifecycle::new("proc1"));
        sup.register("proc1", lc);
        sup.unregister("proc1");
        assert_eq!(sup.process_count(), 0);
    }

    #[test]
    fn test_supervise_restart_limit() {
        let sup = Supervisor::new();
        let lc = Arc::new(RuntimeLifecycle::new("proc1"));
        let policy = SupervisionPolicy {
            max_restarts: 2,
            restart_window: Duration::from_secs(60),
            restart_delay: Duration::from_millis(0),
            enable_crash_dumps: false,
        };
        sup.register_with_policy("proc1", lc, policy);
        assert!(sup.supervise("proc1").is_ok());
        assert!(sup.supervise("proc1").is_ok());
        assert!(sup.supervise("proc1").is_err());
    }

    #[test]
    fn test_supervisor_shutdown_all() {
        let sup = Supervisor::new();
        let lc = Arc::new(RuntimeLifecycle::new("proc1"));
        sup.register("proc1", lc.clone());
        lc.mark_running();
        sup.shutdown_all();
        assert_eq!(lc.state(), RuntimeState::Shutdown);
    }

    #[test]
    fn test_supervisor_report_crash() {
        let sup = Supervisor::new();
        let lc = Arc::new(RuntimeLifecycle::new("proc1"));
        sup.register("proc1", lc.clone());
        lc.mark_running();
        sup.report_crash("proc1");
        assert_eq!(lc.state(), RuntimeState::Crashed);
    }
}
