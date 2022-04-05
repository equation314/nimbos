pub use crate::arch::config::*;
pub use crate::platform::config::*;

// Memory size

pub const BOOT_KERNEL_STACK_SIZE: usize = 4096 * 4; // 16K
pub const USER_STACK_SIZE: usize = 4096 * 4; // 16K
pub const USER_STACK_BASE: usize = 0x7fff_0000_0000 - USER_STACK_SIZE;
pub const KERNEL_STACK_SIZE: usize = 4096 * 4; // 16K
pub const KERNEL_HEAP_SIZE: usize = 0x40_0000; // 4M

// SMP

pub const MAX_CPUS: usize = 1;

// Timer

pub const TICKS_PER_SEC: u64 = 100;

// Syscall IPC

cfg_if! {
    if #[cfg(feature = "rvm")] {
        pub const PHYS_MEMORY_END: usize = PHYS_MEMORY_BASE + PHYS_MEMORY_SIZE
            - scf::SYSCALL_DATA_BUF_SIZE
            - scf::SYSCALL_QUEUE_BUF_SIZE;

        pub mod scf {
            pub const SYSCALL_IPI_IRQ_NUM: usize = 13;

            pub const SYSCALL_DATA_BUF_SIZE: usize = 0x10_0000; // 1M
            pub const SYSCALL_QUEUE_BUF_SIZE: usize = 4096; // 4K

            pub const SYSCALL_DATA_BUF_PADDR: usize = super::PHYS_MEMORY_END;
            pub const SYSCALL_QUEUE_BUF_PADDR: usize = SYSCALL_DATA_BUF_PADDR + SYSCALL_DATA_BUF_SIZE;
        }
    } else {
        pub const PHYS_MEMORY_END: usize = PHYS_MEMORY_BASE + PHYS_MEMORY_SIZE;
    }
}
