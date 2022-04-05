use core::alloc::Layout;
use core::mem::{align_of, size_of};
use core::ptr::NonNull;

use buddy_system_allocator::Heap;

use crate::config::scf::*;
use crate::mm::{PhysAddr, VirtAddr};
use crate::sync::Mutex;

const DATA_BUF_BASE: PhysAddr = PhysAddr::new(SYSCALL_DATA_BUF_PADDR);

pub struct SyscallDataBuffer {
    base_vaddr: usize,
    size: usize,
    heap: Mutex<Heap<32>>,
}

static DATA_BUFFER: SyscallDataBuffer =
    SyscallDataBuffer::new(DATA_BUF_BASE.into_kvaddr(), SYSCALL_DATA_BUF_SIZE);

impl SyscallDataBuffer {
    pub fn get() -> &'static Self {
        &DATA_BUFFER
    }

    const fn new(base_vaddr: VirtAddr, buf_size: usize) -> Self {
        Self {
            base_vaddr: base_vaddr.as_usize(),
            size: buf_size,
            heap: Mutex::new(Heap::<32>::new()),
        }
    }

    pub fn init(&self) {
        unsafe {
            self.heap.lock().init(self.base_vaddr, self.size);
        }
    }

    pub fn offset_of<T>(&self, ptr: *const T) -> u64 {
        let ptr = ptr as usize;
        assert!((self.base_vaddr..self.base_vaddr + self.size).contains(&ptr));
        (ptr - self.base_vaddr) as u64
    }

    unsafe fn alloc_common(&self, layout: Layout) -> *mut u8 {
        let ptr = self
            .heap
            .lock()
            .alloc(layout)
            .ok()
            .map_or(core::ptr::null_mut(), |allocation| allocation.as_ptr());
        assert!(!ptr.is_null(), "syscall data pool is full");
        ptr
    }

    pub unsafe fn alloc_uninit<T: Sized>(&self) -> *mut T {
        self.alloc_common(Layout::new::<T>()) as *mut T
    }

    pub unsafe fn alloc_array_uninit<T: Sized>(&self, len: usize) -> *mut T {
        let (size, align) = (size_of::<T>(), align_of::<T>());
        let layout = Layout::from_size_align(size * len, align).unwrap();
        self.alloc_common(layout) as *mut T
    }

    pub fn alloc<T: Sized>(&self, data: T) -> *mut T {
        unsafe {
            let ptr = self.alloc_uninit();
            core::ptr::write(ptr, data);
            ptr
        }
    }

    pub unsafe fn dealloc<T: Sized>(&self, ptr: *const T) {
        assert!((self.base_vaddr..self.base_vaddr + self.size).contains(&(ptr as usize)));
        self.heap
            .lock()
            .dealloc(NonNull::new_unchecked(ptr as *mut u8), Layout::new::<T>())
    }
}

pub fn init() {
    DATA_BUFFER.init();
}
