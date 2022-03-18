pub fn poweroff() -> ! {
    unsafe { core::arch::asm!("out dx, ax", in("edx") 0x604, in("eax") 0x2000) };
    unreachable!("It should shutdown!")
}
