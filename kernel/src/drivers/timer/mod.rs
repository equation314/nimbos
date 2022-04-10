use super::interrupt::IrqHandlerResult;

cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        mod x86_tsc;
        use x86_tsc as imp;
    } else if #[cfg(target_arch = "aarch64")] {
        mod arm_generic_timer;
        use arm_generic_timer as imp;
    }
}

pub fn timer_tick() -> IrqHandlerResult {
    assert!(crate::arch::instructions::irqs_disabled());
    crate::task::timer_tick()
}

pub use self::imp::{current_time, init};
