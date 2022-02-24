use core::arch::asm;

const BOOT_STACK_SIZE: usize = 4096 * 16;

#[link_section = ".bss.stack"]
static mut BOOT_STACK: [u8; BOOT_STACK_SIZE] = [0; BOOT_STACK_SIZE];

#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    asm!("
        mov     sp, {boot_stack_top}
        b       {rust_main}",
        boot_stack_top = in(reg) BOOT_STACK.as_ptr_range().end,
        rust_main = sym crate::rust_main,
        options(noreturn),
    )
}
