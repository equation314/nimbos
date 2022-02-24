//! PL011 UART

use core::ptr::{read_volatile, write_volatile};

const UART_FIFO_DR: usize = 0x0900_0000;
const UART_FIFO_FR: usize = 0x0900_0018;

pub fn console_putchar(c: u8) {
    unsafe {
        while read_volatile(UART_FIFO_FR as *const u32) & (1 << 5) != 0 {}
        write_volatile(UART_FIFO_DR as *mut u32, c as u32);
    }
}
