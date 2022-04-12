cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        mod apic;
        mod i8259_pic;
        use apic as imp;
        pub use apic::local_apic;
    } else if #[cfg(target_arch = "aarch64")] {
        mod gicv2;
        use gicv2 as imp;
    }
}

pub use self::imp::handle_irq;

#[allow(unused_imports)]
pub(super) use self::imp::{init, register_handler, set_enable};
