pub use crate::arch::config::*;
pub use crate::platform::config::*;

pub const PHYS_MEMORY_END: usize = PHYS_MEMORY_BASE + PHYS_MEMORY_SIZE;

pub const BOOT_KERNEL_STACK_SIZE: usize = 4096 * 4; // 16K
pub const USER_STACK_SIZE: usize = 4096 * 4; // 16K
pub const USER_STACK_BASE: usize = 0x7fff_0000_0000 - USER_STACK_SIZE;
pub const KERNEL_STACK_SIZE: usize = 4096 * 4; // 16K
pub const KERNEL_HEAP_SIZE: usize = 0x40_0000; // 4M

pub const MAX_CPUS: usize = 1;

pub const TICKS_PER_SEC: u64 = 100;

pub const SYSCALL_IPI_IRQ_NUM: usize = 13;
