cfg_if! {
    if #[cfg(feature = "platform-pc")] {
        mod pc;
        pub use self::pc::*;
    } else if #[cfg(feature = "platform-qemu-virt-arm")] {
        mod qemu_virt_arm;
        pub use self::qemu_virt_arm::*;
    }
}
