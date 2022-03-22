const SYSCALL_READ: usize = 0;
const SYSCALL_WRITE: usize = 1;
const SYSCALL_YIELD: usize = 24;
const SYSCALL_NANOSLEEP: usize = 35;
const SYSCALL_GETPID: usize = 39;
const SYSCALL_CLONE: usize = 56;
const SYSCALL_FORK: usize = 57;
const SYSCALL_EXEC: usize = 59;
const SYSCALL_EXIT: usize = 60;
const SYSCALL_WAITPID: usize = 61;
const SYSCALL_GET_TIME_MS: usize = 96;
const SYSCALL_CLOCK_GETTIME: usize = 228;

mod fs;
mod task;
mod time;

use self::fs::*;
use self::task::*;
use self::time::*;
use crate::arch::{instructions, TrapFrame};

pub fn syscall(syscall_id: usize, args: [usize; 3], tf: &mut TrapFrame) -> isize {
    instructions::enable_irqs();
    debug!(
        "syscall {} enter <= ({:#x}, {:#x}, {:#x})",
        syscall_id, args[0], args[1], args[2]
    );
    let ret = match syscall_id {
        SYSCALL_READ => sys_read(args[0], args[1].into(), args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1].into(), args[2]),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_NANOSLEEP => sys_nanosleep(args[0].into()),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_CLONE => sys_clone(args[0], tf),
        SYSCALL_FORK => sys_fork(tf),
        SYSCALL_EXEC => sys_exec(args[0].into(), tf),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1].into()),
        SYSCALL_GET_TIME_MS => sys_get_time_ms(),
        SYSCALL_CLOCK_GETTIME => sys_clock_gettime(args[0], args[1].into()),
        _ => {
            println!("Unsupported syscall_id: {}", syscall_id);
            crate::task::CurrentTask::get().exit(-1);
        }
    };
    debug!("syscall {} ret => {:#x}", syscall_id, ret);
    instructions::disable_irqs();
    ret
}
