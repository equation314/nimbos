mod manager;
mod switch;

use self::manager::TASK_MANAGER;
use self::switch::TaskContext;
use crate::arch;
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
        self.ctx.init(kernel_task_entry as _, self.kstack.top());
        self.status = TaskStatus::Ready;
    }

    pub fn init_user(&mut self, id: usize, entry: usize, ustack_top: usize) {
        self.id = id;
        self.entry = TaskEntryStatus {
            pc: entry as usize,
            arg: 0,
            sp: ustack_top,
        };
        self.ctx.init(user_task_entry as _, self.kstack.top());
        self.status = TaskStatus::Ready;
    }
}

fn kernel_task_entry() -> ! {
    // release the lock that was implicitly held across the reschedule
    unsafe { TASK_MANAGER.force_unlock() };
    arch::enable_irqs();
    let task = CurrentTask::get();
    let entry: fn(usize) -> i32 = unsafe { core::mem::transmute(task.entry.pc) };
    let ret = entry(task.entry.arg);
    CurrentTask::exit(ret as _);
}

fn user_task_entry() -> ! {
    // release the lock that was implicitly held across the reschedule
    unsafe { TASK_MANAGER.force_unlock() };
    assert!(arch::irqs_disabled());
    let task = CurrentTask::get();
    let tf = TrapFrame::new_user(task.entry.pc, task.entry.sp);
    unsafe { tf.exec(task.kstack.top()) };
}

pub struct CurrentTask;

impl CurrentTask {
    fn get() -> &'static Task {
        unsafe { &*(arch::thread_pointer() as *const Task) }
    }

    fn get_mut() -> &'static mut Task {
        unsafe { &mut *(arch::thread_pointer() as *mut Task) }
    }

    fn set(task: &Task) {
        unsafe { arch::set_thread_pointer(task as *const _ as _) };
    }

    pub fn yield_now() {
        TASK_MANAGER.lock().yield_current()
    }

    pub fn exit(exit_code: i32) -> ! {
        TASK_MANAGER.lock().exit_current(exit_code)
    }
}

pub fn init() {
    TASK_MANAGER.lock().init();
}

pub fn run() -> ! {
    let idle = Task::uninit();
    CurrentTask::set(&idle);
    TASK_MANAGER.lock().resched();
    panic!("No tasks to run!");
}
