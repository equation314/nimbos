use alloc::boxed::Box;
use core::arch::asm;
use core::fmt::{Debug, Formatter, Result};

use x86::{segmentation::SegmentSelector, task, Ring};
use x86_64::addr::VirtAddr;
use x86_64::structures::gdt::{Descriptor, DescriptorFlags};
use x86_64::structures::{tss::TaskStateSegment, DescriptorTablePointer};

pub const KCODE32_SELECTOR: SegmentSelector = SegmentSelector::new(1, Ring::Ring0);
pub const KCODE64_SELECTOR: SegmentSelector = SegmentSelector::new(2, Ring::Ring0);
pub const KDATA_SELECTOR: SegmentSelector = SegmentSelector::new(3, Ring::Ring0);
pub const UCODE32_SELECTOR: SegmentSelector = SegmentSelector::new(4, Ring::Ring3);
pub const UDATA_SELECTOR: SegmentSelector = SegmentSelector::new(5, Ring::Ring3);
pub const UCODE64_SELECTOR: SegmentSelector = SegmentSelector::new(6, Ring::Ring3);
pub const TSS_SELECTOR: SegmentSelector = SegmentSelector::new(7, Ring::Ring0);

pub(super) struct TssStruct {
    inner: &'static mut TaskStateSegment,
}

impl TssStruct {
    pub fn alloc() -> Self {
        Self {
            inner: Box::leak(Box::new(TaskStateSegment::new())),
        }
    }

    pub fn kernel_stack_top(&self) -> u64 {
        self.inner.privilege_stack_table[0].as_u64()
    }

    pub fn set_kernel_stack_top(&mut self, rsp0: u64) {
        self.inner.privilege_stack_table[0] = VirtAddr::new(rsp0);
    }
}

pub(super) struct GdtStruct {
    table: &'static mut [u64],
}

#[allow(dead_code)]
impl GdtStruct {
    pub fn alloc() -> Self {
        Self {
            table: Box::leak(Box::new([0u64; 16])),
        }
    }

    pub fn init(&mut self, tss: &TssStruct) {
        // first 3 entries are the same as in multiboot.rs
        self.table[1] = DescriptorFlags::KERNEL_CODE32.bits(); // 0x00cf9b000000ffff
        self.table[2] = DescriptorFlags::KERNEL_CODE64.bits(); // 0x00af9b000000ffff
        self.table[3] = DescriptorFlags::KERNEL_DATA.bits(); // 0x00cf93000000ffff
        self.table[4] = DescriptorFlags::USER_CODE32.bits(); // 0x00cffb000000ffff
        self.table[5] = DescriptorFlags::USER_DATA.bits(); // 0x00cff3000000ffff
        self.table[6] = DescriptorFlags::USER_CODE64.bits(); // 0x00affb000000ffff
        let tss = unsafe { &*(tss.inner as *const _) }; // required static lifetime
        let tss_desc = Descriptor::tss_segment(tss);
        if let Descriptor::SystemSegment(low, high) = tss_desc {
            self.table[7] = low;
            self.table[8] = high;
        }
    }

    fn pointer(&self) -> DescriptorTablePointer {
        DescriptorTablePointer {
            base: VirtAddr::new(self.table.as_ptr() as u64),
            limit: (core::mem::size_of_val(self.table) - 1) as u16,
        }
    }

    pub fn load(&self) {
        unsafe {
            asm!("lgdt [{}]", in(reg) &self.pointer(), options(readonly, nostack, preserves_flags))
        }
    }

    pub fn load_tss(&mut self, selector: SegmentSelector) {
        unsafe { task::load_tr(selector) };
    }
}

impl Debug for GdtStruct {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("GdtStruct")
            .field("pointer", &self.pointer())
            .field("table", &self.table)
            .finish()
    }
}
