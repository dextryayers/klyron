#[cfg(feature = "native")]
use crate::ffi;

#[allow(dead_code)]
static mut NEXT_TIMER_ID: i32 = 1;

#[allow(dead_code)]
fn next_id() -> i32 {
    unsafe {
        let id = NEXT_TIMER_ID;
        NEXT_TIMER_ID += 1;
        id
    }
}

pub struct V8Timer(i32);

impl V8Timer {
    pub fn id(&self) -> i32 { self.0 }
}

#[cfg(feature = "native")]
fn schedule(ctx: *mut ffi::V8ContextHandle, cb: Box<dyn FnOnce()>, ms: u64, repeat: bool) -> i32 {
    let id = next_id();
    let boxed: Box<Box<dyn FnOnce()>> = Box::new(cb);
    let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;

    extern "C" fn trampoline(data: *mut std::ffi::c_void) {
        let cb: Box<Box<dyn FnOnce()>> = unsafe { Box::from_raw(data as *mut Box<dyn FnOnce()>) };
        cb();
    }

    if repeat {
        unsafe { ffi::klyron_v8_timer_set_interval(ctx, Some(trampoline), ptr, ms); }
    } else {
        unsafe { ffi::klyron_v8_timer_set_timeout(ctx, Some(trampoline), ptr, ms); }
    }
    id
}

pub fn set_timeout<F>(ctx: *mut std::ffi::c_void, cb: F, ms: u64) -> V8Timer
where F: FnOnce() + 'static
{
    #[cfg(feature = "native")]
    { let id = schedule(ctx as *mut ffi::V8ContextHandle, Box::new(cb), ms, false); return V8Timer(id); }
    #[cfg(not(feature = "native"))]
    { let _ = (ctx, cb, ms); V8Timer(0) }
}

pub fn set_interval<F>(ctx: *mut std::ffi::c_void, cb: F, ms: u64) -> V8Timer
where F: FnOnce() + 'static
{
    #[cfg(feature = "native")]
    { let id = schedule(ctx as *mut ffi::V8ContextHandle, Box::new(cb), ms, true); return V8Timer(id); }
    #[cfg(not(feature = "native"))]
    { let _ = (ctx, cb, ms); V8Timer(0) }
}

pub fn set_immediate<F>(ctx: *mut std::ffi::c_void, cb: F) -> V8Timer
where F: FnOnce() + 'static
{
    set_timeout(ctx, cb, 0)
}

pub fn clear_timer(timer: V8Timer) {
    #[cfg(feature = "native")]
    unsafe { ffi::klyron_v8_timer_clear(timer.id()) }
    #[cfg(not(feature = "native"))]
    { let _ = timer; }
}

pub fn clear_all_timers() {
    #[cfg(feature = "native")]
    unsafe { ffi::klyron_v8_timer_clear_all() }
}
