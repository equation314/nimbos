# Arguments
ARCH ?= aarch64
MODE ?= release

# Platform
ifeq ($(ARCH), aarch64)
  PLATFORM ?= qemu-virt-arm
endif

# Paths
target := $(shell cat ../targets.txt | grep $(ARCH))
kernel_elf := target/$(target)/$(MODE)/nimbos
kernel_bin := $(kernel_elf).bin

# Cargo features and build args
features := platform-$(PLATFORM)

build_args := --features "$(features)" --target $(target) -Zbuild-std=core,alloc
ifeq ($(MODE), release)
  build_args += --release
endif

# Binutils
OBJDUMP := rust-objdump -d --arch-name=$(ARCH) --print-imm-hex
OBJCOPY := rust-objcopy --binary-architecture=$(ARCH)
GDB := gdb-multiarch

# QEMU
qemu := qemu-system-$(ARCH)
qemu_args := -nographic
ifeq ($(ARCH), aarch64)
  qemu_args += \
    -cpu cortex-a72 \
    -machine virt \
    -kernel $(kernel_bin)
endif

# GDB
GDB := gdb-multiarch

build: $(kernel_bin)

env:
	(rustup target list | grep "$(target) (installed)") || rustup target add $(target)
	cargo install cargo-binutils --vers =0.3.3
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(kernel_bin): kernel
	@$(OBJCOPY) $(kernel_elf) --strip-all -O binary $@

kernel:
	@cd ../user && make build
	@echo Arch: $(ARCH), Platform: $(PLATFORM)
	cargo build $(build_args)

clean:
	@cd ../user && make clean
	@cargo clean

clippy:
	cargo clippy $(build_args)

disasm:
	@$(OBJDUMP) $(kernel_elf) | less

run: build
	$(qemu) $(qemu_args)

debug: build
	$(qemu) $(qemu_args) -s -S &
	sleep 1
	$(GDB) $(kernel_elf) -ex 'target remote localhost:1234'

.PHONY: build env kernel clean disasm run debug
