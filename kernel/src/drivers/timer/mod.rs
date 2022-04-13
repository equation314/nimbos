cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        mod x86_lapic;
        use x86_lapic as imp;
    } else if #[cfg(target_arch = "aarch64")] {
        mod arm_generic_timer;
        use arm_generic_timer as imp;
    }
}

pub(super) use self::imp::init;
pub use self::imp::{current_time, current_time_nanos, set_oneshot_timer};
