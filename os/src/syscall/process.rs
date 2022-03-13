use crate::mm::{UserInPtr, UserOutPtr};
use crate::task::{spawn_task, CurrentTask};
use crate::timer::get_time_ms;
use crate::trap::TrapFrame;

const MAX_STR_LEN: usize = 256;

pub fn sys_exit(exit_code: i32) -> ! {
    CurrentTask::get().exit(exit_code);
}

pub fn sys_yield() -> isize {
    CurrentTask::get().yield_now();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_getpid() -> isize {
    CurrentTask::get().pid().as_usize() as isize
}

pub fn sys_fork(tf: &TrapFrame) -> isize {
    let new_task = CurrentTask::get().new_fork(tf);
    let pid = new_task.pid().as_usize() as isize;
    spawn_task(new_task);
    pid
}

pub fn sys_exec(path: UserInPtr<u8>, tf: &mut TrapFrame) -> isize {
    let (path_buf, len) = path.read_str::<MAX_STR_LEN>();
    let path = core::str::from_utf8(&path_buf[..len]).unwrap();
    CurrentTask::get().exec(path, tf)
}

/// If there is no child process has the same pid as the given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, mut exit_code_ptr: UserOutPtr<i32>) -> isize {
    let mut exit_code = 0;
    let ret = CurrentTask::get().waitpid(pid, &mut exit_code);
    exit_code_ptr.write(exit_code);
    ret
}
