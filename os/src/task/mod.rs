mod switch;

use self::switch::TaskContext;
use crate::loader::KERNEL_STACK;

#[derive(Debug, Default)]
pub struct Task {
    ctx: TaskContext,
}

impl Task {
    pub fn switch_to(&mut self, new_task: &Self) {
        unsafe { switch::context_switch(&mut self.ctx, &new_task.ctx) };
    }
}

static mut TASKS: [Task; 2] = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };

fn test_other_thread() -> ! {
    println!("[kernel] switch to other thread!");
    unsafe { TASKS[1].switch_to(&TASKS[0]) };
    unreachable!()
}

pub fn run() -> ! {
    unsafe {
        TASKS[1]
            .ctx
            .init(test_other_thread as usize, KERNEL_STACK.top());
        TASKS[0].switch_to(&TASKS[1]);
    }
    println!("[kernel] back to idle thread!");
    println!("{:#x?}", unsafe { &TASKS });
    panic!("No tasks to run!");
}
