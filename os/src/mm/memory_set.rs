use alloc::collections::btree_map::{BTreeMap, Entry};

use super::address::{phys_to_virt, virt_to_phys};
use super::{MemFlags, PageTable, PhysFrame};
use crate::arch;
use crate::config::{MEMORY_END, MMIO_REGIONS};
use crate::mm::{PhysAddr, VirtAddr};
use crate::sync::LazyInit;

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

static KERNEL_SPACE: LazyInit<MemorySet> = LazyInit::new();

enum Mapper {
    Offset(usize),
    Framed(BTreeMap<VirtAddr, PhysFrame>),
}

pub struct MapArea {
    pub start: VirtAddr,
    pub size: usize,
    pub flags: MemFlags,
    mapper: Mapper,
}

pub struct MemorySet {
    pt: PageTable,
    areas: BTreeMap<VirtAddr, MapArea>,
}

impl MapArea {
    pub fn new_offset(
        start_vaddr: VirtAddr,
        start_paddr: PhysAddr,
        size: usize,
        flags: MemFlags,
    ) -> Self {
        let start_vaddr = start_vaddr.align_down();
        let start_paddr = start_paddr.align_down();
        let offset = start_vaddr.as_usize() - start_paddr.as_usize();
        Self {
            start: start_vaddr,
            size,
            flags,
            mapper: Mapper::Offset(offset),
        }
    }

    pub fn new_framed(start_vaddr: VirtAddr, size: usize, flags: MemFlags) -> Self {
        Self {
            start: start_vaddr,
            size,
            flags,
            mapper: Mapper::Framed(BTreeMap::new()),
        }
    }

    pub fn map(&mut self, vaddr: VirtAddr) -> PhysAddr {
        match &mut self.mapper {
            Mapper::Offset(off) => PhysAddr::new(vaddr.as_usize() - *off),
            Mapper::Framed(frames) => match frames.entry(vaddr) {
                Entry::Occupied(e) => e.get().start_paddr(),
                Entry::Vacant(e) => e.insert(PhysFrame::alloc().unwrap()).start_paddr(),
            },
        }
    }

    pub fn unmap(&mut self, vaddr: VirtAddr) {
        if let Mapper::Framed(frames) = &mut self.mapper {
            frames.remove(&vaddr);
        }
    }
}

impl MemorySet {
    pub fn new() -> Self {
        Self {
            pt: PageTable::new(),
            areas: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, area: MapArea) {
        if !area.size > 0 {
            if let Entry::Vacant(e) = self.areas.entry(area.start) {
                self.pt.map_area(e.insert(area));
            } else {
                panic!(
                    "MemorySet::insert: MepArea starts from {:#x?} is existed!",
                    area.start
                );
            }
        }
    }

    pub fn clear(&mut self) {
        for area in self.areas.values_mut() {
            self.pt.unmap_area(area);
        }
        self.areas.clear();
    }

    pub unsafe fn activate(&self, is_kernel: bool) {
        let root = self.pt.root_paddr().as_usize();
        arch::activate_paging(root, is_kernel);
    }
}

impl Drop for MemorySet {
    fn drop(&mut self) {
        self.clear();
    }
}

pub fn init_paging() {
    let mut ms = MemorySet::new();
    let mut map_range = |start: usize, end: usize, flags: MemFlags, name: &str| {
        println!("mapping {}: [{:#x}, {:#x})", name, start, end);
        assert!(start < end);
        assert!(VirtAddr::new(start).is_aligned());
        assert!(VirtAddr::new(end).is_aligned());
        ms.insert(MapArea::new_offset(
            VirtAddr::new(start),
            PhysAddr::new(virt_to_phys(start)),
            end - start,
            flags,
        ));
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

    KERNEL_SPACE.init_by(ms);
    unsafe {
        KERNEL_SPACE.activate(true); // set TTBR1 to kernel page table
        arch::activate_paging(0, false); // set TTBR0 to zero for kernel tasks
    }
}

#[allow(unused)]
pub fn remap_test() {
    let pt = &KERNEL_SPACE.pt;
    let mid_text = VirtAddr::new(stext as usize + (etext as usize - stext as usize) / 2);
    let mid_rodata = VirtAddr::new(srodata as usize + (erodata as usize - srodata as usize) / 2);
    let mid_data = VirtAddr::new(sdata as usize + (edata as usize - sdata as usize) / 2);
    let mid_mmio = VirtAddr::new(phys_to_virt(MMIO_REGIONS[0].0));
    assert!(!pt.query(mid_text).unwrap().1.contains(MemFlags::WRITE));
    assert!(!pt.query(mid_rodata).unwrap().1.contains(MemFlags::EXECUTE));
    assert!(pt.query(mid_mmio).unwrap().1.contains(MemFlags::DEVICE));
    println!("remap_test passed!");
}
