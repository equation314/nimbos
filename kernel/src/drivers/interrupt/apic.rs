//! Intel Local APIC and IO APIC.

#![allow(dead_code)]

use x2apic::lapic::{xapic_base, LocalApic, LocalApicBuilder, TimerDivide, TimerMode};

use super::IrqHandlerResult;
use crate::config::TICKS_PER_SEC;
use crate::mm::PhysAddr;
use crate::percpu::PerCpuData;
use crate::sync::LazyInit;

const APIC_TIMER_VECTOR: usize = 0xf0;
const APIC_SPURIOUS_VECTOR: usize = 0xf1;
const APIC_ERROR_VECTOR: usize = 0xf2;

pub const IRQ_COUNT: usize = 256;

static LOCAL_APIC: LazyInit<PerCpuData<LocalApic>> = LazyInit::new();

fn lapic_eoi() {
    unsafe { LOCAL_APIC.as_mut().end_of_interrupt() };
}

pub fn set_enable(_vector: usize, _enable: bool) {
    // TODO: implement IOAPIC
}

pub fn handle_irq(vector: usize) -> IrqHandlerResult {
    lapic_eoi();
    super::HANDLERS.handle(vector)
}

pub fn init() {
    let base_vaddr = PhysAddr::new(unsafe { xapic_base() } as usize).into_kvaddr();
    let mut lapic = LocalApicBuilder::new()
        .timer_vector(APIC_TIMER_VECTOR)
        .error_vector(APIC_ERROR_VECTOR)
        .spurious_vector(APIC_SPURIOUS_VECTOR)
        .timer_mode(TimerMode::Periodic)
        .timer_divide(TimerDivide::Div256) // divide by 1
        .timer_initial((1_000_000_000 / TICKS_PER_SEC) as u32) // FIXME: need to calibrate
        .set_xapic_base(base_vaddr.as_usize() as u64)
        .build()
        .unwrap();
    unsafe { lapic.enable() };
    LOCAL_APIC.init_by(PerCpuData::new(lapic));
    super::register_handler(APIC_TIMER_VECTOR, || IrqHandlerResult::Reschedule);
}

pub fn init_local_apic_ap() {
    unsafe { LOCAL_APIC.as_mut().enable() };
}
