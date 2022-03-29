use core::mem::size_of;
use core::sync::atomic::{AtomicU8, Ordering};

use memoffset::offset_of;

use crate::config::ipc::*;
use crate::mm::PhysAddr;
use crate::task::CurrentTask;

numeric_enum_macro::numeric_enum! {
    #[repr(u8)]
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub enum IpcOpcode {
        Nop = 0,
        Read = 1,
        Write = 2,
        Open = 3,
        Close = 4,
        Unknown = 0xff,
    }
}

#[repr(C)]
#[derive(Debug, Default)]
struct IpcBufferEntry {
    opcode: u8,
    args: u64,
}

#[repr(C)]
#[derive(Debug)]
pub(super) struct IpcBuffer {
    lock: AtomicU8,
    head: u32,
    tail: u32,
    capacity: u32,
    capacity_mask: u32,
    entries: [IpcBufferEntry; 0],
}

impl IpcBufferEntry {
    pub fn new(opcode: IpcOpcode, args: u64) -> Self {
        Self {
            opcode: opcode as u8,
            args,
        }
    }
}

impl IpcBuffer {
    pub fn get_send_buffer<'a>() -> &'a mut Self {
        unsafe {
            &mut *(PhysAddr::new(SYSCALL_SEND_BUF_PADDR)
                .into_kvaddr()
                .as_mut_ptr() as *mut Self)
        }
    }

    pub fn get_recv_buffer<'a>() -> &'a mut Self {
        unsafe {
            &mut *(PhysAddr::new(SYSCALL_RECV_BUF_PADDR)
                .into_kvaddr()
                .as_mut_ptr() as *mut Self)
        }
    }

    pub fn init(&mut self, buf_size: usize) {
        let entries_off = offset_of!(IpcBuffer, entries);
        let capacity = (buf_size - entries_off) / size_of::<IpcBufferEntry>();
        let capacity = if capacity.is_power_of_two() {
            capacity
        } else {
            (capacity / 2).next_power_of_two()
        };
        assert!(capacity > 0);
        self.lock = AtomicU8::new(0);
        self.head = 0;
        self.tail = 0;
        self.capacity = capacity as u32;
        self.capacity_mask = capacity as u32 - 1;
    }

    pub fn send(&mut self, opcode: IpcOpcode, args: u64) -> bool {
        self.lock();
        let ret = self.send_locked(opcode, args);
        self.unlock();
        ret
    }
}

impl IpcBuffer {
    fn count(&self) -> u32 {
        self.tail.wrapping_sub(self.head)
    }

    fn is_full(&self) -> bool {
        self.count() == self.capacity
    }

    fn is_locked(&self) -> bool {
        self.lock.load(Ordering::Relaxed) != 0
    }

    fn lock(&self) {
        while self
            .lock
            .compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.is_locked() {
                CurrentTask::get().yield_now();
            }
        }
    }

    fn unlock(&self) {
        self.lock.store(0, Ordering::Release);
    }

    fn entry_at_locked(&mut self, idx: u32) -> &mut IpcBufferEntry {
        unsafe {
            let ptr = (self as *mut _ as *mut u8).add(offset_of!(IpcBuffer, entries))
                as *mut IpcBufferEntry;
            &mut *ptr.add((idx & self.capacity_mask) as usize)
        }
    }

    fn send_locked(&mut self, opcode: IpcOpcode, args: u64) -> bool {
        if self.is_full() {
            return false;
        }
        *self.entry_at_locked(self.tail) = IpcBufferEntry::new(opcode, args);
        self.tail += 1;
        true
    }
}
