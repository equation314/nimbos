mod manager;
mod switch;

use core::arch::asm;

use self::manager::TASK_MANAGER;
use self::switch::TaskContext;
use crate::config::KERNEL_STACK_SIZE;
use crate::loader::Stack;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

pub struct Task {
    id: usize,
    status: TaskStatus,
    ctx: TaskContext,
    kstack: Stack<KERNEL_STACK_SIZE>,
    entry: usize,
    args: usize,
}

impl Task {
    pub const fn uninit() -> Self {
        Self {
            id: 0,
            status: TaskStatus::UnInit,
            ctx: TaskContext::default(),
            kstack: Stack::default(),
            entry: 0,
            args: 0,
        }
    }

    pub fn init_kernel(&mut self, id: usize, entry: fn(usize) -> usize, args: usize) {
        self.id = id;
        self.entry = entry as _;
        self.args = args;
        self.ctx.init(task_start_fn as _, self.kstack.top());
        self.status = TaskStatus::Ready;
    }

    pub fn switch_to(&mut self, new_task: &mut Self) {
        new_task.status = TaskStatus::Running;
        set_current_task(new_task);
        unsafe { switch::context_switch(&mut self.ctx, &new_task.ctx) };
    }

    pub fn yield_now(&mut self) {
        self.status = TaskStatus::Ready;
        resched();
    }

    pub fn exit(&mut self) -> ! {
        self.status = TaskStatus::Exited;
        resched();
        unreachable!("Task exited!");
    }
}

fn task_start_fn() -> ! {
    let task = current_task();
    let entry: fn(usize) -> usize = unsafe { core::mem::transmute(task.entry) };
    entry(task.args);
    task.exit();
}

pub fn set_current_task(t: &Task) {
    unsafe { asm!("msr tpidr_el1, {}", in(reg) t) }
}

pub fn current_task() -> &'static mut Task {
    let addr: usize;
    unsafe {
        asm!("mrs {}, tpidr_el1", out(reg) addr);
        &mut *(addr as *mut Task)
    }
}

pub fn resched() {
    let curr_task = current_task();
    assert!(curr_task.status != TaskStatus::Running);
    if let Some(next_task) = unsafe { TASK_MANAGER.pick_next_task() } {
        curr_task.switch_to(next_task);
    } else {
        panic!("All applications completed!");
    }
}

pub fn init() {
    unsafe { TASK_MANAGER.init() }
}

pub fn run() -> ! {
    let idle = Task::uninit();
    set_current_task(&idle);
    resched();
    panic!("No tasks to run!");
}
