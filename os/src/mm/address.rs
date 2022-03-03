#![allow(dead_code)]

use core::fmt;

use super::PAGE_SIZE;

const PA_16TB_BITS: usize = 44;
const VA_MAX_BITS: usize = 48;
const TOP_VA_BASE: usize = 0xffff_0000_0000_0000;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(usize);

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}

impl PhysAddr {
    pub const fn new(pa: usize) -> Self {
        Self(pa & ((1 << PA_16TB_BITS) - 1))
    }
    pub const fn as_usize(&self) -> usize {
        self.0
    }
    pub const fn align_down(&self) -> Self {
        Self(align_down(self.0, PAGE_SIZE))
    }
    pub const fn align_up(&self) -> Self {
        Self(align_up(self.0, PAGE_SIZE))
    }
    pub const fn page_offset(&self) -> usize {
        page_offset(self.0, PAGE_SIZE)
    }
    pub const fn is_aligned(&self) -> bool {
        is_aligned(self.0, PAGE_SIZE)
    }
}

impl VirtAddr {
    pub const fn new(va: usize) -> Self {
        let top_bits = va >> VA_MAX_BITS;
        if top_bits == 0 {
            Self(va & ((1 << VA_MAX_BITS) - 1))
        } else if top_bits == 0xfff {
            Self(TOP_VA_BASE + (va & ((1 << VA_MAX_BITS) - 1)))
        } else {
            panic!("invalid VA!")
        }
    }
    pub const fn as_usize(&self) -> usize {
        self.0
    }
    pub const fn align_down(&self) -> Self {
        Self(align_down(self.0, PAGE_SIZE))
    }
    pub const fn align_up(&self) -> Self {
        Self(align_up(self.0, PAGE_SIZE))
    }
    pub const fn page_offset(&self) -> usize {
        page_offset(self.0, PAGE_SIZE)
    }
    pub const fn is_aligned(&self) -> bool {
        is_aligned(self.0, PAGE_SIZE)
    }
}

const fn align_down(addr: usize, page_size: usize) -> usize {
    addr & !(page_size - 1)
}

const fn align_up(addr: usize, page_size: usize) -> usize {
    (addr + page_size - 1) & !(page_size - 1)
}

const fn page_offset(addr: usize, page_size: usize) -> usize {
    addr & (page_size - 1)
}

const fn is_aligned(addr: usize, page_size: usize) -> bool {
    page_offset(addr, page_size) == 0
}
