use crate::sync::{LazyInit, SpinNoIrqLock};
use crate::utils::timer_list::TimerList;

pub use crate::drivers::timer::current_time;
pub use crate::utils::timer_list::TimeValue;

static TIMER_LIST: LazyInit<SpinNoIrqLock<TimerList>> = LazyInit::new();

pub fn init() {
    TIMER_LIST.init_by(SpinNoIrqLock::new(TimerList::new()));
}

pub fn set_timer(deadline: TimeValue, callback: impl FnOnce(TimeValue) + Send + Sync + 'static) {
    TIMER_LIST.lock().set(deadline, callback);
}

pub fn handle_timer_irq() {
    assert!(crate::arch::instructions::irqs_disabled());
    crate::task::timer_tick_periodic();
    let mut timers = TIMER_LIST.lock();
    while timers.expire_one(current_time()).is_some() {}
    let _next_deadline = timers.next_deadline();
}
