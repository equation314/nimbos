mod manager;
mod switch;

use core::arch::asm;

use self::manager::TASK_MANAGER;
use self::switch::TaskContext;
use crate::config::KERNEL_STACK_SIZE;
use crate::loader::Stack;
use crate::trap::TrapFrame;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

#[derive(Debug)]
struct TaskEntryStatus {
    pc: usize,
    sp: usize,
    arg: usize,
}

pub struct Task {
    id: usize,
    status: TaskStatus,
    ctx: TaskContext,
    kstack: Stack<KERNEL_STACK_SIZE>,
    entry: TaskEntryStatus,
}

impl Task {
    pub const fn uninit() -> Self {
        Self {
            id: 0,
            status: TaskStatus::UnInit,
            ctx: TaskContext::default(),
            kstack: Stack::default(),
            entry: TaskEntryStatus {
                pc: 0,
                sp: 0,
                arg: 0,
            },
        }
    }

    pub fn init_kernel(&mut self, id: usize, entry: fn(usize) -> usize, arg: usize) {
        self.id = id;
        self.entry = TaskEntryStatus {
            pc: entry as usize,
            arg,
            sp: 0,
        };
        self.ctx.init(start_kernel_task as _, self.kstack.top());
        self.status = TaskStatus::Ready;
    }

    pub fn init_user(&mut self, id: usize, entry: usize, ustack_top: usize) {
        self.id = id;
        self.entry = TaskEntryStatus {
            pc: entry as usize,
            arg: 0,
            sp: ustack_top,
        };
        self.ctx.init(start_user_task as _, self.kstack.top());
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
        unreachable!("task exited!");
    }
}

fn start_kernel_task() -> ! {
    let task = current_task();
    let entry: fn(usize) -> usize = unsafe { core::mem::transmute(task.entry.pc) };
    entry(task.entry.arg);
    task.exit();
}

fn start_user_task() -> ! {
    let task = current_task();
    let tf = TrapFrame::new_user(task.entry.pc, task.entry.sp);
    unsafe { tf.exec(task.kstack.top()) };
}

fn resched() {
    unsafe { TASK_MANAGER.resched() }
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

pub fn init() {
    unsafe { TASK_MANAGER.init() }
}

pub fn run() -> ! {
    let idle = Task::uninit();
    set_current_task(&idle);
    resched();
    panic!("No tasks to run!");
}
