use alloc::{vec, vec::Vec};
use core::fmt;

use crate::mm::{MapArea, MemFlags, PhysAddr, PhysFrame, VirtAddr, PAGE_SIZE};

bitflags::bitflags! {
    /// Memory attribute fields in the VMSAv8-64 translation table format descriptors.
    pub struct DescriptorAttr: u64 {
        // Attribute fields in stage 1 VMSAv8-64 Block and Page descriptors:

        /// Whether the descriptor is valid.
        const VALID =       1 << 0;
        /// The descriptor gives the address of the next level of translation table or 4KB page.
        /// (not a 2M, 1G block)
        const NON_BLOCK =   1 << 1;
        /// Memory attributes index field.
        const ATTR_INDX =   0b111 << 2;
        /// Non-secure bit. For memory accesses from Secure state, specifies whether the output
        /// address is in Secure or Non-secure memory.
        const NS =          1 << 5;
        /// Access permission: accessable at EL0.
        const AP_EL0 =      1 << 6;
        /// Access permission: read-only.
        const AP_RO =       1 << 7;
        /// Shareability: Inner Shareable (otherwise Outer Shareable).
        const INNER =       1 << 8;
        /// Shareability: Inner or Outer Shareable (otherwise Non-shareable).
        const SHAREABLE =   1 << 9;
        /// The Access flag.
        const AF =          1 << 10;
        /// The not global bit.
        const NG =          1 << 11;
        /// Indicates that 16 adjacent translation table entries point to contiguous memory regions.
        const CONTIGUOUS =  1 <<  52;
        /// The Privileged execute-never field.
        const PXN =         1 <<  53;
        /// The Execute-never or Unprivileged execute-never field.
        const UXN =         1 <<  54;

        // Next-level attributes in stage 1 VMSAv8-64 Table descriptors:

        /// PXN limit for subsequent levels of lookup.
        const PXN_TABLE =           1 << 59;
        /// XN limit for subsequent levels of lookup.
        const XN_TABLE =            1 << 60;
        /// Access permissions limit for subsequent levels of lookup: access at EL0 not permitted.
        const AP_NO_EL0_TABLE =     1 << 61;
        /// Access permissions limit for subsequent levels of lookup: write access not permitted.
        const AP_NO_WRITE_TABLE =   1 << 62;
        /// For memory accesses from Secure state, specifies the Security state for subsequent
        /// levels of lookup.
        const NS_TABLE =            1 << 63;
    }
}

#[repr(u64)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum MemType {
    Device = 0,
    Normal = 1,
}

impl DescriptorAttr {
    const ATTR_INDEX_MASK: u64 = 0b111_00;

    const fn from_mem_type(mem_type: MemType) -> Self {
        let mut bits = (mem_type as u64) << 2;
        if matches!(mem_type, MemType::Normal) {
            bits |= Self::INNER.bits() | Self::SHAREABLE.bits();
        }
        Self::from_bits_truncate(bits)
    }

    fn mem_type(&self) -> MemType {
        let idx = (self.bits() & Self::ATTR_INDEX_MASK) >> 2;
        match idx {
            0 => MemType::Device,
            1 => MemType::Normal,
            _ => panic!("Invalid memory attribute index"),
        }
    }
}

impl From<DescriptorAttr> for MemFlags {
    fn from(attr: DescriptorAttr) -> Self {
        let mut flags = Self::empty();
        if attr.contains(DescriptorAttr::VALID) {
            flags |= Self::READ;
        }
        if !attr.contains(DescriptorAttr::AP_RO) {
            flags |= Self::WRITE;
        }
        if attr.contains(DescriptorAttr::AP_EL0) {
            flags |= Self::USER;
            if !attr.contains(DescriptorAttr::UXN) {
                flags |= Self::EXECUTE;
            }
        } else if !attr.intersects(DescriptorAttr::PXN) {
            flags |= Self::EXECUTE;
        }
        if attr.mem_type() == MemType::Device {
            flags |= Self::DEVICE;
        }
        flags
    }
}

impl From<MemFlags> for DescriptorAttr {
    fn from(flags: MemFlags) -> Self {
        let mut attr = if flags.contains(MemFlags::DEVICE) {
            Self::from_mem_type(MemType::Device)
        } else {
            Self::from_mem_type(MemType::Normal)
        };
        if flags.contains(MemFlags::READ) {
            attr |= Self::VALID;
        }
        if !flags.contains(MemFlags::WRITE) {
            attr |= Self::AP_RO;
        }
        if flags.contains(MemFlags::USER) {
            attr |= Self::AP_EL0 | Self::PXN;
            if !flags.contains(MemFlags::EXECUTE) {
                attr |= Self::UXN;
            }
        } else {
            attr |= Self::UXN;
            if !flags.contains(MemFlags::EXECUTE) {
                attr |= Self::PXN;
            }
        }
        attr
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    const PHYS_ADDR_MASK: usize = PhysAddr::MAX & !(PAGE_SIZE - 1);

    pub const fn empty() -> Self {
        Self(0)
    }
    pub fn new_page(paddr: PhysAddr, flags: MemFlags, is_block: bool) -> Self {
        let mut attr = DescriptorAttr::from(flags) | DescriptorAttr::AF;
        if !is_block {
            attr |= DescriptorAttr::NON_BLOCK;
        }
        Self(attr.bits() | (paddr.as_usize() & Self::PHYS_ADDR_MASK) as u64)
    }
    pub fn new_table(paddr: PhysAddr) -> Self {
        let attr = DescriptorAttr::NON_BLOCK | DescriptorAttr::VALID;
        Self(attr.bits() | (paddr.as_usize() & Self::PHYS_ADDR_MASK) as u64)
    }

    fn paddr(&self) -> PhysAddr {
        PhysAddr::new(self.0 as usize & Self::PHYS_ADDR_MASK)
    }
    fn flags(&self) -> MemFlags {
        DescriptorAttr::from_bits_truncate(self.0).into()
    }
    fn is_unused(&self) -> bool {
        self.0 == 0
    }
    fn is_present(&self) -> bool {
        DescriptorAttr::from_bits_truncate(self.0).contains(DescriptorAttr::VALID)
    }
    fn is_block(&self) -> bool {
        !DescriptorAttr::from_bits_truncate(self.0).contains(DescriptorAttr::NON_BLOCK)
    }
    fn clear(&mut self) {
        self.0 = 0
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("PageTableEntry");
        f.field("raw", &self.0)
            .field("paddr", &self.paddr())
            .field("attr", &DescriptorAttr::from_bits_truncate(self.0))
            .field("flags", &self.flags())
            .finish()
    }
}

pub struct PageTable {
    root_paddr: PhysAddr,
    intrm_tables: Vec<PhysFrame>,
}

impl PageTable {
    pub fn new() -> Self {
        let root_frame = PhysFrame::alloc_zero().unwrap();
        Self {
            root_paddr: root_frame.start_paddr(),
            intrm_tables: vec![root_frame],
        }
    }

    pub const fn root_paddr(&self) -> PhysAddr {
        self.root_paddr
    }

    #[allow(unused)]
    pub unsafe fn from_root(root_paddr: PhysAddr) -> Self {
        Self {
            root_paddr,
            intrm_tables: Vec::new(),
        }
    }

    pub fn map(&mut self, vaddr: VirtAddr, paddr: PhysAddr, flags: MemFlags) {
        let entry = self.get_entry_mut_or_create(vaddr).unwrap();
        if !entry.is_unused() {
            panic!("{:#x?} is mapped before mapping", vaddr);
        }
        *entry = PageTableEntry::new_page(paddr.align_down(), flags, false);
    }

    pub fn unmap(&mut self, vaddr: VirtAddr) {
        let entry = self.get_entry_mut(vaddr).unwrap();
        if entry.is_unused() {
            panic!("{:#x?} is invalid before unmapping", vaddr);
        }
        entry.clear();
    }

    pub fn query(&self, vaddr: VirtAddr) -> Option<(PhysAddr, MemFlags)> {
        let entry = self.get_entry_mut(vaddr)?;
        if entry.is_unused() {
            return None;
        }
        let off = vaddr.page_offset();
        Some((PhysAddr::new(entry.paddr().as_usize() + off), entry.flags()))
    }

    pub fn map_area(&mut self, area: &mut MapArea) {
        let mut vaddr = area.start.as_usize();
        let end = vaddr + area.size;
        while vaddr < end {
            let paddr = area.map(VirtAddr::new(vaddr));
            self.map(VirtAddr::new(vaddr), paddr, area.flags);
            vaddr += PAGE_SIZE;
        }
    }

    pub fn unmap_area(&mut self, area: &mut MapArea) {
        let mut vaddr = area.start.as_usize();
        let end = vaddr + area.size;
        while vaddr < end {
            area.unmap(VirtAddr::new(vaddr));
            self.unmap(VirtAddr::new(vaddr));
            vaddr += PAGE_SIZE;
        }
    }

    #[allow(unused)]
    pub fn dump(&self, limit: usize) {
        use crate::sync::SpinNoIrqLock;
        static LOCK: SpinNoIrqLock<()> = SpinNoIrqLock::new(());
        let _lock = LOCK.lock();

        println!("Root: {:x?}", self.root_paddr());
        self.walk(
            table_of(self.root_paddr()),
            0,
            0,
            limit,
            &|level: usize, idx: usize, vaddr: usize, entry: &PageTableEntry| {
                for _ in 0..level {
                    print!("  ");
                }
                println!("[{} - {:x}], {:08x?}: {:x?}", level, idx, vaddr, entry);
            },
        );
    }
}

impl PageTable {
    fn alloc_intrm_table(&mut self) -> PhysAddr {
        let frame = PhysFrame::alloc_zero().unwrap();
        let paddr = frame.start_paddr();
        self.intrm_tables.push(frame);
        paddr
    }

    fn get_entry_mut(&self, vaddr: VirtAddr) -> Option<&mut PageTableEntry> {
        let p4 = table_of_mut(self.root_paddr());
        let p4e = &mut p4[p4_index(vaddr)];

        let p3 = next_table_mut(p4e)?;
        let p3e = &mut p3[p3_index(vaddr)];

        let p2 = next_table_mut(p3e)?;
        let p2e = &mut p2[p2_index(vaddr)];

        let p1 = next_table_mut(p2e)?;
        let p1e = &mut p1[p1_index(vaddr)];
        Some(p1e)
    }

    fn get_entry_mut_or_create(&mut self, vaddr: VirtAddr) -> Option<&mut PageTableEntry> {
        let p4 = table_of_mut(self.root_paddr());
        let p4e = &mut p4[p4_index(vaddr)];

        let p3 = next_table_mut_or_create(p4e, || self.alloc_intrm_table())?;
        let p3e = &mut p3[p3_index(vaddr)];

        let p2 = next_table_mut_or_create(p3e, || self.alloc_intrm_table())?;
        let p2e = &mut p2[p2_index(vaddr)];

        let p1 = next_table_mut_or_create(p2e, || self.alloc_intrm_table())?;
        let p1e = &mut p1[p1_index(vaddr)];
        Some(p1e)
    }

    fn walk(
        &self,
        table: &[PageTableEntry],
        level: usize,
        start_vaddr: usize,
        limit: usize,
        func: &impl Fn(usize, usize, usize, &PageTableEntry),
    ) {
        let mut n = 0;
        for (i, entry) in table.iter().enumerate() {
            let vaddr = start_vaddr + (i << (12 + (3 - level) * 9));
            if entry.is_present() {
                func(level, i, vaddr, entry);
                if level < 3 && !entry.is_block() {
                    let table_entry = next_table_mut(entry).unwrap();
                    self.walk(table_entry, level + 1, vaddr, limit, func);
                }
                n += 1;
                if n >= limit {
                    break;
                }
            }
        }
    }
}

const ENTRY_COUNT: usize = 512;

const fn p4_index(vaddr: VirtAddr) -> usize {
    (vaddr.as_usize() >> (12 + 27)) & (ENTRY_COUNT - 1)
}

const fn p3_index(vaddr: VirtAddr) -> usize {
    (vaddr.as_usize() >> (12 + 18)) & (ENTRY_COUNT - 1)
}

const fn p2_index(vaddr: VirtAddr) -> usize {
    (vaddr.as_usize() >> (12 + 9)) & (ENTRY_COUNT - 1)
}

const fn p1_index(vaddr: VirtAddr) -> usize {
    (vaddr.as_usize() >> 12) & (ENTRY_COUNT - 1)
}

fn table_of<'a>(paddr: PhysAddr) -> &'a [PageTableEntry] {
    let ptr = paddr.into_kvaddr().as_ptr() as *const PageTableEntry;
    unsafe { core::slice::from_raw_parts(ptr, ENTRY_COUNT) }
}

fn table_of_mut<'a>(paddr: PhysAddr) -> &'a mut [PageTableEntry] {
    let ptr = paddr.into_kvaddr().as_mut_ptr() as *mut PageTableEntry;
    unsafe { core::slice::from_raw_parts_mut(ptr, ENTRY_COUNT) }
}

fn next_table_mut<'a>(entry: &PageTableEntry) -> Option<&'a mut [PageTableEntry]> {
    if !entry.is_present() {
        None
    } else {
        assert!(!entry.is_block());
        Some(table_of_mut(entry.paddr()))
    }
}

fn next_table_mut_or_create<'a>(
    entry: &mut PageTableEntry,
    mut allocator: impl FnMut() -> PhysAddr,
) -> Option<&'a mut [PageTableEntry]> {
    if entry.is_unused() {
        let paddr = allocator();
        *entry = PageTableEntry::new_table(paddr);
        Some(table_of_mut(paddr))
    } else {
        next_table_mut(entry)
    }
}
