mod manager;
mod percpu;
mod schedule;
mod switch;

use alloc::{boxed::Box, sync::Arc};

use self::manager::{TaskLockedCell, TASK_MANAGER};
use self::percpu::PerCpu;
use self::switch::TaskContext;
use crate::arch;
use crate::config::KERNEL_STACK_SIZE;
use crate::loader::{self, Stack};
use crate::mm::MemorySet;
use crate::trap::TrapFrame;

#[derive(Debug, Copy, Clone, PartialEq)]
enum TaskState {
    Ready,
    Running,
    Exited,
}

#[derive(Debug, Default)]
struct EntryState {
    pc: usize,
    sp: usize,
    arg: usize,
}

pub struct Task {
    id: usize,
    memory_set: Option<MemorySet>,
    kstack: Box<Stack<KERNEL_STACK_SIZE>>,
    entry: EntryState,

    state: TaskLockedCell<TaskState>,
    ctx: TaskLockedCell<TaskContext>,
    exit_code: TaskLockedCell<i32>,
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

    pub fn new_kernel(id: usize, entry: fn(usize) -> usize, arg: usize) -> Self {
        assert!(id > 0);
        let mut t = Self::new_common(id);
        t.entry = EntryState {
            pc: entry as usize,
            arg,
            sp: 0,
        };
        t.ctx.get_mut().init(kernel_task_entry as _, t.kstack.top());
        t
    }

    pub fn new_user(id: usize, entry: usize, ustack_top: usize, ms: MemorySet) -> Self {
        assert!(id > 0);
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

fn kernel_task_entry() -> ! {
    // release the lock that was implicitly held across the reschedule
    unsafe { TASK_MANAGER.force_unlock() };
    arch::enable_irqs();
    let task = CurrentTask::get();
    let entry: fn(usize) -> i32 = unsafe { core::mem::transmute(task.entry.pc) };
    let ret = entry(task.entry.arg);
    task.exit(ret as _);
}

fn user_task_entry() -> ! {
    // release the lock that was implicitly held across the reschedule
    unsafe { TASK_MANAGER.force_unlock() };
    assert!(arch::irqs_disabled());
    let task = CurrentTask::get();
    let tf = TrapFrame::new_user(task.entry.pc, task.entry.sp);
    unsafe { tf.exec(task.kstack.top()) };
}

pub struct CurrentTask(Arc<Task>);

impl CurrentTask {
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

pub fn init() {
    percpu::init_percpu();
    manager::init();

    let mut m = TASK_MANAGER.lock();
    let kernel_task_count = 3;
    m.spawn(Task::new_kernel(
        1,
        |arg| {
            println!("test kernel task 0: arg = {:#x}", arg);
            0
        },
        0xdead,
    ));
    m.spawn(Task::new_kernel(
        2,
        |arg| {
            println!("test kernel task 1: arg = {:#x}", arg);
            0
        },
        0xbeef,
    ));
    for i in 0..loader::get_app_count() {
        let (entry, ustack_top, ms) = loader::load_app(i);
        m.spawn(Task::new_user(i + kernel_task_count, entry, ustack_top, ms));
    }
}

pub fn run() -> ! {
    arch::enable_irqs();
    CurrentTask::get().yield_now(); // current task is idle at this time
    println!("All applications completed!");
    println!("Waiting for interrupts...");
    loop {
        arch::wait_for_ints();
    }
}
