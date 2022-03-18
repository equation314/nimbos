cfg_if! {
    if #[cfg(feature = "platform-pc")] {
        mod pl011;
        use pl011 as imp;
    } else if #[cfg(feature = "platform-qemu-virt-arm")] {
        mod pl011;
        use pl011 as imp;
    }
}

pub use self::imp::{console_getchar, console_putchar, init};
