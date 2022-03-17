cfg_if! {
    if #[cfg(feature = "platform-qemu-virt-arm")] {
        mod qemu_virt_arm;
        pub use qemu_virt_arm::*;
    }
}
