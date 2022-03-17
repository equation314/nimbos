cfg_if! {
    if #[cfg(feature = "platform-qemu-virt-arm")] {
        mod psci;
        use psci as imp;
    }
}

pub use self::imp::shutdown;
