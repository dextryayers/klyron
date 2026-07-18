pub mod cache;
pub mod table;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_interner_is_empty() {
        let i = Interner::new();
        assert!(i.is_empty());
        assert_eq!(i.len(), 0);
    }

    #[test]
    fn test_intern_returns_same_arc_for_same_string() {
        let i = Interner::new();
        let a = i.intern("hello");
        let b = i.intern("hello");
        assert!(Arc::ptr_eq(&a, &b));
        assert_eq!(&*a, "hello");
    }

    #[test]
    fn test_intern_distinct_strings() {
        let i = Interner::new();
        let a = i.intern("foo");
        let b = i.intern("bar");
        assert!(!Arc::ptr_eq(&a, &b));
        assert_eq!(i.len(), 2);
    }

    #[test]
    fn test_lookup_found() {
        let i = Interner::new();
        i.intern("key");
        let v = i.lookup("key");
        assert!(v.is_some());
        assert_eq!(&*v.unwrap(), "key");
    }

    #[test]
    fn test_lookup_missing() {
        let i = Interner::new();
        assert!(i.lookup("nonexistent").is_none());
    }

    #[test]
    fn test_clear() {
        let i = Interner::new();
        i.intern("a");
        i.intern("b");
        assert_eq!(i.len(), 2);
        i.clear();
        assert!(i.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let i = Interner::with_capacity(100);
        assert!(i.is_empty());
        i.intern("test");
        assert_eq!(i.len(), 1);
    }

    #[test]
    fn test_intern_empty_string() {
        let i = Interner::new();
        let a = i.intern("");
        let b = i.intern("");
        assert!(Arc::ptr_eq(&a, &b));
        assert_eq!(&*a, "");
    }

    #[test]
    fn test_intern_long_string() {
        let i = Interner::new();
        let long = "a".repeat(10_000);
        let a = i.intern(&long);
        let b = i.intern(&long);
        assert!(Arc::ptr_eq(&a, &b));
        assert_eq!(a.len(), 10_000);
    }

    #[test]
    fn test_global_interner() {
        let gi = global_interner();
        let a = gi.intern("global");
        let b = intern("global");
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn test_global_intern_same_string() {
        let a = intern("duplicate");
        let b = intern("duplicate");
        assert!(Arc::ptr_eq(&a, &b));
    }
}
