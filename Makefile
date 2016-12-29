# Oh, makefiles, makefiles, makefiles...

VERSION := 0.0.1
NAME := asyncos

# Specifies the default architecture to build (if not otherwise specified).
ARCH ?= x86_64
TARGET ?= $(ARCH)-unknown-linux-gnu

CC = gcc

# Specify standard build paths.
SRC := src
BIN_ROOT = bin
BIN = $(BIN_ROOT)/boot
BIN_KERNEL := $(BIN_ROOT)/kernel/$(TARGET)/debug
BIN_IMAGE := $(BIN_ROOT)/image
BOOT_SRC = $(SRC)/boot


# Assembly files
BOOT_ASM_FILES := $(wildcard $(BOOT_SRC)/*.s)
BOOT_ASM_OFILES := $(patsubst $(BOOT_SRC)/%.s, $(BIN)/%.o, $(BOOT_ASM_FILES))

# C files
BOOT_C_FILES := $(wildcard $(BOOT_SRC)/*.c)
BOOT_C_OFILES := $(patsubst $(BOOT_SRC)/%.c, $(BIN)/%.o, $(BOOT_C_FILES))

EFI_INCLUDE_PATH = /usr/include/efi
EFILIB = /usr/lib64
EFI_CRT_OFILES = $(EFILIB)/crt0-efi-$(ARCH).o

BOOT_CFLAGS ?= -std=c99 -g -O0 -Wall -Werror -I$(EFI_INCLUDE_PATH) -I$(EFI_INCLUDE_PATH)/$(ARCH) -I$(EFI_INCLUDE_PATH)/protocol  -fno-stack-protector -fpic -fshort-wchar -mno-red-zone


# Get a reference to the grub configuration and linker script for our architecture.
LINKER_SCRIPT := $(BOOT_SRC)/linker.ld
LDFLAGS = -n --gc-sections -T $(LINKER_SCRIPT)
BOOT_LDFLAGS = -nostdlib -znocombreloc -T /usr/lib64/elf_x86_64_efi.lds -shared -Bsymbolic
GRUB_CFG := $(BOOT_SRC)/grub.cfg

# Output artifacts
KERNEL_OBJECT := $(BIN_KERNEL)/libasync_os.a
KERNEL_ELF_FILE := $(BIN_ROOT)/$(NAME)-$(ARCH).elf
KERNEL_EFI_BINARY := $(BIN_ROOT)/$(NAME)-$(ARCH).efi
KERNEL_IMAGE := $(BIN_ROOT)/$(NAME)-$(ARCH).iso

# Enable virtualization
KVM := true

.PHONY: all build clean run image
.FORCE:

# Definitions of the phony targets.
all: build

build: $(KERNEL_EFI_BINARY)
image: $(KERNEL_IMAGE)

clean:
	rm -rf $(BIN_ROOT)

run: image
ifeq ($(KVM), true)
	qemu-system-x86_64 -enable-kvm -bios uefi/OVMF.fd -net none -cdrom $(KERNEL_IMAGE) --serial mon:stdio
else
	qemu-system-x86_64 -cdrom $(KERNEL_IMAGE) -bios uefi/OVMF.fd -net none--serial mon:stdio
endif

debug: image
ifeq ($(KVM), true)
	qemu-system-x86_64 -s -enable-kvm -cdrom $(KERNEL_IMAGE) --serial mon:stdio
else
	qemu-system-x86_64 -s -cdrom $(KERNEL_IMAGE) --serial mon:stdio
endif

# Definitions of actual build rules.

$(BOOT_ASM_OFILES) : $(BIN)/%.o : $(BOOT_SRC)/%.s
	mkdir -p $(shell dirname $@)
	nasm -f elf64 $< -o $@

$(BOOT_C_OFILES) : $(BIN)/%.o : $(BOOT_SRC)/%.c
	mkdir -p $(shell dirname $@)
	$(CC) $(BOOT_CFLAGS) -c $< -o $@

$(KERNEL_OBJECT): .FORCE
	cargo build --target $(TARGET)

$(KERNEL_ELF_FILE) : $(ASM_OFILES) $(C_OFILES) $(KERNEL_OBJECT) $(LINKER_SCRIPT)
	mkdir -p $(shell dirname $(KERNEL_ELF_FILE))
	ld $(LDFLAGS) $(ASM_OFILES) $(C_OFILES) $(KERNEL_OBJECT)  -o $(KERNEL_ELF_FILE)

$(KERNEL_EFI_BINARY) : $(KERNEL_ELF_FILE) $(BOOT_C_OFILES) $(BOOT_ASM_OFILES)
	ld $(EFI_CRT_OFILES) $(BOOT_LDFLAGS) $(BOOT_ASM_OFILES) $(BOOT_C_OFILES) /usr/lib64/libgnuefi.a /usr/lib64/libefi.a --format=binary $(KERNEL_ELF_FILE) --format=default -o $(KERNEL_ELF_FILE).bldr
	objcopy -j .text -j .sdata -j .data -j .dynamic -j .dynsym  -j .rel -j .rela -j .reloc --target=efi-app-$(ARCH) $(KERNEL_ELF_FILE).bldr $(KERNEL_EFI_BINARY)

$(KERNEL_IMAGE) : $(KERNEL_EFI_BINARY) $(GRUB_CFG)
	mkdir -p $(BIN_IMAGE)/boot/grub
	cp $(KERNEL_EFI_BINARY) $(BIN_IMAGE)/boot/kernel.efi
	cp $(GRUB_CFG) $(BIN_IMAGE)/boot/grub/grub.cfg
	grub-mkrescue -o $(KERNEL_IMAGE) $(BIN_IMAGE)
