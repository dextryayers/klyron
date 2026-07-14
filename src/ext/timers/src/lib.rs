use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

use deno_core::{extension, op2, Extension};

static NEXT_TIMER_ID: AtomicU64 = AtomicU64::new(1);

fn with_timers<F: FnOnce(&mut HashMap<u64, tokio::task::JoinHandle<()>>)>(f: F) {
    static TIMERS: Mutex<Option<HashMap<u64, tokio::task::JoinHandle<()>>>> = Mutex::new(None);
    let mut guard = TIMERS.lock().unwrap();
    f(guard.get_or_insert_with(HashMap::new));
}

extension!(
  klyron_timers,
  ops = [op_set_timeout, op_set_interval, op_clear_timer],
  esm_entry_point = "ext:klyron_timers/timers.js",
  esm = [dir "js", "timers.js"],
);

pub fn init() -> Extension {
  klyron_timers::init()
}

#[op2(fast)]
fn op_set_timeout(delay: f64) -> f64 {
    let id = NEXT_TIMER_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as f64;
    let delay_ms = delay.max(0.0) as u64;
    let rt = tokio::runtime::Handle::current();
    let join = rt.spawn(async move {
        if delay_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        }
    });
    with_timers(|timers| { timers.insert(id as u64, join); });
    id
}

#[op2(fast)]
fn op_set_interval(delay: f64) -> f64 {
    let id = NEXT_TIMER_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as f64;
    let delay_ms = delay.max(1.0) as u64;
    let rt = tokio::runtime::Handle::current();
    let join = rt.spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(delay_ms));
        interval.tick().await;
        loop {
            interval.tick().await;
        }
    });
    with_timers(|timers| { timers.insert(id as u64, join); });
    id
}

#[op2(fast)]
fn op_clear_timer(id: f64) {
    with_timers(|timers| {
        if let Some(handle) = timers.remove(&(id as u64)) {
            handle.abort();
        }
    });
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_init_returns_extension() {
    let ext = init();
    assert_eq!(ext.name, "klyron_timers");
  }
}
