use core::mem::size_of;
use core::sync::atomic::{AtomicU16, Ordering};

use super::IpcOpcode;
use crate::config::scf::*;
use crate::mm::{PhysAddr, VirtAddr};
use crate::sync::LazyInit;

const SYSCALL_QUEUE_BUFFER_MAGIC: u32 = 0x4643537f; // "\x7fSCF"

static QUEUE_BUFFER: LazyInit<SyscallQueueBuffer> = LazyInit::new();

#[repr(C)]
struct SyscallQueueBufferMetadata {
    magic: u32,
    lock: AtomicU16,
    capacity: u16,
    req_head: u16,
    req_tail: u16,
    rsp_head: u16,
    rsp_tail: u16,
}

#[repr(C)]
struct ScfEntry {
    valid: bool,
    opcode: u8,
    args: u64,
}

pub struct SyscallQueueBuffer {
    meta: &'static mut SyscallQueueBufferMetadata,
    entries: &'static mut [ScfEntry],
    req_ring: &'static mut [u16],
    rsp_ring: &'static mut [u16],
}

impl SyscallQueueBuffer {
    pub fn get() -> &'static Self {
        &QUEUE_BUFFER
    }

    pub fn send(&self, opcode: IpcOpcode, args: u64) -> bool {
        false
    }
}

impl SyscallQueueBuffer {
    fn new(base_vaddr: VirtAddr, buf_size: usize) -> Self {
        let capacity = (buf_size - size_of::<SyscallQueueBufferMetadata>())
            / (size_of::<ScfEntry>() + size_of::<u16>() * 2);
        let capacity = if capacity.is_power_of_two() {
            capacity
        } else {
            (capacity / 2).next_power_of_two()
        };
        assert!(capacity > 0);
        assert!(capacity < u16::MAX as usize);
        assert!(base_vaddr.is_aligned());
        info!("Initializing SyscallQueueBuffer: capacity={}", capacity);

        let meta_base = base_vaddr.as_usize();
        let entries_base = meta_base + size_of::<SyscallQueueBufferMetadata>();
        let req_base = entries_base + capacity * size_of::<ScfEntry>();
        let rsp_base = req_base + capacity * size_of::<u16>();

        let meta = unsafe { &mut *(meta_base as *mut SyscallQueueBufferMetadata) };
        let entries =
            unsafe { core::slice::from_raw_parts_mut(entries_base as *mut ScfEntry, capacity) };
        let req_ring = unsafe { core::slice::from_raw_parts_mut(req_base as *mut u16, capacity) };
        let rsp_ring = unsafe { core::slice::from_raw_parts_mut(rsp_base as *mut u16, capacity) };

        *meta = SyscallQueueBufferMetadata {
            magic: SYSCALL_QUEUE_BUFFER_MAGIC,
            capacity: capacity as u16,
            lock: AtomicU16::new(0),
            req_head: 0,
            req_tail: 0,
            rsp_head: 0,
            rsp_tail: 0,
        };
        Self {
            meta,
            entries,
            req_ring,
            rsp_ring,
        }
    }
}

pub fn init() {
    QUEUE_BUFFER.init_by(SyscallQueueBuffer::new(
        PhysAddr::new(SYSCALL_QUEUE_BUF_PADDR).into_kvaddr(),
        SYSCALL_QUEUE_BUF_SIZE,
    ));
}
