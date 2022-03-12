use core::mem::MaybeUninit;

use crate::mm::copy_from_user;

const FD_STDOUT: usize = 1;
const CHUNK_SIZE: usize = 256;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let mut count = 0;
            let mut chunk: MaybeUninit<[u8; CHUNK_SIZE]> = MaybeUninit::uninit();
            while count < len {
                let chunk_len = CHUNK_SIZE.min(len);
                unsafe { copy_from_user(chunk.as_mut_ptr() as *mut u8, buf.add(count), chunk_len) };
                let chunk = unsafe { chunk.assume_init() };
                print!("{}", core::str::from_utf8(&chunk[..chunk_len]).unwrap());
                count += chunk_len;
            }
            count as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
