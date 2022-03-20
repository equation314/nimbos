#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(asm_sym)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_maybe_uninit_zeroed)]
#![allow(dead_code)]
#![allow(unused_variables)]

extern crate alloc;
#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate log;

#[macro_use]
mod logging;

mod arch;
mod config;
mod drivers;
mod loader;
mod mm;
mod platform;
mod sync;
mod syscall;
mod task;
mod utils;

#[cfg(not(test))]
mod lang_items;

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

const LOGO: &str = r"
NN   NN  iii               bb        OOOOO    SSSSS
NNN  NN       mm mm mmmm   bb       OO   OO  SS
NN N NN  iii  mmm  mm  mm  bbbbbb   OO   OO   SSSSS
NN  NNN  iii  mmm  mm  mm  bb   bb  OO   OO       SS
NN   NN  iii  mmm  mm  mm  bbbbbb    OOOO0    SSSSS
              ___    ____    ___    ___
             |__ \  / __ \  |__ \  |__ \
             __/ / / / / /  __/ /  __/ /
            / __/ / /_/ /  / __/  / __/
           /____/ \____/  /____/ /____/
";

pub fn rust_main() -> ! {
    clear_bss();
    drivers::init_early();
    println!("{}", LOGO);

    arch::init();
    mm::init();
    drivers::init();
    logging::init();
    info!("Logging is enabled.");

    task::init();
    loader::list_apps();
    task::run();
}
