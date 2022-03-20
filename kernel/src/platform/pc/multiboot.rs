use core::arch::{asm, global_asm};

use crate::arch::consts::PHYS_VIRT_OFFSET;
use crate::config::BOOT_KERNEL_STACK_SIZE;

const MULTIBOOT_HEADER_MAGIC: u32 = 0x1BAD_B002;
const MULTIBOOT_HEADER_FLAGS: u32 = 0x0001_0002;

const MULTIBOOT_BOOTLOADER_MAGIC: u32 = 0x2BAD_B002;

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
    lgdt    [tmp_gdt_desc - {offset}]
    mov     ax, 0x10    // data segment selector
    mov     ss, ax
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax

    // set PAE, PGE, OSFXSR, OSXMMEXCPT bit
    mov     eax, cr4
    or      eax, (1 << 5) | (1 << 7) | (1 << 9) | (1 << 10)
    mov     cr4, eax

    // load the temporary page table
    lea     ebx, [tmp_pml4 - {offset}]
    mov     cr3, ebx

    // set LME, NXE bit in IA32_EFER
    mov     ecx, 0xC0000080
    rdmsr
    and     eax, ~(1 << 10) // clear LMA
    or      eax, (1 << 8) | (1 << 11)
    wrmsr

    // set protected mode, write protect, paging bit
    mov     eax, cr0
    or      eax, (1 << 0) | (1 << 16) | (1 << 31)
    mov     cr0, eax

    // long return to the 64-bit entry
    push    0x18    // code64 segment selector
    lea     eax, [entry64 - {offset}]
    push    eax
    retf

.section .data
.balign 16
tmp_gdt:
    .quad 0x0000000000000000    // 0x00: null
    .quad 0x00cf9b000000ffff    // 0x08: code segment (base=0, limit=0xfffff, type=32bit code exec/read, DPL=0, 4k)
    .quad 0x00cf93000000ffff    // 0x10: data segment (base=0, limit=0xfffff, type=32bit data read/write, DPL=0, 4k)
    .quad 0x00af9b000000ffff    // 0x18: code segment (base=0, limit=0xfffff, type=64bit code exec/read, DPL=0, 4k)

tmp_gdt_desc:
    .short  tmp_gdt_desc - tmp_gdt - 1  // 16-bit Size (Limit) of GDT.
    .long   tmp_gdt - {offset}          // 32-bit Base Address of GDT. (CPU will zero extend to 64-bit)

.balign 4096
tmp_pml4:
    .quad tmp_pdpt_low - {offset} + 0x3     // PRESENT | WRITABLE | paddr(tmp_pdpt)
    .zero 8 * 510
    .quad tmp_pdpt_high - {offset} + 0x3    // PRESENT | WRITABLE | paddr(tmp_pdpt)

tmp_pdpt_low:
    // 0x0000_0000 ~ 0x4000_0000
    .quad 0x0000 | 0x83                     // PRESENT | WRITABLE | HUGE_PAGE | paddr(0x0)
    .zero 8 * 511

tmp_pdpt_high:
    // 0xffff_ff80_0000_0000 ~ 0xffff_ff80_4000_0000
    .quad 0x0000 | 0x83                     // PRESENT | WRITABLE | HUGE_PAGE | paddr(0x0)
    .zero 8 * 511",
    magic = const MULTIBOOT_HEADER_MAGIC,
    flags = const MULTIBOOT_HEADER_FLAGS,
    checksum = const -((MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS) as i32) as u32,
    offset = const PHYS_VIRT_OFFSET,
);

#[naked]
#[no_mangle]
#[link_section = ".text.boot"]
unsafe extern "C" fn entry64(_magic: u32, _info: usize) -> ! {
    asm!("
        // clear segment selectors
        xor     ax, ax
        mov     ss, ax
        mov     ds, ax
        mov     es, ax
        mov     fs, ax
        mov     gs, ax

        // set stack and jump to rust_main
        movabs  rax, offset {boot_stack}
        add     rax, {boot_stack_size}
        mov     rsp, rax
        movabs  rax, offset {rust_main}
        jmp     rax",
        boot_stack = sym BOOT_STACK,
        boot_stack_size = const BOOT_KERNEL_STACK_SIZE,
        rust_main = sym crate::rust_main,
        options(noreturn),
    )
}
