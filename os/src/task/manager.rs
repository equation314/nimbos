use alloc::vec::Vec;

use super::{CurrentTask, Task, TaskStatus};
use crate::sync::SpinNoIrqLock;

pub struct TaskManager {
    tasks: Vec<Task>,
}

impl TaskManager {
    pub const fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn add_task(&mut self, t: Task) {
        self.tasks.push(t);
    }

    fn pick_next_task(&mut self) -> Option<&mut Task> {
        let current_task = CurrentTask::get();
        let start = if current_task.status == TaskStatus::UnInit {
            0
        } else {
            current_task.id + 1
        };
        let n = self.tasks.len();
        for i in 0..n {
            let id = (start + i) % n;
            if self.tasks[id].status == TaskStatus::Ready {
                return Some(&mut self.tasks[id]);
            }
        }
        None
    }

    pub fn resched(&mut self) {
        let curr_task = CurrentTask::get_mut();
        assert!(crate::arch::irqs_disabled());
        assert!(curr_task.status != TaskStatus::Running);
        if let Some(next_task) = self.pick_next_task() {
            curr_task.switch_to(next_task);
        } else {
            panic!("All applications completed!");
        }
    }

    pub fn yield_current(&mut self) {
        let curr_task = CurrentTask::get_mut();
        assert!(curr_task.status == TaskStatus::Running);
        curr_task.status = TaskStatus::Ready;
        self.resched();
    }

    pub fn exit_current(&mut self, _exit_code: i32) -> ! {
        let curr_task = CurrentTask::get_mut();
        curr_task.status = TaskStatus::Exited;
        self.resched();
        panic!("task exited!");
        // TODO: remove from self.tasks
    }
}

pub(super) static TASK_MANAGER: SpinNoIrqLock<TaskManager> = SpinNoIrqLock::new(TaskManager::new());
