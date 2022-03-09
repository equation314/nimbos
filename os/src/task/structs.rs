use alloc::{boxed::Box, sync::Arc};

use super::manager::{TaskLockedCell, TASK_MANAGER};
use super::percpu::PerCpu;
use super::switch::TaskContext;
use crate::config::KERNEL_STACK_SIZE;
use crate::mm::MemorySet;
use crate::sync::SpinNoIrqLock;
use crate::trap::TrapFrame;
use crate::utils::FreeListAllocator;

#[derive(Debug, Default)]
struct EntryState {
    pc: usize,
    sp: usize,
    arg: usize,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TaskState {
    Ready,
    Running,
    Exited,
}

pub struct Task {
    pub id: usize,
    pub memory_set: Option<MemorySet>,
    kstack: Box<Stack<KERNEL_STACK_SIZE>>,
    entry: EntryState,

    pub state: TaskLockedCell<TaskState>,
    pub ctx: TaskLockedCell<TaskContext>,
    pub exit_code: TaskLockedCell<i32>,
}

lazy_static::lazy_static! {
    static ref PID_ALLOCATOR: SpinNoIrqLock<FreeListAllocator> = SpinNoIrqLock::new(FreeListAllocator::new(1..65536));
}

impl Task {
    fn new_common(id: usize) -> Self {
        Self {
            id,
            memory_set: None,
            kstack: Box::new(Stack::default()),
            entry: EntryState::default(),
            state: TaskLockedCell::new(TaskState::Ready),
            ctx: TaskLockedCell::new(TaskContext::default()),
            exit_code: TaskLockedCell::new(0),
        }
    }

    pub fn new_idle() -> Self {
        let mut t = Self::new_common(0);
        *t.state.get_mut() = TaskState::Running;
        t
    }

    pub fn new_kernel(entry: fn(usize) -> usize, arg: usize) -> Self {
        let id = PID_ALLOCATOR.lock().alloc().expect("Too many tasks!");
        let mut t = Self::new_common(id);
        t.entry = EntryState {
            pc: entry as usize,
            arg,
            sp: 0,
        };
        t.ctx.get_mut().init(kernel_task_entry as _, t.kstack.top());
        t
    }

    pub fn new_user(entry: usize, ustack_top: usize, ms: MemorySet) -> Self {
        let id = PID_ALLOCATOR.lock().alloc().expect("Too many tasks!");
        let mut t = Self::new_common(id);
        t.memory_set = Some(ms);
        t.entry = EntryState {
            pc: entry as usize,
            arg: 0,
            sp: ustack_top,
        };
        t.ctx.get_mut().init(user_task_entry as _, t.kstack.top());
        t
    }

    pub fn is_idle(&self) -> bool {
        self.id == 0
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        if self.id > 0 {
            PID_ALLOCATOR.lock().dealloc(self.id);
        }
    }
}

fn kernel_task_entry() -> ! {
    // release the lock that was implicitly held across the reschedule
    unsafe { TASK_MANAGER.force_unlock() };
    crate::arch::enable_irqs();
    let task = CurrentTask::get();
    let entry: fn(usize) -> i32 = unsafe { core::mem::transmute(task.entry.pc) };
    let ret = entry(task.entry.arg);
    task.exit(ret as _);
}

fn user_task_entry() -> ! {
    // release the lock that was implicitly held across the reschedule
    unsafe { TASK_MANAGER.force_unlock() };
    assert!(crate::arch::irqs_disabled());
    let task = CurrentTask::get();
    let tf = TrapFrame::new_user(task.entry.pc, task.entry.sp);
    unsafe { tf.exec(task.kstack.top()) };
}

pub struct CurrentTask(Arc<Task>);

impl CurrentTask {
    pub(super) fn from(task: Arc<Task>) -> Self {
        Self(task)
    }

    pub fn get() -> Self {
        PerCpu::current().current_task()
    }

    pub fn pid(&self) -> usize {
        self.0.id
    }

    pub fn yield_now(&self) {
        TASK_MANAGER.lock().yield_current(self)
    }

    pub fn exit(&self, exit_code: i32) -> ! {
        TASK_MANAGER.lock().exit_current(self, exit_code)
    }
}

impl core::ops::Deref for CurrentTask {
    type Target = Arc<Task>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[repr(align(4096))]
struct Stack<const N: usize>([u8; N]);

impl<const N: usize> Stack<N> {
    pub const fn default() -> Self {
        Self([0; N])
    }

    pub fn top(&self) -> usize {
        self.0.as_ptr_range().end as usize
    }
}
