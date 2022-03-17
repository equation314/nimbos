use crate::drivers::timer::{get_time_ns, NSEC_PER_SEC};
use crate::mm::UserOutPtr;

#[repr(C)]
pub struct TimeSpec {
    /// seconds
    pub sec: usize,
    /// nano seconds
    pub nsec: usize,
}

impl TimeSpec {
    pub fn total_nano_sec(&self) -> u64 {
        self.sec as u64 * NSEC_PER_SEC + self.nsec as u64
    }
}

pub fn sys_get_time_ms() -> isize {
    (get_time_ns() / 1_000_000) as isize
}

pub fn sys_clock_gettime(_clock_id: usize, mut ts: UserOutPtr<TimeSpec>) -> isize {
    let total_ns = get_time_ns();
    let sec = (total_ns / NSEC_PER_SEC) as usize;
    let nsec = (total_ns % NSEC_PER_SEC) as usize;
    ts.write(TimeSpec { sec, nsec });
    0
}
