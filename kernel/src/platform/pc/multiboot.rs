use core::arch::global_asm;

use x86_64::registers::control::{Cr0Flags, Cr4Flags};
use x86_64::registers::model_specific::EferFlags;

use super::mem::PHYS_VIRT_OFFSET;
use crate::config::BOOT_KERNEL_STACK_SIZE;

const MULTIBOOT_HEADER_MAGIC: u32 = 0x1BAD_B002;
const MULTIBOOT_HEADER_FLAGS: u32 = 0x0001_0002;

const MULTIBOOT_BOOTLOADER_MAGIC: u32 = 0x2BAD_B002;

const CR0: u64 = Cr0Flags::PROTECTED_MODE_ENABLE.bits()
    | Cr0Flags::MONITOR_COPROCESSOR.bits()
    | Cr0Flags::TASK_SWITCHED.bits()
    | Cr0Flags::NUMERIC_ERROR.bits()
    | Cr0Flags::WRITE_PROTECT.bits()
    | Cr0Flags::PAGING.bits();
const CR4: u64 = Cr4Flags::PHYSICAL_ADDRESS_EXTENSION.bits() | Cr4Flags::PAGE_GLOBAL.bits();
const EFER: u64 = EferFlags::LONG_MODE_ENABLE.bits() | EferFlags::NO_EXECUTE_ENABLE.bits();

global_asm!(
    include_str!("multiboot.S"),
    offset = const PHYS_VIRT_OFFSET,
    boot_stack_size = const BOOT_KERNEL_STACK_SIZE,
    cr0 = const CR0,
    cr4 = const CR4,
    efer_msr = const x86::msr::IA32_EFER,
    efer = const EFER,
);
