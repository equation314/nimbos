pub const PHYS_MEMORY_START: usize = 0x4000_0000;
pub const PHYS_MEMORY_SIZE: usize = 0x800_0000; // 128M
pub const PHYS_MEMORY_END: usize = PHYS_MEMORY_START + PHYS_MEMORY_SIZE;

pub const MMIO_REGIONS: &[(usize, usize)] = &[
    (0x0900_0000, 0x1000),   // PL011 UART
    (0x0800_0000, 0x2_0000), // GICv2
];

pub const USER_ASPACE_BASE: usize = 0;
pub const USER_ASPACE_SIZE: usize = 0xffff_ffff_f000;
pub const KERNEL_ASPACE_BASE: usize = 0xffff_0000_0000_0000;
pub const KERNEL_ASPACE_SIZE: usize = 0x0000_ffff_ffff_f000;

pub const PHYS_VIRT_OFFSET: usize = 0xffff_0000_0000_0000;
