mod address;
mod frame_allocator;
mod heap_allocator;

pub use address::{PhysAddr, VirtAddr};
pub use frame_allocator::PhysFrame;

pub const PAGE_SIZE: usize = 0x1000;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
}
