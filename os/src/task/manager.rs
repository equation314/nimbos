use super::{Task, TaskStatus};
use crate::config::MAX_APP_NUM;
use crate::loader;

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
        let kernel_task_count = 2;
        self.task_count = loader::get_app_count() + kernel_task_count;
        self.tasks[0].init_kernel(
            0,
            |arg| {
                println!("test kernel task 0: arg = {:#x}", arg);
                0
            },
            0xdead,
        );
        self.tasks[1].init_kernel(
            1,
            |arg| {
                println!("test kernel task 1: arg = {:#x}", arg);
                0
            },
            0xbeef,
        );
        for i in 0..self.task_count - kernel_task_count {
            let (entry, ustack_top) = loader::load_app(i);
            self.tasks[i + kernel_task_count].init_user(i + kernel_task_count, entry, ustack_top);
        }
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

    pub fn resched(&mut self) {
        let curr_task = super::current_task();
        assert!(curr_task.status != TaskStatus::Running);
        if let Some(next_task) = self.pick_next_task() {
            curr_task.switch_to(next_task);
        } else {
            panic!("All applications completed!");
        }
    }
}

pub(super) static mut TASK_MANAGER: TaskManager = TaskManager::new();
