use cortex_a::registers::{MAIR_EL1, SCTLR_EL1, TCR_EL1};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use super::{MemFlags, PageTable};
use crate::arch;
use crate::config::{MEMORY_END, MMIO_REGIONS};
use crate::mm::{PhysAddr, VirtAddr, PAGE_SIZE};
use crate::sync::LazyInit;

static KERNEL_PAGE_TABLE: LazyInit<PageTable> = LazyInit::new();

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss();
    fn ebss();
    fn boot_stack();
    fn boot_stack_top();
    fn ekernel();
}

fn init_mmu() {
    // Device-nGnRE memory
    let attr0 = MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck;
    // Normal memory
    let attr1 = MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
        + MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc;
    MAIR_EL1.write(attr0 + attr1);
    assert_eq!(MAIR_EL1.get(), 0xff_04);

    // Enable TTBR0 and TTBR1 walks, page size = 4K, vaddr size = 48 bits, paddr size = 40 bits.
    let tcr_flags0 = TCR_EL1::EPD0::EnableTTBR0Walks
        + TCR_EL1::TG0::KiB_4
        + TCR_EL1::SH0::Inner
        + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::T0SZ.val(16);
    let tcr_flags1 = TCR_EL1::EPD1::EnableTTBR1Walks
        + TCR_EL1::TG1::KiB_4
        + TCR_EL1::SH1::Inner
        + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::T1SZ.val(16);
    TCR_EL1.write(TCR_EL1::IPS::Bits_40 + tcr_flags0 + tcr_flags1);

    // Flush TLB
    arch::flush_tlb_all();

    // Enable the MMU and turn on I-cache and D-cache
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);

    // Flush I-cache
    arch::flush_icache_all();
}

pub fn init_paging() {
    let mut pt = PageTable::new();
    let mut map_range = |start: usize, end: usize, flags: MemFlags, name: &str| {
        println!("mapping {}: [{:#x}, {:#x})", name, start, end);
        assert!(VirtAddr::new(start).is_aligned());
        assert!(VirtAddr::new(end).is_aligned());
        let mut vaddr = start;
        while vaddr < end {
            pt.map(VirtAddr::new(vaddr), PhysAddr::new(vaddr), flags);
            vaddr += PAGE_SIZE;
        }
    };

    // map kernel sections
    map_range(
        stext as usize,
        etext as usize,
        MemFlags::READ | MemFlags::EXECUTE,
        ".text",
    );
    map_range(
        srodata as usize,
        erodata as usize,
        MemFlags::READ,
        ".rodata",
    );
    map_range(
        sdata as usize,
        edata as usize,
        MemFlags::READ | MemFlags::WRITE,
        ".data",
    );
    map_range(
        sbss as usize,
        ebss as usize,
        MemFlags::READ | MemFlags::WRITE,
        ".bss",
    );
    map_range(
        boot_stack as usize,
        boot_stack_top as usize,
        MemFlags::READ | MemFlags::WRITE,
        "boot stack",
    );
    map_range(
        ekernel as usize,
        MEMORY_END as usize,
        MemFlags::READ | MemFlags::WRITE,
        "physical memory",
    );
    for (base, size) in MMIO_REGIONS {
        map_range(
            *base,
            *base + *size,
            MemFlags::READ | MemFlags::WRITE | MemFlags::DEVICE,
            "MMIO",
        );
    }

    let root = pt.root_paddr().as_usize();
    KERNEL_PAGE_TABLE.init_by(pt);

    // Set TTBR0_EL1
    unsafe { arch::activate_paging(root) };
    init_mmu();
}

#[allow(unused)]
pub fn remap_test() {
    let pt = &KERNEL_PAGE_TABLE;
    let mid_text = VirtAddr::new((stext as usize + etext as usize) / 2);
    let mid_rodata = VirtAddr::new((srodata as usize + erodata as usize) / 2);
    let mid_data = VirtAddr::new((sdata as usize + edata as usize) / 2);
    let mid_mmio = VirtAddr::new(MMIO_REGIONS[0].0);
    assert!(!pt.query(mid_text).unwrap().1.contains(MemFlags::WRITE));
    assert!(!pt.query(mid_rodata).unwrap().1.contains(MemFlags::EXECUTE));
    assert!(pt.query(mid_mmio).unwrap().1.contains(MemFlags::DEVICE));
    println!("remap_test passed!");
}
