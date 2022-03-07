pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const USER_STACK_BASE: usize = 0x8000_0000_0000 - USER_STACK_SIZE;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;

pub const MEMORY_START: usize = 0x4000_0000;
pub const MEMORY_END: usize = MEMORY_START + 0x800_0000;

pub const MMIO_REGIONS: &[(usize, usize)] = &[
    (0x0900_0000, 0x1000),   // PL011 UART
    (0x0800_0000, 0x2_0000), // GICv2
];

pub const PHYS_VIRT_OFFSET: usize = 0xffff_0000_0000_0000;

pub const TICKS_PER_SEC: u64 = 100;
