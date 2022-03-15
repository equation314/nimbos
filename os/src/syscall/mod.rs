const SYSCALL_READ: usize = 0;
const SYSCALL_WRITE: usize = 1;
const SYSCALL_YIELD: usize = 24;
const SYSCALL_GETPID: usize = 39;
const SYSCALL_CLONE: usize = 56;
const SYSCALL_FORK: usize = 57;
const SYSCALL_EXEC: usize = 59;
const SYSCALL_EXIT: usize = 60;
const SYSCALL_WAITPID: usize = 61;
const SYSCALL_GET_TIME: usize = 96;

mod fs;
mod task;

use self::fs::*;
use self::task::*;
use crate::arch;
use crate::trap::TrapFrame;

pub fn syscall(syscall_id: usize, args: [usize; 3], tf: &mut TrapFrame) -> isize {
    arch::enable_irqs();
    let ret = match syscall_id {
        SYSCALL_READ => sys_read(args[0], args[1].into(), args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1].into(), args[2]),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_CLONE => sys_clone(args[0], tf),
        SYSCALL_FORK => sys_fork(tf),
        SYSCALL_EXEC => sys_exec(args[0].into(), tf),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1].into()),
        SYSCALL_GET_TIME => sys_get_time(),
        _ => {
            println!("Unsupported syscall_id: {}", syscall_id);
            crate::task::CurrentTask::get().exit(-1);
        }
    };
    arch::disable_irqs();
    ret
}
