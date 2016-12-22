; Archaic NASM constant declarations.
%define MULTIBOOT_MAGIC 0xE85250D6
%define MULTIBOOT_ARCH 0
%define MULTIBOOT_LENGTH (multiboot_end - multiboot_start)

section .multiboot
bits 32
multiboot_start:
    dd MULTIBOOT_MAGIC ; Required multiboot 2 magic number
    dd MULTIBOOT_ARCH ; Specifies we want architecture i386
    dd MULTIBOOT_LENGTH ; Multiboot section length

    ; The checksum; we do the wierd subtraction trick to avoid a
    ; compiler complaint about out of bounds (normally, we would just negate
    ; the quantity in the parens).
    dd 0x100000000 - (MULTIBOOT_MAGIC + MULTIBOOT_ARCH + MULTIBOOT_LENGTH)

    ; Extra tags can go here, but we have none.

    ; End tag, which denotes the end of the header.
    dw 0 ; Type
    dw 0 ; Flags
    dd 8 ; Size (2 + 2 + 4 = 8)
multiboot_end:
