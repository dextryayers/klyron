use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::Instant;

enum TimerKind {
    Timeout,
    Interval,
}

struct TimerEntry {
    kind: TimerKind,
    delay_ms: u64,
    last_fired: Instant,
}

pub struct JSCTimers {
    next_id: AtomicU32,
    timers: Mutex<HashMap<u32, TimerEntry>>,
    timeouts: Mutex<HashMap<u32, Box<dyn FnOnce() + Send>>>,
    intervals: Mutex<HashMap<u32, Box<dyn FnMut() + Send>>>,
}

impl JSCTimers {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU32::new(1),
            timers: Mutex::new(HashMap::new()),
            timeouts: Mutex::new(HashMap::new()),
            intervals: Mutex::new(HashMap::new()),
        }
    }

    pub fn next_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn set_timeout(&self, callback: Box<dyn FnOnce() + Send>, delay_ms: u64) -> u32 {
        let id = self.next_id();
        self.timers.lock().unwrap().insert(id, TimerEntry {
            kind: TimerKind::Timeout,
            delay_ms,
            last_fired: Instant::now(),
        });
        self.timeouts.lock().unwrap().insert(id, callback);
        id
    }

    pub fn set_interval(&self, callback: Box<dyn FnMut() + Send>, interval_ms: u64) -> u32 {
        let id = self.next_id();
        self.timers.lock().unwrap().insert(id, TimerEntry {
            kind: TimerKind::Interval,
            delay_ms: interval_ms,
            last_fired: Instant::now(),
        });
        self.intervals.lock().unwrap().insert(id, callback);
        id
    }

    pub fn set_immediate(&self, callback: Box<dyn FnOnce() + Send>) -> u32 {
        self.set_timeout(callback, 0)
    }

    pub fn clear(&self, id: u32) {
        self.timers.lock().unwrap().remove(&id);
        self.timeouts.lock().unwrap().remove(&id);
        self.intervals.lock().unwrap().remove(&id);
    }

    pub fn poll(&self) {
        let ready: Vec<(u32, bool)> = {
            let mut timers = self.timers.lock().unwrap();
            let now = Instant::now();
            let mut ids = Vec::new();
            let mut to_remove = Vec::new();
            for (&id, entry) in timers.iter_mut() {
                let elapsed = now.duration_since(entry.last_fired).as_millis() as u64;
                if elapsed >= entry.delay_ms {
                    entry.last_fired = now;
                    let is_interval = matches!(entry.kind, TimerKind::Interval);
                    ids.push((id, is_interval));
                    if !is_interval {
                        to_remove.push(id);
                    }
                }
            }
            for id in to_remove {
                timers.remove(&id);
            }
            ids
        };

        for (id, is_interval) in ready {
            if is_interval {
                let mut cb = self.intervals.lock().unwrap().remove(&id);
                if let Some(ref mut f) = cb {
                    f();
                }
                if cb.is_some() {
                    self.intervals.lock().unwrap().insert(id, cb.unwrap());
                }
            } else {
                if let Some(f) = self.timeouts.lock().unwrap().remove(&id) {
                    f();
                }
            }
        }
    }
}

impl Default for JSCTimers {
    fn default() -> Self {
        Self::new()
    }
}
