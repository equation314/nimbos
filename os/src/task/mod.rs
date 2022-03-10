mod manager;
mod percpu;
mod schedule;
mod structs;
mod switch;

pub use structs::CurrentTask;

use alloc::sync::Arc;

use self::manager::TASK_MANAGER;
use self::structs::{Task, ROOT_TASK};
use crate::loader;

pub fn init() {
    percpu::init_percpu();
    manager::init();

    ROOT_TASK.init_by(Task::new_kernel(
        |_| loop {
            let curr_task = CurrentTask::get();
            TASK_MANAGER.lock().clean_zombies(&curr_task);
            if curr_task.children.lock().len() == 0 {
                crate::arch::wait_for_ints();
            } else {
                curr_task.yield_now();
            }
        },
        0,
    ));

    let test_kernel_task = |arg| {
        println!(
            "test kernel task: pid = {}, arg = {:#x}",
            CurrentTask::get().pid(),
            arg
        );
        0
    };

    let mut m = TASK_MANAGER.lock();
    m.spawn(ROOT_TASK.clone());
    m.spawn(Task::new_kernel(test_kernel_task, 0xdead));
    m.spawn(Task::new_kernel(test_kernel_task, 0xbeef));
    for i in 0..loader::get_app_count() {
        let (entry, ustack_top, ms) = loader::load_app(i);
        m.spawn(Task::new_user(entry, ustack_top, ms));
    }
}

pub fn spawn_task(task: Arc<Task>) {
    TASK_MANAGER.lock().spawn(task);
}

pub fn run() -> ! {
    crate::arch::enable_irqs();
    CurrentTask::get().yield_now(); // current task is idle at this time
    unreachable!("root task exit!");
}
