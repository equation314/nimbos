mod manager;
mod switch;

use alloc::boxed::Box;

use self::manager::TASK_MANAGER;
use self::switch::TaskContext;
use crate::arch;
use crate::config::KERNEL_STACK_SIZE;
use crate::loader::{self, Stack};
use crate::mm::MemorySet;
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
    memory_set: Option<MemorySet>,
    kstack: Option<Box<Stack<KERNEL_STACK_SIZE>>>,
    ctx: TaskContext,
    entry: TaskEntryStatus,
}

impl Task {
    const fn uninit() -> Self {
        Self {
            id: 0,
            status: TaskStatus::UnInit,
            memory_set: None,
            ctx: TaskContext::default(),
            kstack: None,
            entry: TaskEntryStatus {
                pc: 0,
                sp: 0,
                arg: 0,
            },
        }
    }

    pub fn new_kernel(id: usize, entry: fn(usize) -> usize, arg: usize) -> Self {
        let mut t = Self::uninit();
        let kstack = Box::new(Stack::default());
        t.id = id;
        t.entry = TaskEntryStatus {
            pc: entry as usize,
            arg,
            sp: 0,
        };
        t.ctx.init(kernel_task_entry as _, kstack.top());
        t.kstack = Some(kstack);
        t.status = TaskStatus::Ready;
        t
    }

    pub fn new_user(id: usize, entry: usize, ustack_top: usize, ms: MemorySet) -> Self {
        let mut t = Self::uninit();
        let kstack = Box::new(Stack::default());
        t.id = id;
        t.memory_set = Some(ms);
        t.entry = TaskEntryStatus {
            pc: entry as usize,
            arg: 0,
            sp: ustack_top,
        };
        t.ctx.init(user_task_entry as _, kstack.top());
        t.kstack = Some(kstack);
        t.status = TaskStatus::Ready;
        t
    }

    /// Must called in `TaskManager::resched()`.
    fn switch_to(&mut self, next_task: &mut Task) {
        assert!(TASK_MANAGER.is_locked());
        assert!(next_task.status != TaskStatus::UnInit);
        next_task.status = TaskStatus::Running;
        if core::ptr::eq(self, next_task) {
            return;
        }
        CurrentTask::set(next_task);
        unsafe {
            if let Some(ms) = &mut next_task.memory_set {
                ms.activate(false); // user task
            } else {
                arch::activate_paging(0, false); // kernel task
            }
            switch::context_switch(&mut self.ctx, &next_task.ctx);
        }
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
    unsafe { tf.exec(task.kstack.as_ref().unwrap().top()) };
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
    let mut m = TASK_MANAGER.lock();
    let kernel_task_count = 2;
    m.add_task(Task::new_kernel(
        0,
        |arg| {
            println!("test kernel task 0: arg = {:#x}", arg);
            0
        },
        0xdead,
    ));
    m.add_task(Task::new_kernel(
        1,
        |arg| {
            println!("test kernel task 1: arg = {:#x}", arg);
            0
        },
        0xbeef,
    ));
    for i in 0..loader::get_app_count() {
        let (entry, ustack_top, ms) = loader::load_app(i);
        m.add_task(Task::new_user(i + kernel_task_count, entry, ustack_top, ms));
    }
}

pub fn run() -> ! {
    let idle = Task::uninit();
    CurrentTask::set(&idle);
    TASK_MANAGER.lock().resched();
    panic!("No tasks to run!");
}
