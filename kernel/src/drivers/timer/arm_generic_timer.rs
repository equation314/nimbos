//! ARM Generic Timer.

use cortex_a::registers::{CNTFRQ_EL0, CNTPCT_EL0, CNTP_CTL_EL0, CNTP_TVAL_EL0};
use tock_registers::interfaces::{Readable, Writeable};

use crate::config::TICKS_PER_SEC;
use crate::drivers::interrupt;
use crate::structs::TimeValue;
use crate::sync::LazyInit;

const PHYS_TIMER_IRQ_NUM: usize = 30;

const NANOS_PER_SEC: u64 = 1_000_000_000;

static CLOCK_FREQ: LazyInit<u64> = LazyInit::new();

pub fn current_time() -> TimeValue {
    let ns = CNTPCT_EL0.get() * NANOS_PER_SEC / *CLOCK_FREQ;
    TimeValue::from_nanos(ns)
}

fn set_next_trigger() {
    CNTP_TVAL_EL0.set(*CLOCK_FREQ / TICKS_PER_SEC);
}

pub fn init() {
    CLOCK_FREQ.init_by(CNTFRQ_EL0.get());
    CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET);
    set_next_trigger();
    interrupt::register_handler(PHYS_TIMER_IRQ_NUM, || {
        set_next_trigger();
        super::timer_tick()
    });
    interrupt::set_enable(PHYS_TIMER_IRQ_NUM, true);
}
