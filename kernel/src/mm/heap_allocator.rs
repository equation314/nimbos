use buddy_system_allocator::Heap;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

use crate::config::KERNEL_HEAP_SIZE;
use crate::sync::SpinNoIrqLock;

struct LockedHeap(SpinNoIrqLock<Heap<32>>);

impl LockedHeap {
    pub const fn empty() -> Self {
        LockedHeap(SpinNoIrqLock::new(Heap::<32>::new()))
    }

    pub fn init(&self, start: usize, size: usize) {
        unsafe { self.0.lock().init(start, size) };
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0
            .lock()
            .alloc(layout)
            .ok()
            .map_or(core::ptr::null_mut(), |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init_heap() {
    let heap_start = unsafe { HEAP_SPACE.as_ptr() as usize };
    println!(
        "Initialising kernel heap at: [{:#x}, {:#x})",
        heap_start,
        heap_start + KERNEL_HEAP_SIZE
    );
    HEAP_ALLOCATOR.init(heap_start, KERNEL_HEAP_SIZE);
}

#[allow(unused)]
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    extern "C" {
        fn sbss();
        fn ebss();
    }
    let bss_range = sbss as usize..ebss as usize;
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    for (i, val) in v.iter().take(500).enumerate() {
        assert_eq!(*val, i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    println!("heap_test passed!");
}
