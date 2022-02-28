ARCH := aarch64
TARGET := $(shell cat ../targets.txt | grep $(ARCH))
MODE := release
APP_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
APPS := $(wildcard $(APP_DIR)/*.rs)
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))
BINS := $(patsubst %, %.bin, $(ELFS))
ASMS := $(patsubst %, %.asm, $(ELFS))

BUILD_ARGS := --target $(TARGET) -Zbuild-std=core
ifeq ($(MODE), release)
  BUILD_ARGS += --release
endif

OBJDUMP := rust-objdump --arch-name=$(ARCH) --print-imm-hex
OBJCOPY := rust-objcopy --binary-architecture=$(ARCH)

all: build

$(TARGET_DIR)/%: $(APP_DIR)/%.rs
	$(eval IDX=$(shell echo $(notdir $<) | cut -c2))
	$(eval ADDR=$(shell printf "0x%X\n" $$((0x40200000+$(IDX)*0x20000)) ))
	@echo 'Entry point of "$(notdir $@)": $(ADDR)'
	@cp src/linker.ld src/linker.ld.bak
	@sed "s/BASE_ADDRESS = .*/BASE_ADDRESS = $(ADDR);/" src/linker.ld.bak > src/linker.ld
	cargo build $(BUILD_ARGS) --bin $(notdir $@)
	@mv src/linker.ld.bak src/linker.ld

$(TARGET_DIR)/%.bin: $(TARGET_DIR)/%
	$(OBJCOPY) --strip-all -O binary $< $@

$(TARGET_DIR)/%.asm: $(TARGET_DIR)/%
	$(OBJDUMP) -d $< > $@

binary: $(BINS) $(ELFS)

disasm: $(ASMS) $(ELFS)

build: binary

clean:
	@cargo clean

.PHONY: all binary disasm build clean
