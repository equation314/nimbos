#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::arch::asm;

#[no_mangle]
fn main() -> i32 {
    println!("Try to access EL1 system registers in EL0");
    println!("Kernel should kill this application!");
    unsafe {
        asm!("mrs x0, SPSR_EL1");
    }
    0
}
