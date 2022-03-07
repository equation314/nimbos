//! PL011 UART.

use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite};

use crate::mm::{PhysAddr, VirtAddr};

const UART_BASE: PhysAddr = PhysAddr::new(0x0900_0000);

static UART: Pl011Uart = Pl011Uart::new(UART_BASE.into_kvaddr());

register_structs! {
    Pl011UartRegs {
        /// Data Register.
        (0x00 => dr: ReadWrite<u32>),
        (0x04 => _reserved0),
        /// Flag Register.
        (0x18 => fr: ReadOnly<u32>),
        (0x1c => @END),
    }
}

struct Pl011Uart {
    base_vaddr: VirtAddr,
}

impl Pl011Uart {
    const fn new(base_vaddr: VirtAddr) -> Self {
        Self { base_vaddr }
    }

    const fn regs(&self) -> &Pl011UartRegs {
        unsafe { &*(self.base_vaddr.as_ptr() as *const _) }
    }

    fn putchar(&self, c: u8) {
        while self.regs().fr.get() & (1 << 5) != 0 {}
        self.regs().dr.set(c as u32);
    }
}

pub fn console_putchar(c: u8) {
    UART.putchar(c);
}
