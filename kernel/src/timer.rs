use core::sync::atomic::{AtomicU64, Ordering};

use crate::sync::{LazyInit, SpinNoIrqLock};
use crate::utils::timer_list::TimerList;

pub use crate::drivers::timer::{current_time, current_time_nanos, set_next_trigger};
pub use crate::utils::timer_list::TimeValue;

const PERIODIC_INTERVAL_NS: u64 = 1_000_000_000 / crate::config::TICKS_PER_SEC;

static NEXT_DEADLINE: AtomicU64 = AtomicU64::new(u64::MAX);
static NEXT_PERIODIC_DEADLINE: AtomicU64 = AtomicU64::new(0);

static TIMER_LIST: LazyInit<SpinNoIrqLock<TimerList>> = LazyInit::new();

fn update_deadline(deadline_ns: u64) {
    NEXT_DEADLINE.store(deadline_ns, Ordering::Release);
    set_next_trigger(deadline_ns);
}

pub fn init() {
    TIMER_LIST.init_by(SpinNoIrqLock::new(TimerList::new()));
    let deadline = current_time_nanos() + PERIODIC_INTERVAL_NS;
    NEXT_PERIODIC_DEADLINE.store(deadline, Ordering::Release);
    update_deadline(deadline);
}

pub fn set_timer(deadline: TimeValue, callback: impl FnOnce(TimeValue) + Send + Sync + 'static) {
    TIMER_LIST.lock().set(deadline, callback);
    let deadline_ns = deadline.as_nanos() as u64;
    if deadline_ns < NEXT_DEADLINE.load(Ordering::Acquire) {
        update_deadline(deadline_ns);
    }
}

pub fn handle_timer_irq() {
    assert!(crate::arch::instructions::irqs_disabled());

    let now_ns = current_time_nanos();
    let mut next_deadline = NEXT_PERIODIC_DEADLINE.load(Ordering::Acquire);

    if now_ns >= next_deadline {
        crate::task::timer_tick_periodic();
        NEXT_PERIODIC_DEADLINE.fetch_add(PERIODIC_INTERVAL_NS, Ordering::Release);
        next_deadline = NEXT_PERIODIC_DEADLINE.load(Ordering::Acquire);
    }

    let mut timers = TIMER_LIST.lock();
    while timers.expire_one(current_time()).is_some() {}

    if let Some(d) = timers.next_deadline() {
        next_deadline = next_deadline.min(d.as_nanos() as u64);
    }
    update_deadline(next_deadline);
}
