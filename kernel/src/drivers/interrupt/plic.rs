use crate::utils::irq_handler::IrqHandler;

pub fn handle_irq(_vector: usize) {}

pub fn register_handler(_vector: usize, _handler: IrqHandler) {}

pub fn set_enable(_vector: usize, _enable: bool) {}

pub fn init() {}
