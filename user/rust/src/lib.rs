#![no_std]
#![feature(linkage)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(panic_info_message)]

#[macro_use]
pub mod console;

mod arch;
mod lang_items;
mod syscall;

#[repr(C)]
pub struct TimeSpec {
    /// seconds
    pub sec: usize,
    /// nano seconds
    pub nsec: usize,
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    exit(main());
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

use syscall::*;

pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code)
}

pub fn sched_yield() -> isize {
    sys_yield()
}

pub fn get_time() -> isize {
    sys_get_time()
}

pub fn getpid() -> isize {
    sys_getpid()
}

pub fn fork() -> isize {
    sys_fork()
}

pub fn exec(path: &str) -> isize {
    sys_exec(path)
}

pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => {
                sched_yield();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => {
                sched_yield();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn sleep(period_ms: usize) {
    sys_nanosleep(&TimeSpec {
        sec: period_ms / 1000,
        nsec: (period_ms % 1000) * 1_000_000,
    });
}

pub fn thread_spawn(entry: fn(usize) -> i32, arg: usize) -> usize {
    use core::sync::atomic::{AtomicUsize, Ordering};
    const MAX_THREADS: usize = 16;
    const THREAD_STACK_SIZE: usize = 4096 * 4; // 16K
    static mut THREAD_STACKS: [[u8; THREAD_STACK_SIZE]; MAX_THREADS] =
        [[0; THREAD_STACK_SIZE]; MAX_THREADS];
    static THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);

    let thread_id = THREAD_COUNT.fetch_add(1, Ordering::AcqRel);
    let newsp = unsafe { THREAD_STACKS[thread_id].as_ptr_range().end as usize };
    sys_clone(entry, arg, newsp)
}
