# Oh, makefiles, makefiles, makefiles...

VERSION := 0.0.1
NAME := asyncos

# Specifies the default architecture to build (if not otherwise specified).
ARCH ?= x86_64
TARGET ?= $(ARCH)-unknown-linux-gnu

# Specify standard build paths.
SRC := src
BIN_ROOT = bin
BIN := $(BIN_ROOT)/$(ARCH)
BIN_KERNEL := $(BIN_ROOT)/kernel/$(TARGET)/debug
BIN_IMAGE := $(BIN)/image

# Detect any assembly files for our specific architecture.
ASM_SRC := $(SRC)/arch/$(ARCH)
ASM_FILES := $(wildcard $(ASM_SRC)/*.s)
ASM_OFILES := $(patsubst $(ASM_SRC)/%.s, $(BIN)/%.o, $(ASM_FILES))

# Get a reference to the grub configuration and linker script for our architecture.
LINKER_SCRIPT := $(ASM_SRC)/linker.ld
GRUB_CFG := $(ASM_SRC)/grub.cfg

# Output artifacts
KERNEL_OBJECT := $(BIN_KERNEL)/libasync_os.a
KERNEL_BINARY := $(BIN_ROOT)/$(NAME)-$(ARCH).bin
KERNEL_IMAGE := $(BIN_ROOT)/$(NAME)-$(ARCH).iso

# Enable virtualization
KVM := true

.PHONY: all build clean run image
.FORCE:

# Definitions of the phony targets.
all: build

build: $(KERNEL_BINARY)
image: $(KERNEL_IMAGE)

clean:
	rm -rf $(BIN_ROOT)

run: image
ifeq ($(KVM), true)
	qemu-system-x86_64 -smp 4 -enable-kvm -cdrom $(KERNEL_IMAGE) --serial mon:stdio
else
	qemu-system-x86_64 -smp 4-cdrom $(KERNEL_IMAGE) --serial mon:stdio
endif

debug: image
ifeq ($(KVM), true)
	qemu-system-x86_64 -s -smp 4 -enable-kvm -cdrom $(KERNEL_IMAGE) --serial mon:stdio
else
	qemu-system-x86_64 -s -smp 4 -cdrom $(KERNEL_IMAGE) --serial mon:stdio
endif

# Definitions of actual build rules.

$(ASM_OFILES) : $(BIN)/%.o : $(ASM_SRC)/%.s
	mkdir -p $(shell dirname $@)
	nasm -f elf64 $< -o $@

$(KERNEL_OBJECT): .FORCE
	cargo build --target $(TARGET)

$(KERNEL_BINARY) : $(ASM_OFILES) $(KERNEL_OBJECT) $(LINKER_SCRIPT) 
	mkdir -p $(shell dirname $(KERNEL_BINARY))
	ld -n --gc-sections -T $(LINKER_SCRIPT) -o $(KERNEL_BINARY) $(ASM_OFILES) $(KERNEL_OBJECT)

$(KERNEL_IMAGE) : $(KERNEL_BINARY) $(GRUB_CFG)
	mkdir -p $(BIN_IMAGE)/boot/grub
	cp $(KERNEL_BINARY) $(BIN_IMAGE)/boot/kernel.bin
	cp $(GRUB_CFG) $(BIN_IMAGE)/boot/grub/grub.cfg
	grub-mkrescue -o $(KERNEL_IMAGE) $(BIN_IMAGE)
