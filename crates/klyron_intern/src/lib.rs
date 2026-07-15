use std::sync::Arc;

use dashmap::DashSet;

pub struct Interner {
    pool: DashSet<Arc<str>>,
}

impl Interner {
    pub fn new() -> Self {
        Self {
            pool: DashSet::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            pool: DashSet::with_capacity(cap),
        }
    }

    pub fn intern(&self, s: &str) -> Arc<str> {
        if let Some(existing) = self.pool.get(s) {
            return existing.clone();
        }
        let arc: Arc<str> = Arc::from(s);
        self.pool.insert(arc.clone());
        arc
    }

    pub fn lookup(&self, s: &str) -> Option<Arc<str>> {
        self.pool.get(s).map(|v| v.clone())
    }

    pub fn len(&self) -> usize {
        self.pool.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }

    pub fn clear(&self) {
        self.pool.clear();
    }
}

static GLOBAL_INTERNER: std::sync::LazyLock<Interner> =
    std::sync::LazyLock::new(|| Interner::with_capacity(4096));

pub fn global_interner() -> &'static Interner {
    &GLOBAL_INTERNER
}

pub fn intern(s: &str) -> Arc<str> {
    global_interner().intern(s)
}
