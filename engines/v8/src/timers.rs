#[cfg(feature = "native")]
use crate::ffi;

pub struct V8Timer(i32);

impl V8Timer {
    pub fn id(&self) -> i32 { self.0 }
}

#[cfg(feature = "native")]
fn schedule_timeout(ctx: *mut ffi::V8ContextHandle, cb: Box<dyn FnOnce()>, ms: u64) -> i32 {
    let boxed: Box<Box<dyn FnOnce()>> = Box::new(cb);
    let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;

    extern "C" fn trampoline(data: *mut std::ffi::c_void) {
        let cb: Box<Box<dyn FnOnce()>> = unsafe { Box::from_raw(data as *mut Box<dyn FnOnce()>) };
        cb();
    }

    unsafe { ffi::klyron_v8_timer_set_timeout(ctx, Some(trampoline), ptr, ms) }
}

#[cfg(feature = "native")]
fn schedule_interval(ctx: *mut ffi::V8ContextHandle, cb: Box<dyn Fn()>, ms: u64) -> i32 {
    let boxed: Box<Box<dyn Fn()>> = Box::new(cb);
    let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;

    extern "C" fn trampoline(data: *mut std::ffi::c_void) {
        let cb: &Box<dyn Fn()> = unsafe { &*(data as *const Box<dyn Fn()>) };
        cb();
    }

    unsafe { ffi::klyron_v8_timer_set_interval(ctx, Some(trampoline), ptr, ms) }
}

pub fn set_timeout<F>(ctx: *mut std::ffi::c_void, cb: F, ms: u64) -> V8Timer
where F: FnOnce() + 'static
{
    #[cfg(feature = "native")]
    { let id = schedule_timeout(ctx as *mut ffi::V8ContextHandle, Box::new(cb), ms); return V8Timer(id); }
    #[cfg(not(feature = "native"))]
    { let _ = (ctx, cb, ms); V8Timer(0) }
}

pub fn set_interval<F>(ctx: *mut std::ffi::c_void, cb: F, ms: u64) -> V8Timer
where F: Fn() + 'static
{
    #[cfg(feature = "native")]
    { let id = schedule_interval(ctx as *mut ffi::V8ContextHandle, Box::new(cb), ms); return V8Timer(id); }
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
