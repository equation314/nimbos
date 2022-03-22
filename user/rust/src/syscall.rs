use super::TimeSpec;
use crate::arch::syscall;

pub use crate::arch::sys_clone;

pub const SYSCALL_READ: usize = 0;
pub const SYSCALL_WRITE: usize = 1;
pub const SYSCALL_YIELD: usize = 24;
pub const SYSCALL_NANOSLEEP: usize = 35;
pub const SYSCALL_GETPID: usize = 39;
pub const SYSCALL_CLONE: usize = 56;
pub const SYSCALL_FORK: usize = 57;
pub const SYSCALL_EXEC: usize = 59;
pub const SYSCALL_EXIT: usize = 60;
pub const SYSCALL_WAITPID: usize = 61;
pub const SYSCALL_GET_TIME: usize = 96;

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(
        SYSCALL_READ,
        [fd, buffer.as_mut_ptr() as usize, buffer.len()],
    )
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(exit_code: i32) -> ! {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0]);
    panic!("sys_exit never returns!");
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0])
}

pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0])
}

pub fn sys_exec(path: &str) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, 0, 0])
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0])
}

pub fn sys_nanosleep(req: &TimeSpec) -> isize {
    syscall(SYSCALL_NANOSLEEP, [req as *const _ as usize, 0, 0])
}
