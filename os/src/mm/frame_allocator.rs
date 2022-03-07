#![allow(dead_code)]

use alloc::vec::Vec;
use core::ops::Range;

use super::{address::virt_to_phys, PhysAddr, PAGE_SIZE};
use crate::config::MEMORY_END;
use crate::sync::SpinNoIrqLock;

static FRAME_ALLOCATOR: SpinNoIrqLock<FreeListAllocator> =
    SpinNoIrqLock::new(FreeListAllocator::empty());

trait FrameAllocator {
    fn alloc(&mut self) -> Option<usize>;
    fn dealloc(&mut self, value: usize);
}

pub struct FreeListAllocator {
    range: Range<usize>,
    current: usize,
    free_list: Vec<usize>,
}

impl FreeListAllocator {
    const fn empty() -> Self {
        Self {
            range: 0..0,
            current: 0,
            free_list: Vec::new(),
        }
    }

    fn init(&mut self, start: usize, end: usize) {
        self.range = start..end;
        self.current = start;
    }
}

impl FrameAllocator for FreeListAllocator {
    fn alloc(&mut self) -> Option<usize> {
        if let Some(value) = self.free_list.pop() {
            Some(value)
        } else if self.current >= self.range.end {
            None
        } else {
            self.current += 1;
            Some(self.current - 1)
        }
    }

    fn dealloc(&mut self, value: usize) {
        // validity check
        assert!(self.range.contains(&value));
        assert!(!self.free_list.contains(&value));
        // recycle
        self.free_list.push(value);
    }
}

#[derive(Debug)]
pub struct PhysFrame {
    start_paddr: PhysAddr,
}

impl PhysFrame {
    pub fn alloc() -> Option<Self> {
        FRAME_ALLOCATOR.lock().alloc().map(|value| Self {
            start_paddr: PhysAddr::new(value * PAGE_SIZE),
        })
    }

    pub fn alloc_zero() -> Option<Self> {
        let mut f = Self::alloc()?;
        f.zero();
        Some(f)
    }

    pub fn start_paddr(&self) -> PhysAddr {
        self.start_paddr
    }

    pub fn zero(&mut self) {
        unsafe { core::ptr::write_bytes(self.start_paddr.into_kvaddr().as_mut_ptr(), 0, PAGE_SIZE) }
    }
}

impl Drop for PhysFrame {
    fn drop(&mut self) {
        FRAME_ALLOCATOR
            .lock()
            .dealloc(self.start_paddr.as_usize() / PAGE_SIZE);
    }
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    let start_paddr = PhysAddr::new(virt_to_phys(ekernel as usize)).align_up();
    let end_paddr = PhysAddr::new(MEMORY_END).align_down();
    FRAME_ALLOCATOR.lock().init(
        start_paddr.as_usize() / PAGE_SIZE,
        end_paddr.as_usize() / PAGE_SIZE,
    );
}

#[allow(unused)]
pub fn frame_allocator_test() {
    let mut v: Vec<PhysFrame> = Vec::new();
    for i in 0..5 {
        let frame = PhysFrame::alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = PhysFrame::alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}
