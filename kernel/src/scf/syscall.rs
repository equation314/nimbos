use super::{allocator::SyscallDataBuffer, send_request};
use crate::mm::UserInPtr;

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

const CHUNK_SIZE: usize = 256;

#[repr(C)]
#[derive(Debug)]
struct ReadWriteArgs {
    fd: u32,
    buf_offset: u64,
    len: u64,
}

pub fn sys_write(fd: usize, buf: UserInPtr<u8>, len: usize) -> isize {
    assert!(len < CHUNK_SIZE);
    let pool = SyscallDataBuffer::get();
    let chunk_ptr = unsafe { pool.alloc_array_uninit::<u8>(len) };
    unsafe { buf.read_ptr(chunk_ptr as _, len) };
    let args = pool.alloc(ReadWriteArgs {
        fd: fd as _,
        buf_offset: pool.offset_of(chunk_ptr),
        len: len as _,
    });
    info!("sys_write {:#x?} {:#x?}", chunk_ptr, unsafe { &*args });
    send_request(IpcOpcode::Write, pool.offset_of(args));
    len as _
}
