use super::{data_pool, send_request, IpcOpcode};
use crate::mm::UserInPtr;

const CHUNK_SIZE: usize = 256;

#[repr(C)]
struct ReadWriteArgs {
    fd: u32,
    buf_offset: u64,
    len: u64,
}

pub fn sys_write(fd: usize, buf: UserInPtr<u8>, len: usize) -> isize {
    assert!(len < CHUNK_SIZE);
    let chunk_ptr = unsafe { data_pool::alloc_array_uninit::<u8>(len) };
    unsafe { buf.read_ptr(chunk_ptr as _, len) };
    let args = data_pool::alloc(ReadWriteArgs {
        fd: fd as _,
        buf_offset: data_pool::as_offset(chunk_ptr),
        len: len as _,
    });
    send_request(IpcOpcode::Write, data_pool::as_offset(args));
    len as _
}
