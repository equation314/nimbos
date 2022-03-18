use alloc::sync::Arc;
use core::cell::UnsafeCell;

use super::structs::Task;
use crate::arch::instructions;
use crate::config::MAX_CPUS;
use crate::sync::LazyInit;

static CPUS: [LazyInit<PerCpu>; MAX_CPUS] = [LazyInit::new(); MAX_CPUS];

/// Each CPU can only accesses its own `PerCpu` instance.
#[repr(C)]
pub struct PerCpu {
    self_vaddr: usize,
    _id: usize,
    current_task: UnsafeCell<Arc<Task>>,
    idle_task: Arc<Task>,
}

unsafe impl Sync for PerCpu {}

impl PerCpu {
    fn new(id: usize) -> Self {
        let idle_task = Task::new_idle();
        Self {
            self_vaddr: &CPUS[id] as *const _ as usize,
            _id: id,
            current_task: UnsafeCell::new(idle_task.clone()),
            idle_task,
        }
    }

    pub fn current<'a>() -> &'a Self {
        unsafe { &*(instructions::thread_pointer() as *const Self) }
    }

    pub fn idle_task<'a>() -> &'a Arc<Task> {
        &Self::current().idle_task
    }

    pub fn current_task<'a>(&self) -> &'a Arc<Task> {
        // Safety: Even if there is an interrupt and task preemption after
        // calling this method, the reference of `current_task` can keep unchanged
        // since it will be restored after context switches.
        unsafe { &*self.current_task.get() }
    }

    pub fn set_current_task(&self, task: Arc<Task>) {
        // We must disable interrupts and task preemption.
        assert!(instructions::irqs_disabled());
        let old_task = core::mem::replace(unsafe { &mut *self.current_task.get() }, task);
        drop(old_task)
    }
}

pub(super) fn init_percpu() {
    let cpu_id = 0;
    CPUS[cpu_id].init_by(PerCpu::new(cpu_id));
    unsafe { instructions::set_thread_pointer(CPUS[cpu_id].self_vaddr) };
}
