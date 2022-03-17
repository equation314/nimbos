cfg_if! {
    if #[cfg(feature = "platform-qemu-virt-arm")] {
        mod gicv2;
        use gicv2 as imp;
    }
}

pub use self::imp::{handle_irq, init, set_enable};
