mod context;
mod gdt;
mod idt;
mod page_table;
mod percpu;
mod trap;

pub mod instructions;

pub use self::context::{TaskContext, TrapFrame};
pub use self::page_table::{PageTable, PageTableEntry};
pub use self::percpu::ArchPerCpu;

pub fn init() {
    unsafe { instructions::set_thread_pointer(0) };
    idt::init();
}
