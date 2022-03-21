use core::arch::global_asm;

use x86::{controlregs::cr2, irq::*};

use super::context::TrapFrame;
use crate::drivers::interrupt::IrqHandlerResult;
use crate::task::CurrentTask;

global_asm!(include_str!("trap.S"));

const IRQ_VECTOR_START: u8 = 32;
const IRQ_VECTOR_END: u8 = 255;

#[no_mangle]
fn x86_trap_handler(tf: &mut TrapFrame) {
    match tf.vector as u8 {
        PAGE_FAULT_VECTOR => {
            println!(
                "[kernel] Page Fault @ {:#x}, fault_vaddr = {:#x}, error_code = {:#x}, kernel killed it.",
                tf.rip,
                unsafe{ cr2() },
                tf.error_code,
            );
            CurrentTask::get().exit(-1);
        }
        IRQ_VECTOR_START..=IRQ_VECTOR_END => {
            debug!("IRQ {}", tf.vector);
            if crate::drivers::interrupt::handle_irq() == IrqHandlerResult::Reschedule {
                CurrentTask::get().yield_now();
            }
        }
        _ => {
            panic!(
                "Unsupported exception {} (error_code = {:#x}) @ {:#x}:\n{:#x?}",
                tf.vector, tf.error_code, tf.rip, tf
            );
        }
    }
}
