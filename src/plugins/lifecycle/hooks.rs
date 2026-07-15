use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

pub type HookFn = Arc<dyn Fn(&str, &[u8]) -> Result<Vec<u8>> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct HookHandler {
    pub plugin_name: String,
    pub phase: HookPhase,
    pub handler: HookFn,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum HookPhase {
    PreInstall,
    PostInstall,
    PreBuild,
    PostBuild,
    PreDev,
    PostDev,
    PreRequest,
    PostRequest,
}

impl HookPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookPhase::PreInstall => "pre_install",
            HookPhase::PostInstall => "post_install",
            HookPhase::PreBuild => "pre_build",
            HookPhase::PostBuild => "post_build",
            HookPhase::PreDev => "pre_dev",
            HookPhase::PostDev => "post_dev",
            HookPhase::PreRequest => "pre_request",
            HookPhase::PostRequest => "post_request",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pre_install" | "PRE_INSTALL" => Some(HookPhase::PreInstall),
            "post_install" | "POST_INSTALL" => Some(HookPhase::PostInstall),
            "pre_build" | "PRE_BUILD" => Some(HookPhase::PreBuild),
            "post_build" | "POST_BUILD" => Some(HookPhase::PostBuild),
            "pre_dev" | "PRE_DEV" => Some(HookPhase::PreDev),
            "post_dev" | "POST_DEV" => Some(HookPhase::PostDev),
            "pre_request" | "PRE_REQUEST" => Some(HookPhase::PreRequest),
            "post_request" | "POST_REQUEST" => Some(HookPhase::PostRequest),
            _ => None,
        }
    }

    pub fn all() -> Vec<HookPhase> {
        vec![
            HookPhase::PreInstall, HookPhase::PostInstall,
            HookPhase::PreBuild, HookPhase::PostBuild,
            HookPhase::PreDev, HookPhase::PostDev,
            HookPhase::PreRequest, HookPhase::PostRequest,
        ]
    }
}

#[derive(Debug, Clone)]
pub enum HookResult {
    Success { plugin: String, data: Vec<u8>, duration: std::time::Duration },
    Failure { plugin: String, error: String, duration: std::time::Duration },
}

impl HookResult {
    pub fn is_success(&self) -> bool {
        matches!(self, HookResult::Success { .. })
    }

    pub fn plugin_name(&self) -> &str {
        match self {
            HookResult::Success { plugin, .. } => plugin,
            HookResult::Failure { plugin, .. } => plugin,
        }
    }
}

pub struct HookRegistry {
    hooks: HashMap<HookPhase, Vec<HookHandler>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self { hooks: HashMap::new() }
    }

    pub fn register(&mut self, handler: HookHandler) {
        self.hooks.entry(handler.phase.clone()).or_default().push(handler);
    }

    pub fn unregister(&mut self, plugin_name: &str) {
        for handlers in self.hooks.values_mut() {
            handlers.retain(|h| h.plugin_name != plugin_name);
        }
    }

    pub fn execute_phase(&self, phase: &HookPhase, context: &[u8]) -> Vec<HookResult> {
        let mut results = Vec::new();
        if let Some(handlers) = self.hooks.get(phase) {
            for handler in handlers {
                let start = Instant::now();
                match (handler.handler)(&handler.plugin_name, context) {
                    Ok(data) => {
                        results.push(HookResult::Success {
                            plugin: handler.plugin_name.clone(),
                            data,
                            duration: start.elapsed(),
                        });
                    }
                    Err(e) => {
                        warn!("Hook {} failed for {}: {}", phase.as_str(), handler.plugin_name, e);
                        results.push(HookResult::Failure {
                            plugin: handler.plugin_name.clone(),
                            error: e.to_string(),
                            duration: start.elapsed(),
                        });
                    }
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

    pub fn count(&self) -> usize {
        self.hooks.values().map(|v| v.len()).sum()
    }

    pub fn all_hooks(&self) -> Vec<&HookHandler> {
        self.hooks.values().flat_map(|v| v.iter()).collect()
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}
