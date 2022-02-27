use core::ops::Deref;

use crate::config::{KERNEL_STACK_SIZE, USER_STACK_SIZE};

#[repr(align(4096))]
pub struct Stack<const N: usize>([u8; N]);

impl<const N: usize> Stack<N> {
    const fn default() -> Self {
        Self([0; N])
    }

    pub fn top(&self) -> usize {
        self.0.as_ptr_range().end as usize
    }
}

impl<const N: usize> Deref for Stack<N> {
    type Target = [u8; N];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub static KERNEL_STACK: Stack<KERNEL_STACK_SIZE> = Stack::default();
pub static USER_STACK: Stack<USER_STACK_SIZE> = Stack::default();

extern "C" {
    fn _app_count();
}

pub fn get_app_count() -> usize {
    unsafe { (_app_count as *const u64).read() as usize }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    unsafe {
        let app_0_start_ptr = (_app_count as *const u64).add(1);
        assert!(app_id < get_app_count());
        let app_start = app_0_start_ptr.add(app_id).read() as usize;
        let app_end = app_0_start_ptr.add(app_id + 1).read() as usize;
        let app_size = app_end - app_start;
        assert!(app_size < crate::config::APP_SIZE_LIMIT);
        core::slice::from_raw_parts(app_start as *const u8, app_size)
    }
}
