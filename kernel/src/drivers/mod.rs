pub mod interrupt;
pub mod misc;
pub mod timer;
pub mod uart;

pub fn init() {
    interrupt::init();
    uart::init();
    timer::init();
}
