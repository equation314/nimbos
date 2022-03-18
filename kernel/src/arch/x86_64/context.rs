use core::arch::asm;

use crate::mm::PhysAddr;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TrapFrame {
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    // Pushed by 'vector.S'
    pub vector: u64,
    pub error_code: u64,

    // Pushed by CPU
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,

    // Pushed by CPU when trap from ring-3
    pub user_rsp: u64,
    pub user_ss: u64,
}

impl TrapFrame {
    pub fn new_user(entry: usize, ustack_top: usize, arg0: usize) -> Self {
        Self {
            rdi: arg0 as _,
            rip: entry as _,
            cs: 0x20 | 3,
            rflags: 0x3000 | 0x200 | 0x2, // IOPL = 3, IF = 1 (FIXME: set IOPL = 0 when IO port bitmap is supported)
            user_rsp: ustack_top as _,
            user_ss: 0x28 | 3,
            ..Default::default()
        }
    }

    pub const fn new_clone(&self, ustack_top: usize) -> Self {
        let mut tf = *self;
        tf.user_rsp = ustack_top as _;
        tf.rax = 0; // for child thread, clone returns 0
        tf
    }

    pub const fn new_fork(&self) -> Self {
        let mut tf = *self;
        tf.rax = 0; // for child process, fork returns 0
        tf
    }

    pub unsafe fn exec(&self, kstack_top: usize) -> ! {
        asm!(
            "
            // mov     sp, x1
            // ldp     x30, x9, [x0, 30 * 8]
            // ldp     x10, x11, [x0, 32 * 8]
            // msr     sp_el0, x9
            // msr     elr_el1, x10
            // msr     spsr_el1, x11

            // ldp     x28, x29, [x0, 28 * 8]
            // ldp     x26, x27, [x0, 26 * 8]
            // ldp     x24, x25, [x0, 24 * 8]
            // ldp     x22, x23, [x0, 22 * 8]
            // ldp     x20, x21, [x0, 20 * 8]
            // ldp     x18, x19, [x0, 18 * 8]
            // ldp     x16, x17, [x0, 16 * 8]
            // ldp     x14, x15, [x0, 14 * 8]
            // ldp     x12, x13, [x0, 12 * 8]
            // ldp     x10, x11, [x0, 10 * 8]
            // ldp     x8, x9, [x0, 8 * 8]
            // ldp     x6, x7, [x0, 6 * 8]
            // ldp     x4, x5, [x0, 4 * 8]
            // ldp     x2, x3, [x0, 2 * 8]
            // ldp     x0, x1, [x0]

            iret",
            // in("x0") self,
            // in("x1") kstack_top,
            options(noreturn),
        )
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct TaskContext {
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rbx: u64,
    pub rbp: u64,

    pub rflags: u64,
    pub rsp: u64,
    pub rip: u64,

    pub fs_base: u64,
    pub cr3: u64,
}

impl TaskContext {
    pub const fn default() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }

    pub fn init(&mut self, entry: usize, kstack_top: usize, page_table_root: PhysAddr) {
        self.rsp = kstack_top as u64;
        self.rip = entry as u64;
        self.cr3 = page_table_root.as_usize() as u64;
    }

    pub fn switch_to(&mut self, next_ctx: &Self) {
        unsafe {
            crate::arch::instructions::activate_paging(next_ctx.cr3 as usize, false);
            context_switch(self, next_ctx)
        }
    }
}

#[naked]
unsafe extern "C" fn context_switch(_current_task: &mut TaskContext, _next_task: &TaskContext) {
    asm!(
        "
        // save old context (callee-saved registers)
        // stp     x29, x30, [x0, 12 * 8]
        // stp     x27, x28, [x0, 10 * 8]
        // stp     x25, x26, [x0, 8 * 8]
        // stp     x23, x24, [x0, 6 * 8]
        // stp     x21, x22, [x0, 4 * 8]
        // stp     x19, x20, [x0, 2 * 8]
        // mov     x19, sp
        // mrs     x20, tpidr_el0
        // stp     x19, x20, [x0]

        // restore new context
        // ldp     x19, x20, [x1]
        // mov     sp, x19
        // msr     tpidr_el0, x20
        // ldp     x19, x20, [x1, 2 * 8]
        // ldp     x21, x22, [x1, 4 * 8]
        // ldp     x23, x24, [x1, 6 * 8]
        // ldp     x25, x26, [x1, 8 * 8]
        // ldp     x27, x28, [x1, 10 * 8]
        // ldp     x29, x30, [x1, 12 * 8]

        ret",
        options(noreturn),
    )
}
