ARCH := aarch64
TARGET := $(shell cat ../targets.txt | grep $(ARCH))
MODE := release
APP_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
APPS := $(wildcard $(APP_DIR)/*.rs)
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))
ASMS := $(patsubst %, %.asm, $(ELFS))

BUILD_ARGS := --target $(TARGET) -Zbuild-std=core
ifeq ($(MODE), release)
  BUILD_ARGS += --release
endif

OBJDUMP := rust-objdump --arch-name=$(ARCH) --print-imm-hex
OBJCOPY := rust-objcopy --binary-architecture=$(ARCH)

export RUSTFLAGS=-C target-feature=-neon,-fp

all: build

elf:
	cargo build $(BUILD_ARGS)

$(ELFS): elf

$(TARGET_DIR)/%.asm: $(TARGET_DIR)/%
	$(OBJDUMP) -d $< > $@

disasm: $(ASMS) $(ELFS)

build: elf

clean:
	@cargo clean

.PHONY: all binary disasm build clean
