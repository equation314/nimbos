use crate::task::current_task;

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    current_task().exit();
}

pub fn sys_yield() -> isize {
    current_task().yield_now();
    0
}
