use alloc::collections::btree_map::{BTreeMap, Entry};
use core::fmt;

use super::address::{align_down, is_aligned, phys_to_virt, virt_to_phys};
use super::{MemFlags, PageTable, PhysFrame, PAGE_SIZE};
use crate::arch;
use crate::config::{MEMORY_END, MMIO_REGIONS, USER_STACK_BASE, USER_STACK_SIZE};
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
        assert!(start_vaddr.is_aligned());
        assert!(start_paddr.is_aligned());
        assert!(is_aligned(size, PAGE_SIZE));
        let offset = start_vaddr.as_usize() - start_paddr.as_usize();
        Self {
            start: start_vaddr,
            size,
            flags,
            mapper: Mapper::Offset(offset),
        }
    }

    pub fn new_framed(start_vaddr: VirtAddr, size: usize, flags: MemFlags) -> Self {
        assert!(start_vaddr.is_aligned());
        assert!(is_aligned(size, PAGE_SIZE));
        Self {
            start: start_vaddr,
            size,
            flags,
            mapper: Mapper::Framed(BTreeMap::new()),
        }
    }

    pub fn dup(&self) -> Self {
        let mapper = match &self.mapper {
            Mapper::Offset(off) => Mapper::Offset(*off),
            Mapper::Framed(orig_frames) => {
                let mut new_frames = BTreeMap::new();
                for (&vaddr, orig_frame) in orig_frames {
                    let mut new_frame = PhysFrame::alloc().unwrap();
                    new_frame
                        .as_slice_mut()
                        .copy_from_slice(orig_frame.as_slice());
                    new_frames.insert(vaddr, new_frame);
                }
                Mapper::Framed(new_frames)
            }
        };
        Self {
            start: self.start,
            size: self.size,
            flags: self.flags,
            mapper,
        }
    }

    pub fn map(&mut self, vaddr: VirtAddr) -> PhysAddr {
        assert!(vaddr.is_aligned());
        match &mut self.mapper {
            Mapper::Offset(off) => PhysAddr::new(vaddr.as_usize() - *off),
            Mapper::Framed(frames) => match frames.entry(vaddr) {
                Entry::Occupied(e) => e.get().start_paddr(),
                Entry::Vacant(e) => e.insert(PhysFrame::alloc_zero().unwrap()).start_paddr(),
            },
        }
    }

    pub fn unmap(&mut self, vaddr: VirtAddr) {
        if let Mapper::Framed(frames) = &mut self.mapper {
            frames.remove(&vaddr);
        }
    }

    pub fn write_data(&mut self, offset: usize, data: &[u8]) {
        assert!(offset < self.size);
        assert!(offset + data.len() <= self.size);
        let mut start = offset;
        let mut remain = data.len();
        let mut processed = 0;
        while remain > 0 {
            let start_align = align_down(start, PAGE_SIZE);
            let pgoff = start - start_align;
            let n = (PAGE_SIZE - pgoff).min(remain);

            let vaddr = VirtAddr::new(self.start.as_usize() + start_align);
            let paddr = self.map(vaddr);
            unsafe {
                core::slice::from_raw_parts_mut(paddr.into_kvaddr().as_mut_ptr().add(pgoff), n)
                    .copy_from_slice(&data[processed..processed + n]);
            }
            start += n;
            processed += n;
            remain -= n;
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
            // TODO: check overlap
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

    pub fn load_user(&mut self, elf_data: &[u8]) -> (usize, usize) {
        use xmas_elf::program::{Flags, SegmentData, Type};
        use xmas_elf::{header, ElfFile};

        let elf = ElfFile::new(elf_data).expect("invalid ELF file");
        assert_eq!(
            elf.header.pt1.class(),
            header::Class::SixtyFour,
            "64-bit ELF required"
        );
        assert_eq!(
            elf.header.pt2.type_().as_type(),
            header::Type::Executable,
            "ELF is not an executable object"
        );
        assert_eq!(
            elf.header.pt2.machine().as_machine(),
            header::Machine::AArch64,
            "invalid ELF arch"
        );

        impl From<Flags> for MemFlags {
            fn from(f: Flags) -> Self {
                let mut ret = MemFlags::USER;
                if f.is_read() {
                    ret |= MemFlags::READ;
                }
                if f.is_write() {
                    ret |= MemFlags::WRITE;
                }
                if f.is_execute() {
                    ret |= MemFlags::EXECUTE;
                }
                ret
            }
        }

        for ph in elf.program_iter() {
            if ph.get_type() != Ok(Type::Load) {
                continue;
            }
            let vaddr = VirtAddr::new(ph.virtual_addr() as usize);
            let offset = vaddr.page_offset();
            let area_start = vaddr.align_down();
            let area_end = VirtAddr::new((ph.virtual_addr() + ph.mem_size()) as usize).align_up();
            let data = match ph.get_data(&elf).unwrap() {
                SegmentData::Undefined(data) => data,
                _ => panic!("failed to get ELF segment data"),
            };

            let mut area = MapArea::new_framed(
                area_start,
                area_end.as_usize() - area_start.as_usize(),
                ph.flags().into(),
            );
            area.write_data(offset, data);
            self.insert(area);
            crate::arch::flush_icache_all();
        }
        // user stack
        self.insert(MapArea::new_framed(
            VirtAddr::new(USER_STACK_BASE),
            USER_STACK_SIZE,
            MemFlags::READ | MemFlags::WRITE | MemFlags::USER,
        ));

        let entry = elf.header.pt2.entry_point() as usize;
        let ustack_top = USER_STACK_BASE + USER_STACK_SIZE;
        (entry, ustack_top)
    }

    pub fn clear(&mut self) {
        for area in self.areas.values_mut() {
            self.pt.unmap_area(area);
        }
        self.areas.clear();
    }

    pub fn dup(&self) -> Self {
        let mut ms = Self::new();
        for area in self.areas.values() {
            ms.insert(area.dup());
        }
        ms
    }

    pub fn page_table_root(&self) -> PhysAddr {
        self.pt.root_paddr()
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

    let page_table_root = ms.page_table_root();
    KERNEL_SPACE.init_by(ms);
    unsafe {
        arch::activate_paging(page_table_root.as_usize(), true); // set TTBR0 to zero for kernel tasks
        arch::activate_paging(0, false); // set TTBR0 to zero for kernel tasks
    }
}

impl fmt::Debug for MapArea {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let start = self.start.as_usize();
        let mut s = f.debug_struct("MapArea");
        s.field("va_range", &(start..start + self.size))
            .field("flags", &self.flags);
        match &self.mapper {
            Mapper::Framed(_) => s.field("mapper", &"Frame"),
            Mapper::Offset(off) => s.field("mapper", &alloc::format!("Offset({})", off)),
        }
        .finish()
    }
}

impl fmt::Debug for MemorySet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MemorySet")
            .field("areas", &self.areas.values())
            .field("page_table_root", &self.pt.root_paddr())
            .finish()
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
