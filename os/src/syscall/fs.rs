use crate::mm::UserInPtr;

const FD_STDOUT: usize = 1;
const CHUNK_SIZE: usize = 256;

pub fn sys_write(fd: usize, buf: UserInPtr<u8>, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
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
