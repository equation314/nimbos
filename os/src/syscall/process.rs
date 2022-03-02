use crate::task::CurrentTask;
use crate::timer::get_time_ms;

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    CurrentTask::exit(exit_code);
}

pub fn sys_yield() -> isize {
    CurrentTask::yield_now();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}
