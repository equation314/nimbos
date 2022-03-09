use core::arch::global_asm;

use xmas_elf::program::{Flags, SegmentData, Type};
use xmas_elf::{header, ElfFile};

use crate::config::{USER_STACK_BASE, USER_STACK_SIZE};
use crate::mm::{MapArea, MemFlags, MemorySet, VirtAddr};

global_asm!(include_str!("link_app.S"));

extern "C" {
    fn _app_count();
}

pub fn get_app_count() -> usize {
    unsafe { (_app_count as *const u64).read() as usize }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    unsafe {
        let app_0_start_ptr = (_app_count as *const u64).add(1);
        assert!(app_id < get_app_count());
        let app_start = app_0_start_ptr.add(app_id).read() as usize;
        let app_end = app_0_start_ptr.add(app_id + 1).read() as usize;
        let app_size = app_end - app_start;
        core::slice::from_raw_parts(app_start as *const u8, app_size)
    }
}

pub fn load_app(app_id: usize) -> (usize, usize, MemorySet) {
    assert!(app_id < get_app_count());

    let elf_data = get_app_data(app_id);
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

    let mut ms = MemorySet::new();
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
        ms.insert(area);
        crate::arch::flush_icache_all();
    }
    // user stack
    ms.insert(MapArea::new_framed(
        VirtAddr::new(USER_STACK_BASE),
        USER_STACK_SIZE,
        MemFlags::READ | MemFlags::WRITE | MemFlags::USER,
    ));

    let entry = elf.header.pt2.entry_point() as usize;
    let ustack_top = USER_STACK_BASE + USER_STACK_SIZE;
    (entry, ustack_top, ms)
}

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
