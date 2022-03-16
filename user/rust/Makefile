ARCH ?= aarch64
MODE ?= release

target := $(shell cat ../../targets.txt | grep $(ARCH))
target_dir := target/$(target)/$(MODE)
app_dir := src/bin
apps := $(wildcard $(app_dir)/*.rs)
elfs := $(patsubst $(app_dir)/%.rs, $(target_dir)/%, $(apps))
asms := $(patsubst %, %.asm, $(elfs))

build_args := --target $(target) -Zbuild-std=core
ifeq ($(MODE), release)
  build_args += --release
endif

OBJDUMP := rust-objdump --arch-name=$(ARCH) --print-imm-hex
OBJCOPY := rust-objcopy --binary-architecture=$(ARCH)

export RUSTFLAGS=-C target-feature=-neon,-fp

all: build

elf:
	cargo build $(build_args)

$(elfs): elf

$(target_dir)/%.asm: $(target_dir)/%
	$(OBJDUMP) -d $< > $@

disasm: $(asms) $(elfs)

build: elf
	@cp $(elfs) ../build/$(ARCH)

clean:
	@cargo clean

.PHONY: all binary disasm build clean
