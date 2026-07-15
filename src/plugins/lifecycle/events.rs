use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde_json::Value;

pub type EventHandler = Arc<dyn Fn(&str, &Value) -> Result<()> + Send + Sync>;

pub struct EventBus {
    listeners: Arc<RwLock<HashMap<String, Vec<EventHandler>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn subscribe<F>(&self, event_type: &str, handler: F)
    where
        F: Fn(&str, &Value) -> Result<()> + Send + Sync + 'static,
    {
        self.listeners
            .write()
            .entry(event_type.to_string())
            .or_default()
            .push(Arc::new(handler));
    }

    pub fn emit(&self, event_type: &str, payload: &Value) -> Vec<Result<()>> {
        let mut results = Vec::new();
        if let Some(handlers) = self.listeners.read().get(event_type) {
            for handler in handlers {
                results.push(handler(event_type, payload));
            }
        }
        results
    }

    pub fn unsubscribe_all(&self, event_type: &str) {
        self.listeners.write().remove(event_type);
    }

    pub fn clear(&self) {
        self.listeners.write().clear();
    }

    pub fn listener_count(&self, event_type: &str) -> usize {
        self.listeners
            .read()
            .get(event_type)
            .map(|v| v.len())
            .unwrap_or(0)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_bus_subscribe_emit() {
        let bus = EventBus::new();
        let emitted = Arc::new(RwLock::new(Vec::new()));
        let emitted_clone = emitted.clone();

        bus.subscribe("test.event", move |_event_type, payload| {
            emitted_clone.write().push(payload.clone());
            Ok(())
        });

        let payload = serde_json::json!({"key": "value"});
        let results = bus.emit("test.event", &payload);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());

        let stored = emitted.read();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0], payload);
    }

    #[test]
    fn test_event_bus_no_listeners() {
        let bus = EventBus::new();
        let results = bus.emit("nonexistent", &serde_json::json!({}));
        assert!(results.is_empty());
    }
}
