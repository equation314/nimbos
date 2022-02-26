ARCH := aarch64
TARGET := $(shell cat ../targets.txt | grep $(ARCH))
MODE := release
APP_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
APPS := $(wildcard $(APP_DIR)/*.rs)
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))
BINS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%.bin, $(APPS))

BUILD_ARGS := --target $(TARGET) -Zbuild-std=core
ifeq ($(MODE), release)
  BUILD_ARGS += --release
endif

OBJDUMP := rust-objdump --arch-name=$(ARCH) --print-imm-hex
OBJCOPY := rust-objcopy --binary-architecture=$(ARCH)

elf: $(APPS)
	cargo build $(BUILD_ARGS)
	@echo $(APPS)
	@echo $(ELFS)
	@echo $(BINS)

binary: elf
	$(foreach elf, $(ELFS), $(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%,$(TARGET_DIR)/%.bin,$(elf));)

disasm: elf
	$(foreach elf, $(ELFS), $(OBJDUMP) $(elf) -d > $(patsubst $(TARGET_DIR)/%,$(TARGET_DIR)/%.asm,$(elf));)

build: binary

clean:
	@cargo clean

.PHONY: elf binary disasm build clean
