use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;

pub struct InternTable {
    inner: RwLock<HashMap<Arc<str>, u32>>,
    strings: RwLock<Vec<Arc<str>>>,
}

impl InternTable {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
            strings: RwLock::new(Vec::new()),
        }
    }

    pub fn intern(&self, s: &str) -> u32 {
        {
            let read = self.inner.read();
            if let Some(&id) = read.get(s) {
                return id;
            }
        }
        let arc: Arc<str> = Arc::from(s);
        let mut write = self.inner.write();
        if let Some(&id) = write.get(&arc) {
            return id;
        }
        let mut strings = self.strings.write();
        let id = strings.len() as u32;
        strings.push(arc.clone());
        write.insert(arc, id);
        id
    }

    pub fn lookup(&self, id: u32) -> Option<Arc<str>> {
        let strings = self.strings.read();
        strings.get(id as usize).cloned()
    }

    pub fn get_id(&self, s: &str) -> Option<u32> {
        self.inner.read().get(s).copied()
    }

    pub fn len(&self) -> usize {
        self.strings.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.strings.read().is_empty()
    }

    pub fn clear(&self) {
        self.inner.write().clear();
        self.strings.write().clear();
    }
}

impl Default for InternTable {
    fn default() -> Self {
        Self::new()
    }
}
