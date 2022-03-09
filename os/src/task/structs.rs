use alloc::sync::{Arc, Weak};
use alloc::{boxed::Box, vec::Vec};

use super::manager::{TaskLockedCell, TASK_MANAGER};
use super::percpu::PerCpu;
use super::switch::TaskContext;
use crate::config::KERNEL_STACK_SIZE;
use crate::mm::MemorySet;
use crate::sync::{Mutex, SpinNoIrqLock};
use crate::trap::TrapFrame;
use crate::utils::FreeListAllocator;

enum EntryState {
    Kernel { pc: usize, arg: usize },
    User(Box<TrapFrame>),
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

    parent: Mutex<Weak<Task>>,
    children: Mutex<Vec<Arc<Task>>>,
}

lazy_static::lazy_static! {
    static ref PID_ALLOCATOR: SpinNoIrqLock<FreeListAllocator> = SpinNoIrqLock::new(FreeListAllocator::new(1..65536));
}

impl Task {
    fn new_common(id: isize) -> Self {
        let id = if id < 0 {
            PID_ALLOCATOR.lock().alloc().expect("Too many tasks!")
        } else {
            id as usize
        };
        Self {
            id,
            memory_set: None,
            kstack: Box::new(Stack::default()),
            entry: EntryState::Kernel { pc: 0, arg: 0 },

            state: TaskLockedCell::new(TaskState::Ready),
            ctx: TaskLockedCell::new(TaskContext::default()),
            exit_code: TaskLockedCell::new(0),

            parent: Mutex::new(Weak::default()),
            children: Mutex::new(Vec::new()),
        }
    }

    pub fn new_idle() -> Arc<Self> {
        let mut t = Self::new_common(0);
        *t.state.get_mut() = TaskState::Running;
        Arc::new(t)
    }

    pub fn new_kernel(entry: fn(usize) -> usize, arg: usize) -> Arc<Self> {
        let mut t = Self::new_common(-1);
        t.entry = EntryState::Kernel {
            pc: entry as usize,
            arg,
        };
        t.ctx.get_mut().init(task_entry as _, t.kstack.top());
        Arc::new(t)
    }

    pub fn new_user(entry: usize, ustack_top: usize, ms: MemorySet) -> Arc<Self> {
        let mut t = Self::new_common(-1);
        t.memory_set = Some(ms);
        t.entry = EntryState::User(Box::new(TrapFrame::new_user(entry, ustack_top)));
        t.ctx.get_mut().init(task_entry as _, t.kstack.top());
        Arc::new(t)
    }

    pub fn new_fork(self: &Arc<Self>, tf: &TrapFrame) -> Arc<Self> {
        assert!(!self.is_kernel_task());
        let mut t = Self::new_common(-1);
        t.memory_set = self.memory_set.clone();
        t.entry = EntryState::User(Box::new(tf.new_fork()));
        t.ctx.get_mut().init(task_entry as _, t.kstack.top());

        *t.parent.lock() = Arc::downgrade(self);
        let t = Arc::new(t);
        self.children.lock().push(t.clone());
        t
    }

    pub fn pid(&self) -> usize {
        self.id
    }

    pub fn is_kernel_task(&self) -> bool {
        self.memory_set.is_none()
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

fn task_entry() -> ! {
    // release the lock that was implicitly held across the reschedule
    unsafe { TASK_MANAGER.force_unlock() };
    crate::arch::enable_irqs();
    let task = CurrentTask::get();
    match &task.entry {
        EntryState::Kernel { pc, arg } => {
            let entry: fn(usize) -> i32 = unsafe { core::mem::transmute(*pc) };
            let ret = entry(*arg);
            task.exit(ret as _);
        }
        EntryState::User(tf) => {
            unsafe { tf.exec(task.kstack.top()) };
        }
    }
}

pub struct CurrentTask(Arc<Task>);

impl CurrentTask {
    pub(super) fn from(task: Arc<Task>) -> Self {
        Self(task)
    }

    pub fn get() -> Self {
        PerCpu::current().current_task()
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
