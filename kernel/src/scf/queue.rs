use alloc::vec::Vec;
use core::fmt;
use core::mem::{align_of, size_of};
use core::sync::atomic::{fence, AtomicBool, Ordering};

use super::syscall::SyscallCondVar;
use super::ScfOpcode;
use crate::config::scf::*;
use crate::mm::{PhysAddr, VirtAddr};
use crate::sync::LazyInit;
use crate::sync::{spin_lock_irqsave, spin_unlock_irqrestore};

const SYSCALL_QUEUE_BUFFER_MAGIC: u32 = 0x4643537f; // "\x7fSCF"

static mut QUEUE_BUFFER: LazyInit<SyscallQueueBuffer> = LazyInit::new();

#[derive(Clone, Copy, Debug, Default)]
pub struct ScfRequestToken(u64);

#[repr(C)]
#[derive(Debug)]
struct SyscallQueueBufferMetadata {
    magic: u32,
    lock: AtomicBool,
    capacity: u16,
    req_index: u16,
    rsp_index: u16,
}

#[repr(C)]
#[derive(Debug)]
struct ScfDescriptor {
    valid: bool,
    opcode: u8,
    args: u64,
    ret_val: u64,
}

#[derive(Debug)]
pub struct ScfResponse {
    pub opcode: u8,
    pub token: ScfRequestToken,
    pub ret_val: u64,
}

pub struct SyscallQueueBuffer {
    free_count: u16,
    capacity_mask: u16,
    req_index_shadow: u16,
    rsp_index_last: u16,
    tokens: Vec<ScfRequestToken>,

    meta: &'static mut SyscallQueueBufferMetadata,
    desc: &'static mut [ScfDescriptor],
    req_ring: &'static mut [u16],
    rsp_ring: &'static mut [u16],
}

impl ScfRequestToken {
    pub fn from(cond: &SyscallCondVar) -> Self {
        Self(cond as *const _ as _)
    }

    pub const fn is_valid(&self) -> bool {
        self.0 > 0
    }

    pub fn as_cond_var(&self) -> &SyscallCondVar {
        assert!(self.is_valid());
        // FIXME: may be buggy if the task was killed before syscall completed.
        unsafe { &*(self.0 as *const SyscallCondVar) }
    }
}

impl SyscallQueueBuffer {
    pub fn get() -> &'static mut Self {
        unsafe { &mut QUEUE_BUFFER }
    }

    pub fn send(&mut self, opcode: ScfOpcode, args: u64, token: ScfRequestToken) -> bool {
        let flag = spin_lock_irqsave(&self.meta.lock);
        let ret = self.send_locked(opcode, args, token);
        spin_unlock_irqrestore(&self.meta.lock, flag);
        ret
    }

    fn pop_response(&mut self) -> Option<ScfResponse> {
        let flag = spin_lock_irqsave(&self.meta.lock);
        let ret = self.pop_response_locked();
        spin_unlock_irqrestore(&self.meta.lock, flag);
        ret
    }
}

impl SyscallQueueBuffer {
    fn new(base_vaddr: VirtAddr, buf_size: usize) -> Self {
        let meta_size = align_up(
            size_of::<SyscallQueueBufferMetadata>(),
            align_of::<ScfDescriptor>(),
        );
        let capacity = (buf_size - meta_size) / (size_of::<ScfDescriptor>() + size_of::<u16>() * 2);
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
        let desc_base = meta_base + meta_size;
        let req_base = desc_base + capacity * size_of::<ScfDescriptor>();
        let rsp_base = req_base + capacity * size_of::<u16>();

        let meta = unsafe { &mut *(meta_base as *mut SyscallQueueBufferMetadata) };
        let desc =
            unsafe { core::slice::from_raw_parts_mut(desc_base as *mut ScfDescriptor, capacity) };
        let req_ring = unsafe { core::slice::from_raw_parts_mut(req_base as *mut u16, capacity) };
        let rsp_ring = unsafe { core::slice::from_raw_parts_mut(rsp_base as *mut u16, capacity) };

        *meta = SyscallQueueBufferMetadata {
            magic: SYSCALL_QUEUE_BUFFER_MAGIC,
            capacity: capacity as u16,
            lock: AtomicBool::new(false),
            req_index: 0,
            rsp_index: 0,
        };
        desc.iter_mut().for_each(|d| d.valid = false);
        Self {
            free_count: capacity as u16,
            capacity_mask: capacity as u16 - 1,
            req_index_shadow: 0,
            rsp_index_last: 0,
            tokens: alloc::vec![ScfRequestToken::default(); capacity],
            meta,
            desc,
            req_ring,
            rsp_ring,
        }
    }

    fn is_full(&self) -> bool {
        self.free_count == 0
    }

    fn has_response(&self) -> bool {
        self.rsp_index_last != self.meta.rsp_index
    }

    fn find_free_desc_locked(&self) -> Option<u16> {
        for (i, e) in self.desc.iter().enumerate() {
            if !e.valid {
                return Some(i as u16);
            }
        }
        None
    }

    fn free_desc_locked(&mut self, index: u16) {
        assert!(index <= self.capacity_mask);
        self.tokens[index as usize] = ScfRequestToken::default();
        self.desc[index as usize].valid = false;
        self.free_count += 1;
    }

    fn pop_response_locked(&mut self) -> Option<ScfResponse> {
        if self.has_response() {
            fence(Ordering::SeqCst);
            let entry_idx = self.rsp_ring[(self.rsp_index_last & self.capacity_mask) as usize];
            self.rsp_index_last = self.rsp_index_last.wrapping_add(1);
            if entry_idx > self.capacity_mask {
                warn!(
                    "SyscallQueueBuffer: entry_idx {} >= capacity {}!",
                    entry_idx,
                    self.capacity_mask + 1
                );
                return None;
            }
            let entry = &self.desc[entry_idx as usize];
            if !entry.valid {
                warn!("SyscallQueueBuffer: invalid {:#x?}", entry);
                return None;
            }

            let rsp = ScfResponse {
                opcode: entry.opcode,
                ret_val: entry.ret_val,
                token: self.tokens[entry_idx as usize],
            };
            self.free_desc_locked(entry_idx);
            Some(rsp)
        } else {
            None
        }
    }

    fn send_locked(&mut self, opcode: ScfOpcode, args: u64, token: ScfRequestToken) -> bool {
        if self.is_full() {
            return false;
        }

        let entry_idx = self.find_free_desc_locked().unwrap();
        self.tokens[entry_idx as usize] = token;
        self.desc[entry_idx as usize] = ScfDescriptor {
            valid: true,
            opcode: opcode as u8,
            args,
            ret_val: 0,
        };
        self.free_count -= 1;

        self.req_ring[(self.req_index_shadow & self.capacity_mask) as usize] = entry_idx;
        self.req_index_shadow = self.req_index_shadow.wrapping_add(1);
        fence(Ordering::SeqCst);
        self.meta.req_index = self.req_index_shadow;
        true
    }
}

impl fmt::Debug for SyscallQueueBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("SyscallQueueBuffer");
        f.field("capacity_mask", &self.capacity_mask)
            .field("free_count", &self.free_count)
            .field("req_index_shadow", &self.req_index_shadow)
            .field("rsp_index_last", &self.rsp_index_last)
            .field("meta", &self.meta)
            .finish()
    }
}

const fn align_up(addr: usize, alignment: usize) -> usize {
    (addr + alignment - 1) & !(alignment - 1)
}

fn handle_irq() {
    while let Some(rsp) = SyscallQueueBuffer::get().pop_response() {
        if rsp.token.is_valid() {
            rsp.token.as_cond_var().signal(rsp.ret_val);
        }
    }
}

pub fn init() {
    unsafe { &QUEUE_BUFFER }.init_by(SyscallQueueBuffer::new(
        PhysAddr::new(SYSCALL_QUEUE_BUF_PADDR).into_kvaddr(),
        SYSCALL_QUEUE_BUF_SIZE,
    ));
    crate::drivers::timer::add_timer_event(handle_irq);
}
