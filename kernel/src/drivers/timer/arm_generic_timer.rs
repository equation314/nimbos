//! ARM Generic Timer.

use cortex_a::registers::{CNTFRQ_EL0, CNTPCT_EL0, CNTP_CTL_EL0, CNTP_TVAL_EL0};
use tock_registers::interfaces::{Readable, Writeable};

use crate::drivers::interrupt;
use crate::sync::LazyInit;

const PHYS_TIMER_IRQ_NUM: usize = 30;

static CLOCK_FREQ_HZ: LazyInit<u64> = LazyInit::new();

pub fn current_ticks() -> u64 {
    CNTPCT_EL0.get()
}

pub fn frequency_hz() -> u64 {
    *CLOCK_FREQ_HZ
}

pub fn set_oneshot_timer(deadline_ns: u64) {
    let cnptct = CNTPCT_EL0.get();
    let cnptct_deadline = crate::timer::nanos_to_ticks(deadline_ns, frequency_hz());
    if cnptct < cnptct_deadline {
        let interval = cnptct_deadline - cnptct;
        debug_assert!(interval <= u32::MAX as u64);
        CNTP_TVAL_EL0.set(interval);
    } else {
        CNTP_TVAL_EL0.set(0);
    }
}

pub fn init() {
    CLOCK_FREQ_HZ.init_by(CNTFRQ_EL0.get());
    CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET);
    interrupt::register_handler(PHYS_TIMER_IRQ_NUM, crate::timer::handle_timer_irq);
    interrupt::set_enable(PHYS_TIMER_IRQ_NUM, true);
}
