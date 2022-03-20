mod manager;
mod percpu;
mod schedule;
mod structs;

pub use structs::{CurrentTask, TaskId};

use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};

use self::manager::TASK_MANAGER;
use self::structs::{Task, ROOT_TASK};
use crate::arch::instructions;

static TASK_INITED: AtomicBool = AtomicBool::new(false);

pub fn is_init() -> bool {
    TASK_INITED.load(Ordering::SeqCst)
}

pub fn init() {
    println!("Initialising task manager...");

    percpu::init_percpu();
    manager::init();

    ROOT_TASK.init_by(Task::new_kernel(
        |_| loop {
            let curr_task = CurrentTask::get();
            let mut exit_code = 0;
            while curr_task.waitpid(-1, &mut exit_code) > 0 {}
            if curr_task.children.lock().len() == 0 {
                instructions::wait_for_ints();
            } else {
                curr_task.yield_now();
            }
        },
        0,
    ));

    let test_kernel_task = |arg| {
        println!(
            "test kernel task: pid = {:?}, arg = {:#x}",
            CurrentTask::get().pid(),
            arg
        );
        0
    };

    let mut m = TASK_MANAGER.lock();
    m.spawn(ROOT_TASK.clone());
    m.spawn(Task::new_kernel(test_kernel_task, 0xdead));
    m.spawn(Task::new_kernel(test_kernel_task, 0xbeef));
    m.spawn(Task::new_user("user_shell"));

    TASK_INITED.store(true, Ordering::SeqCst);
}

pub fn spawn_task(task: Arc<Task>) {
    TASK_MANAGER.lock().spawn(task);
}

pub fn run() -> ! {
    println!("Running tasks...");
    instructions::enable_irqs();
    CurrentTask::get().yield_now(); // current task is idle at this time
    unreachable!("root task exit!");
}
