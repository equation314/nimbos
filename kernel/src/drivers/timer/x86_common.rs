use x2apic::lapic::{TimerDivide, TimerMode};

use super::{x86_hpet, x86_tsc};
use crate::drivers::interrupt::local_apic;
use crate::sync::LazyInit;
use crate::timer::{current_time_nanos, nanos_to_ticks};

pub use x86_tsc::{current_ticks, frequency_hz}; // use TSC as the clock source.

static LAPIC_FREQ_HZ: LazyInit<u64> = LazyInit::new();

pub fn set_oneshot_timer(deadline_ns: u64) {
    let now_ns = current_time_nanos();
    unsafe {
        if now_ns < deadline_ns {
            let apic_ticks = nanos_to_ticks(deadline_ns - now_ns, *LAPIC_FREQ_HZ);
            debug_assert!(apic_ticks <= u32::MAX as u64);
            local_apic().set_timer_initial(apic_ticks.max(1) as u32);
        } else {
            local_apic().set_timer_initial(1);
        }
    }
}

fn calibrate_lapic_timer() {
    let lapic = local_apic();
    unsafe {
        lapic.set_timer_mode(TimerMode::OneShot);
        lapic.set_timer_divide(TimerDivide::Div256); // divide 1
    }

    let mut best_freq_hz = u64::MAX;
    for _ in 0..5 {
        unsafe { lapic.set_timer_initial(u32::MAX) };
        x86_hpet::wait_millis(10);
        let ticks_per_sec = unsafe { (u32::MAX - lapic.timer_current()) as u64 * 100 };
        if ticks_per_sec < best_freq_hz {
            best_freq_hz = ticks_per_sec;
        }
    }
    println!(
        "Calibrated LAPIC frequency: {}.{:03} MHz",
        best_freq_hz / 1_000_000,
        best_freq_hz % 1_000_000 / 1_000,
    );

    LAPIC_FREQ_HZ.init_by(best_freq_hz);
    unsafe { lapic.enable_timer() } // enable APIC timer IRQ
}

pub fn init() {
    super::x86_hpet::init();
    x86_tsc::calibrate_tsc();
    calibrate_lapic_timer();
}
