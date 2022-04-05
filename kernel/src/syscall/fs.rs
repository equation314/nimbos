#![allow(dead_code)]
#![allow(unused_imports)]

use crate::drivers::uart::console_getchar;
use crate::mm::{UserInPtr, UserOutPtr};
use crate::task::CurrentTask;

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;
const FD_STDERR: usize = 2;
const CHUNK_SIZE: usize = 256;

#[cfg(not(feature = "rvm"))]
pub fn sys_write(fd: usize, buf: UserInPtr<u8>, len: usize) -> isize {
    match fd {
        FD_STDOUT | FD_STDERR => {
            let mut count = 0;
            while count < len {
                let chunk_len = CHUNK_SIZE.min(len);
                let chunk: [u8; CHUNK_SIZE] = unsafe { buf.add(count).read_array(chunk_len) };
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

#[cfg(not(feature = "rvm"))]
pub fn sys_read(fd: usize, mut buf: UserOutPtr<u8>, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            assert_eq!(len, 1, "Only support len = 1 in sys_read!");
            loop {
                if let Some(c) = console_getchar() {
                    buf.write(c);
                    return 1;
                } else {
                    CurrentTask::get().yield_now();
                }
            }
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}
