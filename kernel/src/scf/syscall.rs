use core::slice::{from_raw_parts, from_raw_parts_mut};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::allocator::SyscallDataBuffer;
use super::queue::{ScfRequestToken, SyscallQueueBuffer};
use crate::mm::{UserInPtr, UserOutPtr};
use crate::task::CurrentTask;

numeric_enum_macro::numeric_enum! {
    #[repr(u8)]
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub enum ScfOpcode {
        Nop = 0,
        Read = 1,
        Write = 2,
        Open = 3,
        Close = 4,
        Unknown = 0xff,
    }
}

const CHUNK_SIZE: usize = 256;

pub struct SyscallCondVar {
    ok: AtomicBool,
    ret_val: AtomicU64,
}

impl SyscallCondVar {
    pub fn new() -> Self {
        Self {
            ok: AtomicBool::new(false),
            ret_val: AtomicU64::new(0),
        }
    }

    pub fn signal(&self, ret_val: u64) {
        self.ret_val.store(ret_val, Ordering::Release);
        self.ok.store(true, Ordering::Release);
    }

    pub fn wait(&self) -> u64 {
        while !self.ok.load(Ordering::Acquire) {
            CurrentTask::get().yield_now();
        }
        self.ret_val.load(Ordering::Acquire)
    }
}

#[repr(C)]
#[derive(Debug)]
struct ReadWriteArgs {
    fd: u32,
    buf_offset: u64,
    len: u64,
}

fn send_request(opcode: ScfOpcode, args: u64, token: ScfRequestToken) {
    while !SyscallQueueBuffer::get().send(opcode, args, token) {
        CurrentTask::get().yield_now();
    }
    super::notify();
}

pub fn sys_write(fd: usize, buf: UserInPtr<u8>, len: usize) -> isize {
    assert!(len < CHUNK_SIZE);
    let pool = SyscallDataBuffer::get();
    let chunk_ptr = unsafe { pool.alloc_array_uninit::<u8>(len) };
    buf.read_buf(unsafe { from_raw_parts_mut(chunk_ptr as _, len) });
    let args = pool.alloc(ReadWriteArgs {
        fd: fd as _,
        buf_offset: pool.offset_of(chunk_ptr),
        len: len as _,
    });
    let cond = SyscallCondVar::new();
    send_request(
        ScfOpcode::Write,
        pool.offset_of(args),
        ScfRequestToken::from(&cond),
    );
    let ret = cond.wait();
    unsafe {
        pool.dealloc(chunk_ptr);
        pool.dealloc(args);
    }
    ret as _
}

pub fn sys_read(fd: usize, mut buf: UserOutPtr<u8>, len: usize) -> isize {
    assert!(len < CHUNK_SIZE);
    let pool = SyscallDataBuffer::get();
    let chunk_ptr = unsafe { pool.alloc_array_uninit::<u8>(len) };
    let args = pool.alloc(ReadWriteArgs {
        fd: fd as _,
        buf_offset: pool.offset_of(chunk_ptr),
        len: len as _,
    });
    let cond = SyscallCondVar::new();
    send_request(
        ScfOpcode::Read,
        pool.offset_of(args),
        ScfRequestToken::from(&cond),
    );
    let ret = cond.wait();
    unsafe {
        buf.write_buf(from_raw_parts(chunk_ptr as _, len));
        pool.dealloc(chunk_ptr);
        pool.dealloc(args);
    }
    ret as _
}
