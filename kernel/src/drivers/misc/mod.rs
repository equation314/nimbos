cfg_if! {
    if #[cfg(feature = "platform-pc")] {
        mod qemu_x86_reset;
        use qemu_x86_reset as imp;
    } else if #[cfg(feature = "platform-qemu-virt-arm")] {
        mod psci;
        use psci as imp;
    }
}

pub use self::imp::shutdown;
