//! Intel Local APIC and IO APIC.

#![allow(dead_code)]

use x2apic::lapic::{xapic_base, LocalApic, LocalApicBuilder};

use crate::mm::PhysAddr;
use crate::sync::{LazyInit, PerCpuData};
use crate::utils::irq_handler::{IrqHandler, IrqHandlerTable};

const APIC_TIMER_VECTOR: usize = 0xf0;
const APIC_SPURIOUS_VECTOR: usize = 0xf1;
const APIC_ERROR_VECTOR: usize = 0xf2;

const IRQ_COUNT: usize = 256;

static LOCAL_APIC: LazyInit<PerCpuData<LocalApic>> = LazyInit::new();
static HANDLERS: IrqHandlerTable<IRQ_COUNT> = IrqHandlerTable::new();

fn lapic_eoi() {
    unsafe { local_apic().end_of_interrupt() };
}

pub fn set_enable(_vector: usize, _enable: bool) {
    // TODO: implement IOAPIC
}

pub fn handle_irq(vector: usize) {
    HANDLERS.handle(vector);
    lapic_eoi();
}

pub fn register_handler(vector: usize, handler: IrqHandler) {
    HANDLERS.register_handler(vector, handler);
}

pub fn init() {
    super::i8259_pic::init();

    let base_vaddr = PhysAddr::new(unsafe { xapic_base() } as usize).into_kvaddr();
    let mut lapic = LocalApicBuilder::new()
        .timer_vector(APIC_TIMER_VECTOR)
        .error_vector(APIC_ERROR_VECTOR)
        .spurious_vector(APIC_SPURIOUS_VECTOR)
        .set_xapic_base(base_vaddr.as_usize() as u64)
        .build()
        .unwrap();
    unsafe { lapic.enable() };
    LOCAL_APIC.init_by(PerCpuData::new(lapic));

    super::register_handler(APIC_TIMER_VECTOR, crate::timer::handle_timer_irq);
}

pub fn local_apic() -> &'static mut LocalApic {
    unsafe { LOCAL_APIC.as_mut() }
}
