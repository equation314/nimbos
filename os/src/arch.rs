use core::arch::asm;

use cortex_a::registers::{DAIF, TPIDR_EL1};
use tock_registers::interfaces::{Readable, Writeable};

pub fn enable_irqs() {
    unsafe { asm!("msr daifclr, #2") };
}

pub fn disable_irqs() {
    unsafe { asm!("msr daifset, #2") };
}

pub fn irqs_disabled() -> bool {
    DAIF.matches_all(DAIF::I::Masked)
}

pub fn thread_pointer() -> usize {
    TPIDR_EL1.get() as _
}

pub fn set_thread_pointer(tp: usize) {
    TPIDR_EL1.set(tp as _)
}

pub fn flush_icache() {
    unsafe { asm!("ic iallu; dsb sy; isb") };
}
