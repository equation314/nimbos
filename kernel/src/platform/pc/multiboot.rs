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

#[no_mangle]
#[link_section = ".bss.stack"]
static mut BOOT_STACK: [u8; BOOT_KERNEL_STACK_SIZE] = [0; BOOT_KERNEL_STACK_SIZE];

global_asm!("
.section .text.boot
.code32
.global _start
_start:
    mov     edi, eax        // magic
    mov     esi, ebx        // multiboot info
    jmp     entry32

.balign 4
.type multiboot_header, STT_OBJECT
multiboot_header:
    .int    {magic}
    .int    {flags}
    .int    {checksum}
    .int    multiboot_header - {offset}        // header_addr
    .int    skernel - {offset}                 // load_addr
    .int    edata - {offset}                   // load_end
    .int    ebss - {offset}                    // bss_end_addr
    .int    _start - {offset}                  // entry_addr

entry32:
    // load the temporary GDT
    lgdt    [.Ltmp_gdt_desc_phys - {offset}]
    mov     ax, 0x10    // data segment selector
    mov     ss, ax
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax

    // set PAE, PGE bit in CR4
    mov     eax, {cr4}
    mov     cr4, eax

    // load the temporary page table
    lea     ebx, [.Ltmp_pml4 - {offset}]
    mov     cr3, ebx

    // set LME, NXE bit in IA32_EFER
    mov     ecx, {efer_msr}
    mov     edx, 0
    mov     eax, {efer}
    wrmsr

    // set protected mode, write protect, paging bit in CR0
    mov     eax, {cr0}
    mov     cr0, eax

    // long return to the 64-bit entry
    push    0x18    // code64 segment selector
    lea     eax, [entry64 - {offset}]
    push    eax
    retf

.code64
entry64:
    // reload GDT by high address
    movabs  rax, offset .Ltmp_gdt_desc
    lgdt    [rax]

    // clear segment selectors
    xor     ax, ax
    mov     ss, ax
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax

    // set stack and jump to rust_main
    movabs  rax, offset BOOT_STACK
    add     rax, {boot_stack_size}
    mov     rsp, rax
    movabs  rax, offset rust_main
    jmp     rax

.section .rodata
.balign 8
.Ltmp_gdt_desc_phys:
    .short  .Ltmp_gdt_end - .Ltmp_gdt - 1   // limit
    .long   .Ltmp_gdt - {offset}            // base

.balign 8
.Ltmp_gdt_desc:
    .short  .Ltmp_gdt_end - .Ltmp_gdt - 1   // limit
    .quad   .Ltmp_gdt                       // base

.section .data
.balign 16
.Ltmp_gdt:
    .quad 0x0000000000000000    // 0x00: null
    .quad 0x00cf9b000000ffff    // 0x08: code segment (base=0, limit=0xfffff, type=32bit code exec/read, DPL=0, 4k)
    .quad 0x00cf93000000ffff    // 0x10: data segment (base=0, limit=0xfffff, type=32bit data read/write, DPL=0, 4k)
    .quad 0x00af9b000000ffff    // 0x18: code segment (base=0, limit=0xfffff, type=64bit code exec/read, DPL=0, 4k)
.Ltmp_gdt_end:

.balign 4096
.Ltmp_pml4:
    .quad .Ltmp_pdpt_low - {offset} + 0x3   // PRESENT | WRITABLE | paddr(tmp_pdpt)
    .zero 8 * 510
    .quad .Ltmp_pdpt_high - {offset} + 0x3  // PRESENT | WRITABLE | paddr(tmp_pdpt)

.Ltmp_pdpt_low:
    // 0x0000_0000 ~ 0x4000_0000
    .quad 0x0000 | 0x83                     // PRESENT | WRITABLE | HUGE_PAGE | paddr(0x0)
    .zero 8 * 511

.Ltmp_pdpt_high:
    // 0xffff_ff80_0000_0000 ~ 0xffff_ff80_4000_0000
    .quad 0x0000 | 0x83                     // PRESENT | WRITABLE | HUGE_PAGE | paddr(0x0)
    .zero 8 * 511",
    magic = const MULTIBOOT_HEADER_MAGIC,
    flags = const MULTIBOOT_HEADER_FLAGS,
    checksum = const -((MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS) as i32) as u32,
    offset = const PHYS_VIRT_OFFSET,
    boot_stack_size = const BOOT_KERNEL_STACK_SIZE,
    cr0 = const CR0,
    cr4 = const CR4,
    efer_msr = const x86::msr::IA32_EFER,
    efer = const EFER,
);
