pub const MEMORY_START: usize = 0x4000_0000;
pub const MEMORY_END: usize = MEMORY_START + 0x800_0000;

pub const MMIO_REGIONS: &[(usize, usize)] = &[
    (0x0900_0000, 0x1000),   // PL011 UART
    (0x0800_0000, 0x2_0000), // GICv2
];
