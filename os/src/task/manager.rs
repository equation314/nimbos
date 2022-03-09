use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::UnsafeCell;

use super::percpu::PerCpu;
use super::schedule::{Scheduler, SimpleScheduler};
use super::structs::{CurrentTask, Task, TaskState};
use crate::sync::{LazyInit, SpinNoIrqLock};

pub struct TaskManager<S: Scheduler> {
    tasks: Vec<Arc<Task>>,
    scheduler: S,
}

impl<S: Scheduler> TaskManager<S> {
    fn new(scheduler: S) -> Self {
        Self {
            tasks: Vec::new(),
            scheduler,
        }
    }

    pub fn spawn(&mut self, t: Task) {
        assert!(t.state.get() == TaskState::Ready);
        let t = Arc::new(t);
        self.scheduler.add_ready_task(&t);
        self.tasks.push(t);
    }

    fn switch_to(&self, curr_task: &Arc<Task>, next_task: &Arc<Task>) {
        next_task.state.set(TaskState::Running);
        if Arc::ptr_eq(curr_task, next_task) {
            return;
        }
        PerCpu::current().set_current_task(next_task.clone());
        unsafe {
            if let Some(ms) = &next_task.memory_set {
                // for user task, set TTBR0 to the user page table root
                ms.activate(false);
            } else {
                // for kernel task, disable TTBR0 translation
                crate::arch::activate_paging(0, false);
            }
            super::switch::context_switch(curr_task.ctx.as_mut(), next_task.ctx.as_ref());
        }
    }

    fn resched(&mut self, curr_task: &CurrentTask) {
        assert!(curr_task.state.get() != TaskState::Running);
        if let Some(next_task) = self.scheduler.pick_next_task() {
            self.switch_to(curr_task, &next_task);
        } else {
            self.switch_to(curr_task, PerCpu::idle_task());
        }
    }

    pub fn yield_current(&mut self, curr_task: &CurrentTask) {
        assert!(curr_task.state.get() == TaskState::Running);
        curr_task.state.set(TaskState::Ready);
        if !curr_task.is_idle() {
            self.scheduler.add_ready_task(curr_task);
        }
        self.resched(curr_task);
    }

    pub fn exit_current(&mut self, curr_task: &CurrentTask, exit_code: i32) -> ! {
        assert!(!curr_task.is_idle());
        assert!(curr_task.state.get() == TaskState::Running);
        curr_task.state.set(TaskState::Exited);
        curr_task.exit_code.set(exit_code);
        self.tasks.retain(|t| t.id != curr_task.id);
        self.resched(curr_task);
        unreachable!("task exited!");
    }
}

/// A wrapper structure which can only be accessed while holding the lock of `TASK_MANAGER`.
pub struct TaskLockedCell<T> {
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for TaskLockedCell<T> {}

impl<T> TaskLockedCell<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }

    pub fn as_ref(&self) -> &T {
        assert!(TASK_MANAGER.is_locked());
        assert!(crate::arch::irqs_disabled());
        unsafe { &*self.data.get() }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn as_mut(&self) -> &mut T {
        assert!(TASK_MANAGER.is_locked());
        assert!(crate::arch::irqs_disabled());
        unsafe { &mut *self.data.get() }
    }
}

impl<T: Copy> TaskLockedCell<T> {
    pub fn get(&self) -> T {
        *self.as_ref()
    }

    pub fn set(&self, val: T) {
        *self.as_mut() = val;
    }
}

pub(super) static TASK_MANAGER: LazyInit<SpinNoIrqLock<TaskManager<SimpleScheduler>>> =
    LazyInit::new();

pub(super) fn init() {
    TASK_MANAGER.init_by(SpinNoIrqLock::new(TaskManager::new(SimpleScheduler::new())));
}
