cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        mod apic;
        use apic as imp;
        pub use apic::init_local_apic_ap;
    } else if #[cfg(target_arch = "aarch64")] {
        mod gicv2;
        use gicv2 as imp;
    }
}

pub use self::imp::{handle_irq, init, send_ipi, set_enable, IRQ_COUNT};

use core::cell::UnsafeCell;

#[derive(Debug, Eq, PartialEq)]
pub enum IrqHandlerResult {
    Reschedule,
    NoReschedule,
}

pub type IrqHandler = fn() -> IrqHandlerResult;

struct IrqHandlerTable<const IRQ_COUNT: usize> {
    handlers: [UnsafeCell<Option<IrqHandler>>; IRQ_COUNT],
}

unsafe impl<const IRQ_COUNT: usize> Sync for IrqHandlerTable<IRQ_COUNT> {}

impl<const IRQ_COUNT: usize> IrqHandlerTable<IRQ_COUNT> {
    #[allow(clippy::declare_interior_mutable_const)]
    pub const fn new() -> Self {
        const EMPTY: UnsafeCell<Option<IrqHandler>> = UnsafeCell::new(None);
        Self {
            handlers: [EMPTY; IRQ_COUNT],
        }
    }

    pub fn register_handler(&self, vector: usize, handler: IrqHandler) {
        unsafe { *self.handlers[vector].get() = Some(handler) };
    }

    pub fn handle(&self, vector: usize) -> IrqHandlerResult {
        trace!("IRQ {}", vector);
        if let Some(handler) = unsafe { &*self.handlers[vector].get() } {
            handler()
        } else {
            IrqHandlerResult::NoReschedule
        }
    }
}

static HANDLERS: IrqHandlerTable<IRQ_COUNT> = IrqHandlerTable::new();

pub fn register_handler(vector: usize, handler: IrqHandler) {
    HANDLERS.register_handler(vector, handler)
}
