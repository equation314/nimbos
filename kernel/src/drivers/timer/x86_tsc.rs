use raw_cpuid::CpuId;

use crate::sync::LazyInit;

static CPU_FREQUENCY: LazyInit<u16> = LazyInit::new();

pub fn get_time_ns() -> u64 {
    let cycle = unsafe { core::arch::x86_64::_rdtsc() };
    cycle * 1000 / *CPU_FREQUENCY as u64
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
    // timer interrupts are initialized by local APIC
}
