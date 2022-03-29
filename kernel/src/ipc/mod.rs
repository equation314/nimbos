//! Inter-Processor Communication.

mod structs;

pub use structs::IpcOpcode;

use self::structs::IpcBuffer;
use crate::config::ipc;
use crate::task::CurrentTask;

pub fn notify() {
    crate::drivers::interrupt::send_ipi(ipc::SYSCALL_IPI_IRQ_NUM);
}

pub fn init() {
    IpcBuffer::get_send_buffer().init(ipc::SYSCALL_SEND_BUF_SIZE);
    IpcBuffer::get_recv_buffer().init(ipc::SYSCALL_RECV_BUF_SIZE);
}

pub fn send_request(opcode: IpcOpcode, args: u64) {
    while !IpcBuffer::get_send_buffer().send(opcode, args) {
        CurrentTask::get().yield_now();
    }
    notify();
}
