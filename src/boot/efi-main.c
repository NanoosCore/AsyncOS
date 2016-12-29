#include <efi.h>
#include <efilib.h>
#include <stdint.h>
#include <stddef.h>
#include <elf.h>

// Pointers to the kernel image
extern char _binary_bin_asyncos_x86_64_elf_start;
extern char _binary_bin_asyncos_x86_64_elf_end;
extern uint64_t _binary_bin_asyncos_x86_64_elf_size;

// This should be accessible from Rust
struct boot_state {
    UINTN                 memory_map_size;
    EFI_MEMORY_DESCRIPTOR *memory_map;
    UINTN                 map_key;
    UINTN                 descriptor_size;
    UINT32                descriptor_version;
};

struct boot_state boot_state;

EFI_STATUS
EFIAPI efi_main (EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE *SystemTable) {
    InitializeLib(ImageHandle, SystemTable);
    char* kernel_start = &_binary_bin_asyncos_x86_64_elf_start;
    char* kernel_end = &_binary_bin_asyncos_x86_64_elf_end;
    Print(L"AsyncOS UEFI loader: Loading kernel image at 0x%x-0x%x\n", kernel_start, kernel_end);

    Elf64_Ehdr* kernel_header = (Elf64_Ehdr*) kernel_start;
    /* Verify the ELF header */
    if (!(kernel_header->e_ident[0] == ELFMAG0 && kernel_header->e_ident[1] == ELFMAG1 && kernel_header->e_ident[2] == ELFMAG2 && kernel_header->e_ident[3] == ELFMAG3)) {
        Print(L"Invalid ELF magic number.\n");
        while (1);
        return EFI_LOAD_ERROR;
    }
    Print(L"... Valid ELF magic number\n");

    if (kernel_header->e_ident[EI_CLASS] != ELFCLASS64) {
        Print(L"Unsupported ELF File Class.\n");
        while (1);
        return EFI_LOAD_ERROR;
    }
    Print(L"... Valid ELF class\n");
    if (kernel_header->e_ident[EI_DATA] != ELFDATA2LSB) {
        Print(L"Unsupported ELF File byte order.\n");
        while (1);
        return EFI_LOAD_ERROR;
    }
    Print(L"... Valid ELF byte order\n");
    if (kernel_header->e_machine != EM_X86_64) {
        Print(L"Unsupported ELF File target.\n");
        while (1);
        return EFI_LOAD_ERROR;
    }
    Print(L"... Valid ELF target\n");
    if (kernel_header->e_ident[EI_VERSION] != EV_CURRENT) {
        Print(L"Unsupported ELF File version.\n");
        while (1);
        return EFI_LOAD_ERROR;
    }
    Print(L"... Valid ELF version\n");
    if (kernel_header->e_type != ET_EXEC) {
        Print(L"Unsupported ELF File type.\n");
        while (1);
        return EFI_LOAD_ERROR;
    }
    Print(L"... Valid ELF file type\n");
    Print(L"Found valid ELF file!\n");
    uint64_t current_offset = kernel_header->e_phoff;
    Print(L"Parsing ELF file starting at phoff 0x%x, phnum: %d\n", current_offset, kernel_header->e_phnum);
    for (uint64_t i = 0; i < kernel_header->e_phnum; i++) {
        Elf64_Phdr* program_header = (Elf64_Phdr*) (kernel_start + current_offset);
        current_offset += kernel_header->e_phentsize;

        if (program_header->p_type == PT_LOAD) {
            uint8_t* target_vmem = (uint8_t*) program_header->p_vaddr;
            uint8_t* target_pmem = (uint8_t*) program_header->p_paddr;
            uint64_t memory_size = program_header->p_memsz;
            uint64_t file_size = program_header->p_filesz;
            uint64_t file_offset = program_header->p_offset;

            Print(L"Found PT_LOAD segment: paddr: 0x%lx, vaddr: 0x%lx, memsz: %d, filesz: %d, p_offset: %d\n", target_pmem, target_vmem, memory_size, file_size, file_offset);
            /* TODO: copy kernel to physical address */
        } else {
            Print(L"Found section type 0x%x\n", program_header->p_type);
        }
    }
    Print(L"ELF entry point: 0x%lx\n", kernel_header->e_entry);

    boot_state.memory_map = LibMemoryMap(&boot_state.memory_map_size,
                                         &boot_state.map_key,
                                         &boot_state.descriptor_size,
                                         &boot_state.descriptor_version);

    uefi_call_wrapper((void*) SystemTable->BootServices->ExitBootServices, 2, ImageHandle, boot_state.map_key);

    // TODO: set up higher-half mapping and jump to rust_init.

    uefi_call_wrapper((void*) SystemTable->RuntimeServices->SetVirtualAddressMap, 4, boot_state.memory_map_size, boot_state.descriptor_size, boot_state.descriptor_version, boot_state.memory_map);

    uefi_call_wrapper((void*)SystemTable->RuntimeServices->ResetSystem, 4, EfiResetShutdown, EFI_SUCCESS, 0, NULL);

    while (1); // We can't return to GRUB since we no longer have EFI boot services
    return EFI_SUCCESS;
}
