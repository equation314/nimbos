use raw_cpuid::CpuId;
use x2apic::lapic::TimerMode;

use super::super::interrupt::local_apic;
use crate::{sync::LazyInit, timer::TimeValue};

const LAPIC_FREQ: u64 = 1_000_000_000; // TODO: need to calibrate

static CPU_FREQUENCY: LazyInit<u16> = LazyInit::new();

fn lapic_nanos_to_ticks(nanos: u64) -> u32 {
    let ticks = nanos * (LAPIC_FREQ / 1_000_000_000);
    debug_assert!(ticks <= u32::MAX as u64);
    ticks as u32
}

pub fn current_time_nanos() -> u64 {
    let cycle = unsafe { core::arch::x86_64::_rdtsc() };
    cycle * 1000 / *CPU_FREQUENCY as u64
}

pub fn current_time() -> TimeValue {
    TimeValue::from_nanos(current_time_nanos())
}

pub fn set_oneshot_timer(deadline_ns: u64) {
    unsafe {
        let now_ns = current_time_nanos();
        if now_ns < deadline_ns {
            local_apic().set_timer_initial(lapic_nanos_to_ticks(deadline_ns - now_ns));
        } else {
            local_apic().set_timer_initial(1);
        }
    }
}

pub fn init() {
    const DEFAULT_FREQ: u16 = 4000;
    CPU_FREQUENCY.init_by(
        CpuId::new()
            .get_processor_frequency_info()
            .map(|info| info.processor_base_frequency())
            .and_then(|freq| if freq == 0 { None } else { Some(freq) })
            .unwrap_or(DEFAULT_FREQ),
    );
    let lapic = local_apic();
    unsafe {
        lapic.set_timer_mode(TimerMode::OneShot);
        lapic.enable_timer();
    }
}
