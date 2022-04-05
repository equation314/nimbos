//! Syscall Forwarding.

mod allocator;
mod queue;

pub mod syscall;

use crate::config::scf;
use crate::task::CurrentTask;
use self::syscall::IpcOpcode;

fn send_request(opcode: IpcOpcode, args: u64) {
    while !queue::SyscallQueueBuffer::get().send(opcode, args) {
        CurrentTask::get().yield_now();
    }
    notify();
}

pub fn notify() {
    crate::drivers::interrupt::send_ipi(scf::SYSCALL_IPI_IRQ_NUM);
}

pub fn init() {
    queue::init();
    allocator::init();
}
