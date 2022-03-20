use super::IrqHandlerResult;

pub const IRQ_COUNT: usize = 256;

pub fn set_enable(vector: usize, enable: bool) {}

pub fn handle_irq() -> IrqHandlerResult {
    IrqHandlerResult::NoReschedule
}

pub fn init() {}
