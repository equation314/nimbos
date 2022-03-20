mod context;
mod page_table;
mod trap;

pub mod instructions;

pub use self::context::{TaskContext, TrapFrame};
pub use self::page_table::{PageTable, PageTableEntry};

pub fn init() {
    trap::init();
    unsafe { instructions::set_thread_pointer(0) };
}
