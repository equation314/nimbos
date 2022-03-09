use crate::task::CurrentTask;
use crate::timer::get_time_ms;

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
    CurrentTask::get().pid() as isize
}

pub fn sys_fork() -> isize {
    unimplemented!()
}

pub fn sys_exec(_path: *const u8) -> isize {
    unimplemented!()
}

pub fn sys_waitpid(_pid: isize, _exit_code_ptr: *mut i32) -> isize {
    unimplemented!()
}
