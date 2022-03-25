cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        mod qemu_x86_reset;
        use qemu_x86_reset as imp;
    } else if #[cfg(target_arch = "aarch64")] {
        mod psci;
        use psci as imp;
    }
}

pub use self::imp::shutdown;
