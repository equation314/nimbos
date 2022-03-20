use core::arch::asm;

use x86::controlregs::{cr3, cr3_write};
use x86::msr::{wrmsr, IA32_GS_BASE};
use x86_64::registers::rflags::{self, RFlags};

#[inline]
pub fn enable_irqs() {
    unsafe { asm!("sti") };
}

#[inline]
pub fn disable_irqs() {
    unsafe { asm!("cli") };
}

#[inline]
pub fn irqs_disabled() -> bool {
    !rflags::read().contains(RFlags::INTERRUPT_FLAG)
}

pub fn thread_pointer() -> usize {
    // read PerCpu::self_vaddr
    let ret;
    unsafe { core::arch::asm!("mov {0}, gs:0", out(reg) ret, options(pure, readonly, nostack)) };
    ret
}

pub unsafe fn set_thread_pointer(tp: usize) {
    wrmsr(IA32_GS_BASE, tp as _)
}

pub unsafe fn activate_paging(page_table_root: usize, _is_kernel: bool) {
    cr3_write(page_table_root as _)
}

pub fn flush_tlb_all() {
    unsafe { cr3_write(cr3()) }
}

pub fn flush_icache_all() {}

pub fn wait_for_ints() {
    if !irqs_disabled() {
        x86_64::instructions::hlt();
    }
}
