use core::arch::asm;

use crate::mm::{PhysAddr, VirtAddr};

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
    pub fn new_user(entry: VirtAddr, ustack_top: VirtAddr, arg0: usize) -> Self {
        Self {
            rdi: arg0 as _,
            rip: entry.as_usize() as _,
            cs: 0x20 | 3,
            rflags: 0x202, // IOPL = 0, IF = 1
            user_rsp: ustack_top.as_usize() as _,
            user_ss: 0x28 | 3,
            ..Default::default()
        }
    }

    pub const fn new_clone(&self, ustack_top: VirtAddr) -> Self {
        let mut tf = *self;
        tf.user_rsp = ustack_top.as_usize() as _;
        tf.rax = 0; // for child thread, clone returns 0
        tf
    }

    pub const fn new_fork(&self) -> Self {
        let mut tf = *self;
        tf.rax = 0; // for child process, fork returns 0
        tf
    }

    pub unsafe fn exec(&self, _kstack_top: VirtAddr) -> ! {
        unimplemented!()
    }
}

#[repr(C)]
#[derive(Debug, Default)]
struct ContextSwitchFrame {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
    rflags: u64,
    rip: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct TaskContext {
    pub rsp: u64,
    pub fs_base: u64,
    pub cr3: u64,
}

impl TaskContext {
    pub const fn default() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }

    pub fn init(
        &mut self,
        entry: usize,
        kstack_top: VirtAddr,
        page_table_root: PhysAddr,
        _is_kernel: bool,
    ) {
        unsafe {
            let frame_ptr = (kstack_top.as_mut_ptr() as *mut ContextSwitchFrame).sub(1);
            core::ptr::write(
                frame_ptr,
                ContextSwitchFrame {
                    rip: entry as _,
                    rflags: 0x2, // IOPL = 0, IF = 0
                    ..Default::default()
                },
            );
            self.rsp = frame_ptr as u64;
        }
        self.cr3 = page_table_root.as_usize() as u64;
    }

    pub fn switch_to(&mut self, next_ctx: &Self) {
        unsafe {
            crate::arch::instructions::set_user_page_table_root(next_ctx.cr3 as usize);
            // TODO: swtich fs_base
            context_switch(&mut self.rsp, &next_ctx.rsp)
        }
    }
}

#[naked]
unsafe extern "C" fn context_switch(_current_stack: &mut u64, _next_stack: &u64) {
    asm!(
        "
        pushf
        push    rbp
        push    rbx
        push    r12
        push    r13
        push    r14
        push    r15
        mov     [rdi], rsp

        mov     rsp, [rsi]
        pop     r15
        pop     r14
        pop     r13
        pop     r12
        pop     rbx
        pop     rbp
        popf
        ret",
        options(noreturn),
    )
}
