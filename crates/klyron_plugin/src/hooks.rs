use crate::manifest::HookPhase;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

pub type HookFn = Arc<dyn Fn(&str, &[u8]) -> Result<Vec<u8>> + Send + Sync>;

#[derive(Clone)]
pub struct HookHandler {
    pub plugin_name: String,
    pub phase: HookPhase,
    pub handler: HookFn,
}

pub struct HookRegistry {
    hooks: HashMap<HookPhase, Vec<HookHandler>>,
    execution_order: Vec<HookPhase>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
            execution_order: HookPhase::all(),
        }
    }

    pub fn register(&mut self, handler: HookHandler) {
        self.hooks
            .entry(handler.phase.clone())
            .or_default()
            .push(handler);
    }

    pub fn unregister(&mut self, plugin_name: &str) {
        for handlers in self.hooks.values_mut() {
            handlers.retain(|h| h.plugin_name != plugin_name);
        }
    }

    pub fn unregister_phase(&mut self, plugin_name: &str, phase: &HookPhase) {
        if let Some(handlers) = self.hooks.get_mut(phase) {
            handlers.retain(|h| h.plugin_name != plugin_name);
        }
    }

    pub fn execute_phase(&self, phase: &HookPhase, context: &[u8]) -> Vec<HookResult> {
        let mut results = Vec::new();
        if let Some(handlers) = self.hooks.get(phase) {
            for handler in handlers {
                let start = std::time::Instant::now();
                match (handler.handler)(&handler.plugin_name, context) {
                    Ok(data) => {
                        let elapsed = start.elapsed();
                        info!(
                            "Hook {} executed for plugin {} in {:?}",
                            phase.as_str(),
                            handler.plugin_name,
                            elapsed
                        );
                        results.push(HookResult::Success {
                            plugin: handler.plugin_name.clone(),
                            data,
                            duration: elapsed,
                        });
                    }
                    Err(e) => {
                        let elapsed = start.elapsed();
                        warn!(
                            "Hook {} failed for plugin {}: {}",
                            phase.as_str(),
                            handler.plugin_name,
                            e
                        );
                        results.push(HookResult::Failure {
                            plugin: handler.plugin_name.clone(),
                            error: e.to_string(),
                            duration: elapsed,
                        });
                    }
                }
            }
        }
        results
    }

    pub fn execute_phase_with_rollback(
        &self,
        phase: &HookPhase,
        context: &[u8],
    ) -> Vec<HookResult> {
        let results = self.execute_phase(phase, context);
        let has_failure = results.iter().any(|r| matches!(r, HookResult::Failure { .. }));
        if has_failure {
            for r in &results {
                if let HookResult::Success { plugin, .. } = r {
                    info!("Rolling back hook {} for plugin {}", phase.as_str(), plugin);
                }
            }
        }
        results
    }

    pub fn plugins_for_phase(&self, phase: &HookPhase) -> Vec<&str> {
        self.hooks
            .get(phase)
            .map(|h| h.iter().map(|h| h.plugin_name.as_str()).collect())
            .unwrap_or_default()
    }

    pub fn is_registered(&self, plugin_name: &str, phase: &HookPhase) -> bool {
        self.hooks
            .get(phase)
            .map(|h| h.iter().any(|h| h.plugin_name == plugin_name))
            .unwrap_or(false)
    }

    pub fn all_hooks(&self) -> Vec<&HookHandler> {
        self.hooks
            .values()
            .flat_map(|v| v.iter())
            .collect()
    }

    pub fn count(&self) -> usize {
        self.hooks.values().map(|v| v.len()).sum()
    }
}

#[derive(Debug, Clone)]
pub enum HookResult {
    Success {
        plugin: String,
        data: Vec<u8>,
        duration: std::time::Duration,
    },
    Failure {
        plugin: String,
        error: String,
        duration: std::time::Duration,
    },
}

impl HookResult {
    pub fn plugin_name(&self) -> &str {
        match self {
            HookResult::Success { plugin, .. } => plugin,
            HookResult::Failure { plugin, .. } => plugin,
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, HookResult::Success { .. })
    }

    pub fn duration(&self) -> std::time::Duration {
        match self {
            HookResult::Success { duration, .. } => *duration,
            HookResult::Failure { duration, .. } => *duration,
        }
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}
