use std::sync::Arc;

use dashmap::DashMap;

pub struct InternCache {
    map: DashMap<String, Arc<str>>,
}

impl InternCache {
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            map: DashMap::with_capacity(cap),
        }
    }

    pub fn get_or_intern(&self, s: &str) -> Arc<str> {
        if let Some(entry) = self.map.get(s) {
            return entry.clone();
        }
        let arc: Arc<str> = Arc::from(s);
        self.map.insert(s.to_string(), arc.clone());
        arc
    }

    pub fn get(&self, s: &str) -> Option<Arc<str>> {
        self.map.get(s).map(|v| v.clone())
    }

    pub fn contains(&self, s: &str) -> bool {
        self.map.contains_key(s)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn clear(&self) {
        self.map.clear();
    }

    pub fn remove(&self, s: &str) -> Option<Arc<str>> {
        self.map.remove(s).map(|(_, v)| v)
    }
}

impl Default for InternCache {
    fn default() -> Self {
        Self::new()
    }
}
