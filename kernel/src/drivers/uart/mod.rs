cfg_if! {
    if #[cfg(feature = "platform-pc")] {
        mod uart16550;
        use uart16550 as imp;
    } else if #[cfg(feature = "platform-qemu-virt-arm")] {
        mod pl011;
        use pl011 as imp;
    }
}

pub use self::imp::{console_getchar, console_putchar, init, init_early};
