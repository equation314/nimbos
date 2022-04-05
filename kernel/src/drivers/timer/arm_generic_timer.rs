//! ARM Generic Timer.

use cortex_a::registers::{CNTFRQ_EL0, CNTPCT_EL0, CNTP_CTL_EL0, CNTP_TVAL_EL0};
use tock_registers::interfaces::{Readable, Writeable};

use super::NSEC_PER_SEC;
use crate::config::TICKS_PER_SEC;
use crate::drivers::interrupt::{self, IrqHandlerResult};
use crate::sync::LazyInit;

const PHYS_TIMER_IRQ_NUM: usize = 30;

static CLOCK_FREQ: LazyInit<u64> = LazyInit::new();

pub fn get_time_ns() -> u64 {
    CNTPCT_EL0.get() * NSEC_PER_SEC / *CLOCK_FREQ
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
        super::timer_tick();
        IrqHandlerResult::Reschedule
    });
    interrupt::set_enable(PHYS_TIMER_IRQ_NUM, true);
}
