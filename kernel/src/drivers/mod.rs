pub mod interrupt;
pub mod misc;
pub mod timer;
pub mod uart;

pub fn init() {
    println!("Initializing drivers...");
    interrupt::init();
    uart::init();
    timer::init();
}
