const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;

mod fs;
mod process;

use self::fs::*;
use self::process::*;
use crate::arch;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    arch::enable_irqs();
    let ret = match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(),
        _ => {
            println!("Unsupported syscall_id: {}", syscall_id);
            crate::task::CurrentTask::exit(-1);
        }
    };
    arch::disable_irqs();
    ret
}
