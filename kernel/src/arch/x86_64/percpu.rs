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
        IDT.load();
        self.gdt.init(&self.tss);
        self.gdt.load();
        self.gdt.load_tss(TSS_SELECTOR);
    }

    pub fn kernel_stack_top(&self) -> VirtAddr {
        VirtAddr::new(self.tss.kernel_stack_top() as usize)
    }

    pub fn set_kernel_stack_top(&mut self, kstack_top: VirtAddr) {
        trace!("set percpu kernel stack: {:#x?}", kstack_top);
        self.tss.set_kernel_stack_top(kstack_top.as_usize() as u64)
    }
}
