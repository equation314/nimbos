use core::arch::asm;
use core::ops::Deref;

use crate::config::{APP_BASE_ADDRESS, APP_SIZE_LIMIT, MAX_APP_NUM, USER_STACK_SIZE};

#[repr(align(4096))]
#[derive(Clone, Copy)]
pub struct Stack<const N: usize>([u8; N]);

impl<const N: usize> Stack<N> {
    pub const fn default() -> Self {
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

pub static USER_STACK: [Stack<USER_STACK_SIZE>; MAX_APP_NUM] = [Stack::default(); MAX_APP_NUM];

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

pub fn load_app(app_id: usize) -> (usize, usize) {
    assert!(app_id < get_app_count());
    let entry = APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT;

    // clear app area
    unsafe { core::slice::from_raw_parts_mut(entry as *mut u8, APP_SIZE_LIMIT).fill(0) };
    // copy app binary
    let app_data = get_app_data(app_id);
    let app_dst = unsafe { core::slice::from_raw_parts_mut(entry as *mut u8, app_data.len()) };
    app_dst.copy_from_slice(app_data);
    // clear icache
    unsafe { asm!("ic iallu; dsb sy; isb") };

    let ustack_top = USER_STACK[app_id].top();
    (entry, ustack_top)
}
