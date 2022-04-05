pub const MSEC_PER_SEC: u64 = 1000;
pub const USEC_PER_SEC: u64 = MSEC_PER_SEC * 1000;
pub const NSEC_PER_SEC: u64 = USEC_PER_SEC * 1000;

cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        mod x86_tsc;
        use x86_tsc as imp;
    } else if #[cfg(target_arch = "aarch64")] {
        mod arm_generic_timer;
        use arm_generic_timer as imp;
    }
}

pub use self::imp::{get_time_ns, init};

use crate::sync::SpinNoIrqLock;
use alloc::vec::Vec;

type TimerCallback = fn();

static TIMER_EVENTS: SpinNoIrqLock<Vec<TimerCallback>> = SpinNoIrqLock::new(Vec::new());

pub fn timer_tick() {
    assert!(crate::arch::instructions::irqs_disabled());
    for callback in TIMER_EVENTS.lock().iter() {
        callback();
    }
}

pub fn add_timer_event(callback: TimerCallback) {
    TIMER_EVENTS.lock().push(callback);
}
