pub const PHYS_MEMORY_START: usize = 0;
pub const PHYS_MEMORY_SIZE: usize = 0x800_0000; // 128M
pub const PHYS_MEMORY_END: usize = PHYS_MEMORY_START + PHYS_MEMORY_SIZE;

pub const MMIO_REGIONS: &[(usize, usize)] = &[];

pub const USER_ASPACE_BASE: usize = 0;
pub const USER_ASPACE_SIZE: usize = 0x7fff_ffff_f000;
pub const KERNEL_ASPACE_BASE: usize = 0xffff_ff80_0000_0000;
pub const KERNEL_ASPACE_SIZE: usize = 0x0000_007f_ffff_f000;

pub const PHYS_VIRT_OFFSET: usize = 0xffff_ff80_0000_0000;
