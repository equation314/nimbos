use core::alloc::Layout;
use core::mem::{align_of, size_of};
use core::ptr::NonNull;

use buddy_system_allocator::Heap;

use crate::config::ipc::*;
use crate::mm::{PhysAddr, VirtAddr};
use crate::sync::Mutex;

const DATA_POOL_BASE: VirtAddr = PhysAddr::new(SYSCALL_DATA_BUF_PADDR).into_kvaddr();

struct IpcDataPool(Mutex<Heap<32>>);

static IPC_DATA_POOL: IpcDataPool = IpcDataPool(Mutex::new(Heap::<32>::new()));

pub fn init() {
    unsafe {
        IPC_DATA_POOL
            .0
            .lock()
            .init(DATA_POOL_BASE.as_usize(), SYSCALL_DATA_BUF_SIZE);
    }
}

fn alloc_common(layout: Layout) -> *mut u8 {
    let ptr = IPC_DATA_POOL
        .0
        .lock()
        .alloc(layout)
        .ok()
        .map_or(core::ptr::null_mut(), |allocation| allocation.as_ptr());
    assert!(!ptr.is_null(), "syscall data pool is full");
    ptr
}

pub unsafe fn alloc_uninit<T: Sized>() -> *mut T {
    alloc_common(Layout::new::<T>()) as *mut T
}

pub unsafe fn alloc_array_uninit<T: Sized>(len: usize) -> *mut T {
    let (size, align) = (size_of::<T>(), align_of::<T>());
    let layout = Layout::from_size_align(size * len, align).unwrap();
    alloc_common(layout) as *mut T
}

pub fn alloc<T: Sized>(data: T) -> *mut T {
    unsafe {
        let ptr = alloc_uninit();
        core::ptr::write(ptr, data);
        ptr
    }
}

#[allow(dead_code)]
pub unsafe fn dealloc<T: Sized>(ptr: *const T) {
    IPC_DATA_POOL
        .0
        .lock()
        .dealloc(NonNull::new_unchecked(ptr as *mut u8), Layout::new::<T>())
}

pub fn as_offset<T>(ptr: *const T) -> u64 {
    let ptr = ptr as usize;
    let base = DATA_POOL_BASE.as_usize();
    assert!(ptr >= base);
    (ptr - base) as u64
}
