# Bootstrapping

The bootstrapping process for AsyncOS is a little magical, as it is for any 64-bit kernel. The bootstrap/loader
code can be found in src/arch/x86_64/; there are two facets to it, as well: linking and bootstrapping.

## Linking

We use a "higher-half" kernel (though as we're in 64-bit land, it's more like "higher-portion"), which is located at
0xE00000100000; however, it needs to be loaded in physical memory starting at 0x100000. Furthermore, the bootstrap code itself
needs to be loaded in low, 16-bit addressable memory (0x7C00) as we need the other processors to access it while they're in 16-bit mode.

To accomplish this mess, we set up the linker script (linker.ld) to do the following:

1. Link the bootstrap code in multiboot.asm and bootstrap.asm, and place them in low physical memory (0x7C00).
2. Then, link the kernel code; locate it in high virtual memory (0xE00000100000), but LOAD it into low physical memory (0x100000). The virtual memory mappings
created during bootstrapping will ensure that the high virtual memory works when the kernel starts.

The multiboot loader will take care of the rest for us, loading everything properly and throwing us into 32-bit mode. From there, we move onto..

## Bootstrapping

The bootstrapping process takes off from when multiboot hands over control to us; here, we perform the following steps:

1. Check several cpu flags and multiboot metadata to ensure we can, in fact, switch to long mode on this device.
2. Set up the page tables, identity mapping low memory (so we can enable paging without triple faulting due to a failed instruction fetch),
    and then map the high virtual memory to the low physical memory where the kernel is actually located.
    - We first set up a 2nd-level page table which maps all of the 1st gigabyte of memory; then, we create a 3rd-level table
     whose first entry points to this 2nd-level page table. Finally, we create the top-level table whose first entry points to
     the 3rd level table (creating an identity mapping), and whose entry at 0xE000...0 also points to that 3rd level table
     (creating the virtual mapping to the 1st gigabyte of physical memory).
3. Switch to the extended 32-bit mode, and then set up the 64-bit GDT/IDT which gives us just enough to long-jump into 64-bit mode.
4. Hand off control to the kernel proper, which can then set up nicer and more permanent mappings and install it's own data tables/structures.