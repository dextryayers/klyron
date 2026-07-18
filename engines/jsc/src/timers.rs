pub struct JSCTimers {
    ids: std::sync::atomic::AtomicU32,
}

impl JSCTimers {
    pub fn new() -> Self {
        Self {
            ids: std::sync::atomic::AtomicU32::new(1),
        }
    }

    pub fn next_id(&self) -> u32 {
        self.ids.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn set_timeout(&self, _callback: Box<dyn FnOnce()>, _delay_ms: u64) -> u32 {
        let id = self.next_id();
        id
    }

    pub fn set_interval(&self, _callback: Box<dyn FnMut()>, _interval_ms: u64) -> u32 {
        let id = self.next_id();
        id
    }

    pub fn set_immediate(&self, callback: Box<dyn FnOnce()>) -> u32 {
        self.set_timeout(callback, 0)
    }

    pub fn clear(&self, _id: u32) {}
}

impl Default for JSCTimers {
    fn default() -> Self {
        Self::new()
    }
}
