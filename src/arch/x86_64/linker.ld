ENTRY(asm_init32)
OUTPUT_FORMAT(elf64-x86-64)

/* Location of physical bootstrap code; this needs to reside within the first 64k as
we do some SMP ugliness which requires 16-bit addressable bootstrap code. Ha. */
BOOTSTRAP_PHYSICAL = 0x7C00;

/* Kernel low memory address - where it will be located in physical memory */
/* In this case, 1MB, which is a typical place to load kernels */
KERNEL_PHYSICAL = 0x100000;

/* Kernel high memory address - where it will be located in virtual memory */
/* In this case, since we're using 48-bit addressing for x86_64, we'll mount the kernel at 0xE000...0, 
which gives us 4GB for the kernel proper. */
/* The other memory regions, like 0xE0010..0 - 0xEEEEFF...F and so on are used for the kernel heap and
physical memory mappings */
KERNEL_VIRTUAL = 0xFFFFE00000100000;

SECTIONS {
    /* Start emitting the multiboot info and bootstrap info immediately. */
    . = BOOTSTRAP_PHYSICAL;

	/* The multiboot section comes first and MUST not be auto-removed. */
	.multiboot : { KEEP(bin/x86_64/multiboot.o (.multiboot)) }

	/* Next comes the bootstrap section, which is our 16bit -> 32bit -> 64bit code. */
	.bootstrap : { bin/x86_64/bootstrap.o (.text .data .rodata .bss) }

	/* After this, we load the kernel proper, which is way up there in virtual memory. */
	. = KERNEL_VIRTUAL;

	/* Followed by the kernel source. */
	.text : AT(ADDR(.text) - KERNEL_VIRTUAL + KERNEL_PHYSICAL) { 
		*(.text .text.*)
		*(EXCLUDE_FILE(bin/x86_64/bootstrap.o bin/x86_64/multiboot.o) .text)
		. = ALIGN(4096);
	}

	/* Followed by all the other sections, like data, bss, etc... */
	.data : AT(ADDR(.data) - KERNEL_VIRTUAL + KERNEL_PHYSICAL) {
		*(.data .data.*)
		*(.rodata .rodata.*)
		*(EXCLUDE_FILE(bin/x86_64/bootstrap.o bin/x86_64/multiboot.o) .data .rodata)
		. = ALIGN(4096);
	}

	.gcc_except_table : AT(ADDR(.gcc_except_table) - KERNEL_VIRTUAL + KERNEL_PHYSICAL) {
		*(.gcc_except_table .gcc_except_table.*)
		. = ALIGN(4096);
	}

	.bss : AT(ADDR(.bss) - KERNEL_VIRTUAL + KERNEL_PHYSICAL) {
		*(.bss .bss.*)
		*(EXCLUDE_FILE(bin/x86_64/bootstrap.o bin/x86_64/multiboot.o) .bss)
		. = ALIGN(4096);
	}
}

