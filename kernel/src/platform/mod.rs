cfg_if! {
    if #[cfg(any(feature = "platform-pc", feature = "platform-pc-rvm"))] {
        mod pc;
        pub use self::pc::*;
    } else if #[cfg(feature = "platform-qemu-virt-arm")] {
        mod qemu_virt_arm;
        pub use self::qemu_virt_arm::*;
    }
}

pub mod config;
