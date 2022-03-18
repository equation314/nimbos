#![allow(dead_code)]

pub const USER_ASPACE_BASE: usize = 0;
pub const USER_ASPACE_SIZE: usize = 0x1_0000_0000_0000;
pub const KERNEL_ASPACE_BASE: usize = 0xffff_0000_0000_0000;
pub const KERNEL_ASPACE_SIZE: usize = 0x0001_0000_0000_0000;

pub const PHYS_VIRT_OFFSET: usize = 0xffff_0000_0000_0000;
