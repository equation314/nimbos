use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::UnsafeCell;

use super::percpu::PerCpu;
use super::schedule::{Scheduler, SimpleScheduler};
use super::structs::{CurrentTask, Task, TaskState, ROOT_TASK};
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

    pub fn spawn(&mut self, t: Arc<Task>) {
        assert!(t.state.get() == TaskState::Ready);
        self.scheduler.add_ready_task(&t);
        self.tasks.push(t);
    }

    fn switch_to(&self, curr_task: &Arc<Task>, next_task: Arc<Task>) {
        next_task.state.set(TaskState::Running);
        if Arc::ptr_eq(curr_task, &next_task) {
            return;
        }

        let page_table_root = next_task.page_table_root.as_usize();
        let curr_ctx_ptr = curr_task.ctx.as_ptr();
        let next_ctx_ptr = next_task.ctx.as_ptr();

        // Decrement the strong reference count of `curr_task` and `next_task`,
        // but don't drop at here.
        assert!(Arc::strong_count(curr_task) > 1);
        assert!(Arc::strong_count(&next_task) > 1);
        PerCpu::current().set_current_task(next_task);

        unsafe {
            crate::arch::activate_paging(page_table_root, false);
            super::switch::context_switch(&mut *curr_ctx_ptr, &*next_ctx_ptr);
        }
    }

    fn resched(&mut self, curr_task: &CurrentTask) {
        assert!(curr_task.state.get() != TaskState::Running);
        if let Some(next_task) = self.scheduler.pick_next_task() {
            self.switch_to(curr_task, next_task);
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
        assert!(!curr_task.is_root());
        assert!(curr_task.state.get() == TaskState::Running);

        curr_task.state.set(TaskState::Zombie);
        curr_task.exit_code.set(exit_code);

        // Make all child tasks as the children of the root task
        for c in curr_task.children.lock().iter() {
            ROOT_TASK.add_child(c);
        }
        curr_task.children.lock().clear();

        self.resched(curr_task);
        unreachable!("task exited!");
    }

    pub fn clean_zombies(&mut self, curr_task: &CurrentTask) {
        let mut children = curr_task.children.lock();
        let old_len = children.len();
        children.retain(|t| t.state.get() != TaskState::Zombie);
        if children.len() < old_len {
            self.tasks.retain(|t| {
                if let Some(p) = t.parent.lock().upgrade() {
                    if p.id == curr_task.id && t.state.get() == TaskState::Zombie {
                        assert_eq!(Arc::strong_count(t), 1);
                        return false;
                    }
                }
                true
            });
        }
    }

    #[allow(unused)]
    pub fn dump(&self) {
        println!(
            "{:>4} {:>4} {:>6} {:>4}  STATE",
            "PID", "PPID", "#CHILD", "#REF",
        );
        for t in &self.tasks {
            let ref_count = Arc::strong_count(t);
            let children_count = t.children.lock().len();
            let state = t.state.get();
            if let Some(p) = t.parent.lock().upgrade() {
                println!(
                    "{:>4} {:>4} {:>6} {:>4}  {:?}",
                    t.id, p.id, children_count, ref_count, state
                );
            } else {
                println!(
                    "{:>4} {:>4} {:>6} {:>4}  {:?}",
                    t.id, '-', children_count, ref_count, state
                );
            };
        }
        println!();
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

    pub const fn as_ptr(&self) -> *mut T {
        self.data.get()
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
