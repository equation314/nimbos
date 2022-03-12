use crate::mm::copy_to_user;
use crate::task::{spawn_task, CurrentTask};
use crate::timer::get_time_ms;
use crate::trap::TrapFrame;

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

pub fn sys_exec(_path: *const u8) -> isize {
    unimplemented!()
}

/// If there is no child process has the same pid as the given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let mut exit_code = 0;
    let ret = CurrentTask::get().waitpid(pid, &mut exit_code);
    unsafe { copy_to_user(exit_code_ptr, &exit_code as _, 1) };
    ret
}
