use super::{Task, TaskStatus};
use crate::config::MAX_APP_NUM;

pub struct TaskManager {
    task_count: usize,
    tasks: [Task; MAX_APP_NUM],
}

impl TaskManager {
    pub const fn new() -> Self {
        const T: Task = Task::uninit();
        Self {
            task_count: 0,
            tasks: [T; MAX_APP_NUM],
        }
    }

    pub fn init(&mut self) {
        self.task_count = 2;
        self.tasks[0].init_kernel(
            0,
            |args| {
                println!("test kernel task 0: arg = {:#x}", args);
                0
            },
            0xdead,
        );
        self.tasks[1].init_kernel(
            1,
            |args| {
                println!("test kernel task 1: arg = {:#x}", args);
                0
            },
            0xbeef,
        )
    }

    pub fn pick_next_task(&mut self) -> Option<&mut Task> {
        let current_task = super::current_task();
        let start = if current_task.status == TaskStatus::UnInit {
            0
        } else {
            current_task.id + 1
        };
        for i in 0..self.task_count {
            let id = (start + i) % self.task_count;
            if self.tasks[id].status == TaskStatus::Ready {
                return Some(&mut self.tasks[id]);
            }
        }
        None
    }
}

pub(super) static mut TASK_MANAGER: TaskManager = TaskManager::new();
