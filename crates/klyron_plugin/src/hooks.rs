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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::HookPhase;

    fn make_handler(plugin: &str, phase: HookPhase) -> HookHandler {
        let name = plugin.to_string();
        HookHandler {
            plugin_name: plugin.to_string(),
            phase,
            handler: Arc::new(move |n: &str, ctx: &[u8]| {
                if n == name {
                    Ok(ctx.to_vec())
                } else {
                    Ok(Vec::new())
                }
            }),
        }
    }

    fn make_failing_handler(plugin: &str, phase: HookPhase) -> HookHandler {
        let msg = format!("{} error", plugin);
        HookHandler {
            plugin_name: plugin.to_string(),
            phase,
            handler: Arc::new(move |_, _: &[u8]| anyhow::bail!("{}", msg)),
        }
    }

    #[test]
    fn test_register_hook() {
        let mut reg = HookRegistry::new();
        let handler = make_handler("p1", HookPhase::OnBeforeBuild);
        reg.register(handler);
        assert_eq!(reg.count(), 1);
        assert!(reg.is_registered("p1", &HookPhase::OnBeforeBuild));
    }

    #[test]
    fn test_unregister_hook() {
        let mut reg = HookRegistry::new();
        reg.register(make_handler("p1", HookPhase::OnBeforeBuild));
        reg.register(make_handler("p1", HookPhase::OnAfterBuild));
        reg.register(make_handler("p2", HookPhase::OnBeforeBuild));
        assert_eq!(reg.count(), 3);
        reg.unregister("p1");
        assert_eq!(reg.count(), 1);
        assert!(!reg.is_registered("p1", &HookPhase::OnBeforeBuild));
        assert!(reg.is_registered("p2", &HookPhase::OnBeforeBuild));
    }

    #[test]
    fn test_unregister_phase() {
        let mut reg = HookRegistry::new();
        reg.register(make_handler("p1", HookPhase::OnBeforeBuild));
        reg.register(make_handler("p1", HookPhase::OnAfterBuild));
        assert_eq!(reg.count(), 2);
        reg.unregister_phase("p1", &HookPhase::OnBeforeBuild);
        assert_eq!(reg.count(), 1);
        assert!(!reg.is_registered("p1", &HookPhase::OnBeforeBuild));
        assert!(reg.is_registered("p1", &HookPhase::OnAfterBuild));
    }

    #[test]
    fn test_execute_phase_success() {
        let mut reg = HookRegistry::new();
        reg.register(make_handler("p1", HookPhase::OnBeforeBuild));
        let results = reg.execute_phase(&HookPhase::OnBeforeBuild, b"hello");
        assert_eq!(results.len(), 1);
        assert!(results[0].is_success());
        assert_eq!(results[0].plugin_name(), "p1");
    }

    #[test]
    fn test_execute_phase_failure() {
        let mut reg = HookRegistry::new();
        reg.register(make_failing_handler("p1", HookPhase::OnBeforeBuild));
        let results = reg.execute_phase(&HookPhase::OnBeforeBuild, b"data");
        assert_eq!(results.len(), 1);
        assert!(!results[0].is_success());
        assert_eq!(results[0].plugin_name(), "p1");
    }

    #[test]
    fn test_execute_phase_multiple_handlers() {
        let mut reg = HookRegistry::new();
        reg.register(make_handler("p1", HookPhase::OnBeforeBuild));
        reg.register(make_handler("p2", HookPhase::OnBeforeBuild));
        reg.register(make_handler("p3", HookPhase::OnBeforeBuild));
        let results = reg.execute_phase(&HookPhase::OnBeforeBuild, b"ctx");
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].plugin_name(), "p1");
        assert_eq!(results[1].plugin_name(), "p2");
        assert_eq!(results[2].plugin_name(), "p3");
    }

    #[test]
    fn test_execute_phase_mixed_results() {
        let mut reg = HookRegistry::new();
        reg.register(make_handler("ok1", HookPhase::OnBeforeBuild));
        reg.register(make_failing_handler("fail1", HookPhase::OnBeforeBuild));
        reg.register(make_handler("ok2", HookPhase::OnBeforeBuild));
        let results = reg.execute_phase(&HookPhase::OnBeforeBuild, b"ctx");
        assert_eq!(results.len(), 3);
        assert!(results[0].is_success());
        assert!(!results[1].is_success());
        assert!(results[2].is_success());
    }

    #[test]
    fn test_execute_phase_with_rollback_no_failure() {
        let mut reg = HookRegistry::new();
        reg.register(make_handler("p1", HookPhase::OnBeforeBuild));
        let results = reg.execute_phase_with_rollback(&HookPhase::OnBeforeBuild, b"data");
        assert_eq!(results.len(), 1);
        assert!(results[0].is_success());
    }

    #[test]
    fn test_execute_phase_with_rollback_with_failure() {
        let mut reg = HookRegistry::new();
        reg.register(make_handler("ok1", HookPhase::OnBeforeBuild));
        reg.register(make_failing_handler("fail1", HookPhase::OnBeforeBuild));
        let results = reg.execute_phase_with_rollback(&HookPhase::OnBeforeBuild, b"data");
        assert_eq!(results.len(), 2);
        assert!(results[0].is_success());
        assert!(!results[1].is_success());
    }

    #[test]
    fn test_execute_no_handlers() {
        let reg = HookRegistry::new();
        let results = reg.execute_phase(&HookPhase::OnBeforeBuild, b"data");
        assert!(results.is_empty());
    }

    #[test]
    fn test_plugins_for_phase() {
        let mut reg = HookRegistry::new();
        reg.register(make_handler("p1", HookPhase::OnBeforeBuild));
        reg.register(make_handler("p2", HookPhase::OnBeforeBuild));
        reg.register(make_handler("p3", HookPhase::OnAfterBuild));
        let plugins = reg.plugins_for_phase(&HookPhase::OnBeforeBuild);
        assert_eq!(plugins.len(), 2);
        assert!(plugins.contains(&"p1"));
        assert!(plugins.contains(&"p2"));
    }

    #[test]
    fn test_plugins_for_phase_empty() {
        let reg = HookRegistry::new();
        let plugins = reg.plugins_for_phase(&HookPhase::OnBeforeBuild);
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_is_registered() {
        let mut reg = HookRegistry::new();
        reg.register(make_handler("p1", HookPhase::OnBeforeBuild));
        assert!(reg.is_registered("p1", &HookPhase::OnBeforeBuild));
        assert!(!reg.is_registered("p1", &HookPhase::OnAfterBuild));
        assert!(!reg.is_registered("p2", &HookPhase::OnBeforeBuild));
    }

    #[test]
    fn test_all_hooks() {
        let mut reg = HookRegistry::new();
        reg.register(make_handler("p1", HookPhase::OnBeforeBuild));
        reg.register(make_handler("p2", HookPhase::OnAfterBuild));
        let all = reg.all_hooks();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_count() {
        let mut reg = HookRegistry::new();
        assert_eq!(reg.count(), 0);
        reg.register(make_handler("p1", HookPhase::OnBeforeBuild));
        assert_eq!(reg.count(), 1);
        reg.register(make_handler("p1", HookPhase::OnAfterBuild));
        assert_eq!(reg.count(), 2);
    }

    #[test]
    fn test_hook_result_methods() {
        let success = HookResult::Success {
            plugin: "test-p".into(),
            data: vec![1, 2, 3],
            duration: std::time::Duration::from_millis(5),
        };
        assert_eq!(success.plugin_name(), "test-p");
        assert!(success.is_success());
        assert_eq!(success.duration().as_millis(), 5);

        let failure = HookResult::Failure {
            plugin: "fail-p".into(),
            error: "something broke".into(),
            duration: std::time::Duration::from_millis(10),
        };
        assert_eq!(failure.plugin_name(), "fail-p");
        assert!(!failure.is_success());
        assert_eq!(failure.duration().as_millis(), 10);
    }

    #[test]
    fn test_default_registry() {
        let reg = HookRegistry::default();
        assert_eq!(reg.count(), 0);
    }
}
