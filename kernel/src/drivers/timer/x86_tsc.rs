use raw_cpuid::CpuId;

use crate::sync::LazyInit;

static TSC_FREQ_HZ: LazyInit<u64> = LazyInit::new();

pub fn current_ticks() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

pub fn frequency_hz() -> u64 {
    *TSC_FREQ_HZ
}

pub(super) fn calibrate_tsc() {
    if let Some(freq) = CpuId::new()
        .get_processor_frequency_info()
        .map(|info| info.processor_base_frequency())
    {
        if freq > 0 {
            println!("Got TSC frequency by CPUID: {} MHz", freq,);
            TSC_FREQ_HZ.init_by(freq as u64 * 1_000_000);
            return;
        }
    }

    let mut best_freq_hz = u64::MAX;
    for _ in 0..5 {
        let tsc_start = unsafe { core::arch::x86_64::_rdtsc() };
        super::x86_hpet::wait_millis(10);
        let tsc_end = unsafe { core::arch::x86_64::_rdtsc() };
        let freq_hz = (tsc_end - tsc_start) * 100;
        if freq_hz < best_freq_hz {
            best_freq_hz = freq_hz;
        }
    }
    println!(
        "Calibrated TSC frequency: {}.{:03} MHz",
        best_freq_hz / 1_000_000,
        best_freq_hz % 1_000_000 / 1_000,
    );

    TSC_FREQ_HZ.init_by(best_freq_hz);
}
