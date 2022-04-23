use riscv::register::{mtvec::TrapMode, stvec};

pub fn init() {
    unsafe { stvec::write(0, TrapMode::Direct) };
}
