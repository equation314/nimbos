mod context;

pub use context::TrapContext;

use core::arch::global_asm;

use cortex_a::registers::{ESR_EL1, FAR_EL1, VBAR_EL1};
use tock_registers::interfaces::{Readable, Writeable};

global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" {
        fn exception_vector_base();
    }
    VBAR_EL1.set(exception_vector_base as u64);
}

#[repr(u8)]
#[derive(Debug)]
#[allow(dead_code)]
enum TrapKind {
    Synchronous = 0,
    IRQ = 1,
    FIQ = 2,
    SError = 3,
}

#[repr(u8)]
#[derive(Debug)]
#[allow(dead_code)]
enum TrapSource {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[no_mangle]
fn invalid_exception(cx: &mut TrapContext, kind: TrapKind, source: TrapSource) {
    panic!(
        "Invalid exception {:?} from {:?}:\n{:#x?}",
        kind, source, cx
    );
}

#[no_mangle]
fn handle_sync_exception(cx: &mut TrapContext) {
    let esr = ESR_EL1.extract();
    match esr.read_as_enum(ESR_EL1::EC) {
        Some(ESR_EL1::EC::Value::SVC64) => {
            println!("Syscall: {:#x?}", cx);
        }
        Some(ESR_EL1::EC::Value::DataAbortLowerEL)
        | Some(ESR_EL1::EC::Value::DataAbortCurrentEL) => {
            println!("Data Abort @ {:#x}, FAR = {:#x}", cx.elr, FAR_EL1.get());
        }
        Some(ESR_EL1::EC::Value::InstrAbortLowerEL)
        | Some(ESR_EL1::EC::Value::InstrAbortCurrentEL) => {
            println!(
                "Instruction Abort @ {:#x}, FAR = {:#x}",
                cx.elr,
                FAR_EL1.get()
            );
        }
        _ => {
            panic!(
                "Unsupported synchronous exception @ {:#x}: ESR = {:#x} (EC {:#08b}, ISS {:#x})",
                cx.elr,
                esr.get(),
                esr.read(ESR_EL1::EC),
                esr.read(ESR_EL1::ISS),
            );
        }
    }
}

#[no_mangle]
fn handle_irq_exception(_cx: &mut TrapContext) -> ! {
    panic!("Unsupported IRQ exception!");
}
