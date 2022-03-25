mod context;
mod page_table;
mod percpu;
mod trap;

pub mod config;
pub mod instructions;

pub use self::context::{TaskContext, TrapFrame};
pub use self::page_table::{PageTable, PageTableEntry};
pub use self::percpu::ArchPerCpu;

pub fn init() {
    trap::init();
}
