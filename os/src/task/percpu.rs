use alloc::sync::Arc;
use core::cell::Cell;

use super::structs::{CurrentTask, Task};
use crate::config::MAX_CPUS;
use crate::sync::LazyInit;

static CPUS: [LazyInit<PerCpu>; MAX_CPUS] = [LazyInit::new(); MAX_CPUS];

/// Each CPU can only accesses its own `PerCpu` instance.
pub struct PerCpu {
    _id: usize,
    current_task: Cell<Arc<Task>>,
    idle_task: Arc<Task>,
}

unsafe impl Sync for PerCpu {}

impl PerCpu {
    fn new(id: usize) -> Self {
        let idle_task = Arc::new(Task::new_idle());
        Self {
            _id: id,
            current_task: Cell::new(idle_task.clone()),
            idle_task,
        }
    }

    pub fn current<'a>() -> &'a Self {
        unsafe { &*(crate::arch::thread_pointer() as *const Self) }
    }

    pub fn idle_task<'a>() -> &'a Arc<Task> {
        &Self::current().idle_task
    }

    pub fn current_task(&self) -> CurrentTask {
        unsafe { CurrentTask::from((*self.current_task.as_ptr()).clone()) }
    }

    pub fn set_current_task(&self, task: Arc<Task>) {
        assert!(crate::arch::irqs_disabled());
        self.current_task.set(task);
    }
}

pub(super) fn init_percpu() {
    let cpu_id = 0;
    CPUS[cpu_id].init_by(PerCpu::new(cpu_id));
    unsafe { crate::arch::set_thread_pointer(&*CPUS[cpu_id] as *const PerCpu as usize) };
}
