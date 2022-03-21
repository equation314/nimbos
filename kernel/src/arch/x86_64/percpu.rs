use super::gdt::{GdtStruct, TssStruct, TSS_SELECTOR};
use super::idt::IDT;
use crate::mm::VirtAddr;

pub struct ArchPerCpu {
    tss: TssStruct,
    gdt: GdtStruct,
}

impl ArchPerCpu {
    pub fn new() -> Self {
        Self {
            tss: TssStruct::alloc(),
            gdt: GdtStruct::alloc(),
        }
    }

    pub fn init(&mut self, cpu_id: usize) {
        println!("Loading IDT and GDT for CPU {}...", cpu_id);
        self.gdt.init(&self.tss);
        self.gdt.load();
        self.gdt.load_tss(TSS_SELECTOR);
        IDT.load();
    }

    pub fn set_kernel_stack(&mut self, kstack_top: VirtAddr) {
        self.tss.set_kernel_stack(kstack_top.as_usize() as u64)
    }
}
