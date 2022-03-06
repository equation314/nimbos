use super::address::{phys_to_virt, virt_to_phys};
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

pub fn init_paging() {
    let mut pt = PageTable::new();
    let mut map_range = |start: usize, end: usize, flags: MemFlags, name: &str| {
        println!("mapping {}: [{:#x}, {:#x})", name, start, end);
        assert!(VirtAddr::new(start).is_aligned());
        assert!(VirtAddr::new(end).is_aligned());
        let mut vaddr = start;
        while vaddr < end {
            pt.map(
                VirtAddr::new(vaddr),
                PhysAddr::new(virt_to_phys(vaddr)),
                flags,
            );
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
        phys_to_virt(MEMORY_END),
        MemFlags::READ | MemFlags::WRITE,
        "physical memory",
    );
    for (base, size) in MMIO_REGIONS {
        map_range(
            phys_to_virt(*base),
            phys_to_virt(*base + *size),
            MemFlags::READ | MemFlags::WRITE | MemFlags::DEVICE,
            "MMIO",
        );
    }

    let root = pt.root_paddr().as_usize();
    KERNEL_PAGE_TABLE.init_by(pt);

    // Set TTBR0_EL1
    unsafe { arch::activate_paging(root) };
}

#[allow(unused)]
pub fn remap_test() {
    let pt = &KERNEL_PAGE_TABLE;
    let mid_text = VirtAddr::new(stext as usize + (etext as usize - stext as usize) / 2);
    let mid_rodata = VirtAddr::new(srodata as usize + (erodata as usize - srodata as usize) / 2);
    let mid_data = VirtAddr::new(sdata as usize + (edata as usize - sdata as usize) / 2);
    let mid_mmio = VirtAddr::new(phys_to_virt(MMIO_REGIONS[0].0));
    assert!(!pt.query(mid_text).unwrap().1.contains(MemFlags::WRITE));
    assert!(!pt.query(mid_rodata).unwrap().1.contains(MemFlags::EXECUTE));
    assert!(pt.query(mid_mmio).unwrap().1.contains(MemFlags::DEVICE));
    println!("remap_test passed!");
}
