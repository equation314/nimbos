#![allow(dead_code)]

pub const USER_ASPACE_BASE: usize = 0;
pub const USER_ASPACE_SIZE: usize = 0x8000_0000_0000;
pub const KERNEL_ASPACE_BASE: usize = 0xffff_ff00_0000_0000;
pub const KERNEL_ASPACE_SIZE: usize = 0x0000_0100_0000_0000;

pub const PHYS_VIRT_OFFSET: usize = 0xffff_ff00_0000_0000;
