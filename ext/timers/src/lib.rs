use std::sync::atomic::AtomicU64;

use deno_core::{extension, op2, Extension};

static NEXT_TIMER_ID: AtomicU64 = AtomicU64::new(1);

extension!(
  klyron_timers,
  ops = [op_set_timeout, op_clear_timer],
  esm_entry_point = "ext:klyron_timers/timers.js",
  esm = [dir "js", "timers.js"],
);

pub fn init() -> Extension {
  klyron_timers::init()
}

#[op2]
#[serde]
fn op_set_timeout(#[number] delay: u64) -> u64 {
  let _ = delay;
  NEXT_TIMER_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[op2(fast)]
fn op_clear_timer() {
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
