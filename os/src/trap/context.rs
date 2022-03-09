use core::arch::asm;

use cortex_a::registers::SPSR_EL1;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TrapFrame {
    /// General-purpose registers (R0..R30).
    pub r: [u64; 31],
    /// User Stack Pointer (SP_EL0).
    pub usp: u64,
    /// Exception Link Register (ELR_EL1).
    pub elr: u64,
    /// Saved Process Status Register (SPSR_EL1).
    pub spsr: u64,
}

impl TrapFrame {
    pub fn new_user(entry: usize, ustack_top: usize) -> Self {
        Self {
            usp: ustack_top as _,
            elr: entry as _,
            spsr: (SPSR_EL1::M::EL0t
                + SPSR_EL1::D::Masked
                + SPSR_EL1::A::Masked
                + SPSR_EL1::I::Unmasked
                + SPSR_EL1::F::Masked)
                .value, // enable IRQ, mask others
            ..Default::default()
        }
    }

    pub fn new_fork(&self) -> Self {
        let mut tf = *self;
        tf.r[0] = 0; // for child process, fork returns 0
        tf
    }

    pub unsafe fn exec(&self, kstack_top: usize) -> ! {
        asm!("
            mov     sp, x1
            ldp     x30, x9, [x0, 30 * 8]
            ldp     x10, x11, [x0, 32 * 8]
            msr     sp_el0, x9
            msr     elr_el1, x10
            msr     spsr_el1, x11

            ldp     x28, x29, [x0, 28 * 8]
            ldp     x26, x27, [x0, 26 * 8]
            ldp     x24, x25, [x0, 24 * 8]
            ldp     x22, x23, [x0, 22 * 8]
            ldp     x20, x21, [x0, 20 * 8]
            ldp     x18, x19, [x0, 18 * 8]
            ldp     x16, x17, [x0, 16 * 8]
            ldp     x14, x15, [x0, 14 * 8]
            ldp     x12, x13, [x0, 12 * 8]
            ldp     x10, x11, [x0, 10 * 8]
            ldp     x8, x9, [x0, 8 * 8]
            ldp     x6, x7, [x0, 6 * 8]
            ldp     x4, x5, [x0, 4 * 8]
            ldp     x2, x3, [x0, 2 * 8]
            ldp     x0, x1, [x0]

            eret",
            in("x0") self,
            in("x1") kstack_top,
            options(noreturn),
        )
    }
}
