use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type Listener = Box<dyn Fn(&[serde_json::Value])>;

#[derive(Clone)]
pub struct EventEmitter {
    listeners: Rc<RefCell<HashMap<String, Vec<Listener>>>>,
    max_listeners: Rc<RefCell<usize>>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self {
            listeners: Rc::new(RefCell::new(HashMap::new())),
            max_listeners: Rc::new(RefCell::new(10)),
        }
    }

    pub fn on<F>(&self, event: &str, listener: F)
    where
        F: Fn(&[serde_json::Value]) + 'static,
    {
        let mut map = self.listeners.borrow_mut();
        map.entry(event.to_string())
            .or_default()
            .push(Box::new(listener));
    }

    pub fn once<F>(&self, event: &str, listener: F)
    where
        F: Fn(&[serde_json::Value]) + 'static,
    {
        let event_owned = event.to_string();
        let listeners_clone = self.listeners.clone();
        let wrapped = move |args: &[serde_json::Value]| {
            listener(args);
            let mut map = listeners_clone.borrow_mut();
            if let Some(listeners) = map.get_mut(&event_owned) {
                listeners.retain(|_| false);
            }
        };
        self.on(&event, wrapped);
    }

    pub fn emit(&self, event: &str, args: &[serde_json::Value]) -> bool {
        let map = self.listeners.borrow();
        if let Some(listeners) = map.get(event) {
            for listener in listeners {
                listener(args);
            }
            true
        } else {
            false
        }
    }

    pub fn remove_all_listeners(&self, event: &str) {
        let mut map = self.listeners.borrow_mut();
        map.remove(event);
    }

    pub fn listener_count(&self, event: &str) -> usize {
        let map = self.listeners.borrow();
        map.get(event).map(|l| l.len()).unwrap_or(0)
    }

    pub fn event_names(&self) -> Vec<String> {
        let map = self.listeners.borrow();
        map.keys().cloned().collect()
    }

    pub fn set_max_listeners(&self, n: usize) {
        *self.max_listeners.borrow_mut() = n;
    }

    pub fn get_max_listeners(&self) -> usize {
        *self.max_listeners.borrow()
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

    #[test]
    fn test_event_emitter_on_emit() {
        let emitter = EventEmitter::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        emitter.on("test", move |_| {
            called_clone.store(true, Ordering::SeqCst);
        });

        assert!(emitter.emit("test", &[]));
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_event_emitter_unregistered_event() {
        let emitter = EventEmitter::new();
        assert!(!emitter.emit("nonexistent", &[]));
    }

    #[test]
    fn test_event_emitter_listener_count() {
        let emitter = EventEmitter::new();
        emitter.on("data", |_| {});
        emitter.on("data", |_| {});
        assert_eq!(emitter.listener_count("data"), 2);
    }

    #[test]
    fn test_event_emitter_remove_all() {
        let emitter = EventEmitter::new();
        emitter.on("data", |_| {});
        emitter.remove_all_listeners("data");
        assert!(!emitter.emit("data", &[]));
    }

    #[test]
    fn test_event_emitter_max_listeners() {
        let emitter = EventEmitter::new();
        assert_eq!(emitter.get_max_listeners(), 10);
        emitter.set_max_listeners(20);
        assert_eq!(emitter.get_max_listeners(), 20);
    }

    #[test]
    fn test_event_emitter_event_names() {
        let emitter = EventEmitter::new();
        emitter.on("a", |_| {});
        emitter.on("b", |_| {});
        let names = emitter.event_names();
        assert!(names.contains(&"a".to_string()));
        assert!(names.contains(&"b".to_string()));
    }
}
