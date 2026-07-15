pub mod hooks;
pub mod events;

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use anyhow::Result;
use tracing::info;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleState {
    Initializing,
    Ready,
    Running,
    Stopping,
    Stopped,
    Error(String),
}

pub struct LifecycleManager {
    states: Arc<RwLock<HashMap<String, LifecycleState>>>,
    hooks: Arc<RwLock<HashMap<String, Vec<Box<dyn Fn(&str) -> Result<()> + Send + Sync>>>>>,
}

impl LifecycleManager {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            hooks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set_state(&self, name: &str, state: LifecycleState) {
        self.states.write().insert(name.to_string(), state);
    }

    pub fn get_state(&self, name: &str) -> Option<LifecycleState> {
        self.states.read().get(name).cloned()
    }

    pub fn register_hook<F>(&self, name: &str, hook: F)
    where
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        self.hooks
            .write()
            .entry(name.to_string())
            .or_default()
            .push(Box::new(hook));
    }

    pub fn execute_hooks(&self, name: &str) -> Result<()> {
        if let Some(hooks) = self.hooks.read().get(name) {
            for hook in hooks {
                hook(name)?;
            }
        }
        Ok(())
    }

    pub fn transition(&self, name: &str, new_state: LifecycleState) -> Result<()> {
        let current = self.get_state(name);
        info!("Plugin '{}' state transition: {:?} -> {:?}", name, current, new_state);
        self.set_state(name, new_state);
        Ok(())
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}
