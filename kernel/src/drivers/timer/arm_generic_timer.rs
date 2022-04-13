//! ARM Generic Timer.

use cortex_a::registers::{CNTFRQ_EL0, CNTPCT_EL0, CNTP_CTL_EL0, CNTP_TVAL_EL0};
use tock_registers::interfaces::{Readable, Writeable};

use crate::drivers::interrupt;
use crate::sync::LazyInit;
use crate::timer::TimeValue;

const PHYS_TIMER_IRQ_NUM: usize = 30;

const NANOS_PER_SEC: u64 = 1_000_000_000;

static CLOCK_FREQ: LazyInit<u64> = LazyInit::new();

fn ticks_to_nanos(ticks: u64) -> u64 {
    ticks * NANOS_PER_SEC / *CLOCK_FREQ
}

fn nanos_to_ticks(nanos: u64) -> u64 {
    nanos * *CLOCK_FREQ / NANOS_PER_SEC
}

pub fn current_time_nanos() -> u64 {
    ticks_to_nanos(CNTPCT_EL0.get())
}

pub fn current_time() -> TimeValue {
    TimeValue::from_nanos(current_time_nanos())
}

pub fn set_oneshot_timer(deadline_ns: u64) {
    let cnptct = CNTPCT_EL0.get();
    let cnptct_deadline = nanos_to_ticks(deadline_ns);
    if cnptct < cnptct_deadline {
        let interval = cnptct_deadline - cnptct;
        debug_assert!(interval <= u32::MAX as u64);
        CNTP_TVAL_EL0.set(interval);
    } else {
        CNTP_TVAL_EL0.set(0);
    }
}

pub fn init() {
    CLOCK_FREQ.init_by(CNTFRQ_EL0.get());
    CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET);
    interrupt::register_handler(PHYS_TIMER_IRQ_NUM, crate::timer::handle_timer_irq);
    interrupt::set_enable(PHYS_TIMER_IRQ_NUM, true);
}
