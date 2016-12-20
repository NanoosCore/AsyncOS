# Oh, makefiles, makefiles, makefiles...

VERSION := 0.0.1
NAME := asyncos

# Specifies the default architecture to build (if not otherwise specified).
ARCH ?= x86_64

# Specify standard build paths.
SRC := src
BIN_ROOT = bin
BIN := $(BIN_ROOT)/$(ARCH)
IMAGE_BIN := $(BIN)/image
ASM_SRC := $(SRC)/arch/$(ARCH)

# Detect any assembly files for our specific architecture.
ASM_FILES := $(wildcard $(ASM_SRC)/*.asm)
ASM_OFILES := $(patsubst $(ASM_SRC)/%.asm, $(BIN)/%.o, $(ASM_FILES))

# Get a reference to the grub configuration and linker script for our architecture.
LINKER_SCRIPT := $(ASM_SRC)/linker.ld
GRUB_CFG := $(ASM_SRC)/grub.cfg

# Output artifacts
KERNEL_BINARY := $(BIN_ROOT)/$(NAME)-$(ARCH).bin
KERNEL_IMAGE := $(BIN_ROOT)/$(NAME)-$(ARCH).iso

# Enable virtualization
KVM := true

.PHONY: all build clean run image

# Definitions of the phony targets.
all: build

build: $(KERNEL_BINARY)
image: $(KERNEL_IMAGE)

clean:
	rm -rf $(BIN_ROOT)

run: image
ifeq ($(KVM),true)
	qemu-system-x86_64 -enable-kvm -cdrom $(KERNEL_IMAGE)
else
	qemu-system-x86_64 -cdrom $(KERNEL_IMAGE)
endif

# Definitions of actual build rules.

$(ASM_OFILES) : $(BIN)/%.o : $(ASM_SRC)/%.asm
	mkdir -p $(shell dirname $@)
	nasm -f elf64 $< -o $@

$(KERNEL_BINARY) : $(ASM_OFILES) $(LINKER_SCRIPT)
	mkdir -p $(shell dirname $(KERNEL_BINARY))
	ld -n -T $(LINKER_SCRIPT) -o $(KERNEL_BINARY) $(ASM_OFILES)

$(KERNEL_IMAGE) : $(KERNEL_BINARY) $(GRUB_CFG)
	mkdir -p $(IMAGE_BIN)/boot/grub
	cp $(KERNEL_BINARY) $(IMAGE_BIN)/boot/kernel.bin
	cp $(GRUB_CFG) $(IMAGE_BIN)/boot/grub/grub.cfg
	grub-mkrescue -o $(KERNEL_IMAGE) $(IMAGE_BIN)
