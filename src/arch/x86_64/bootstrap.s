; TODO: I very much would like to change most of this to a 32-bit rust loader.
; This bootstrapper identity maps the first gigabyte and furthermore maps the 1st gigabyte after the KERNEL_VIRTUAL
; address to the first gigabyte as well.

%define INIT_STACK_SIZE 1024

; TODO: This is redefined in linker.ld (well, it almost is - this is the start of where to map, not where to
; load the kernel; here, this needs to be 512-gb aligned)
KERNEL_VIRTUAL equ 0xFFFFE00000100000

section .text
bits 32

; The 32-bit entry point for the initial processor; multiboot passes off control to this.
global asm_init32
asm_init32:
    ; Set up our initialization stack which is relatively small.
    mov esp, init_stack_top

    ; Upon entry, eax has the magic value and ebx has the multiboot structure ptr.

    ; Perform a series of system tests to ensure this CPU has
    ; any OS features we require (the biggest one being long-mode)
    ; Note that these all clobber some registers, but promise to
    ; preserve ebx (which we need!)
    call check_multiboot
    call check_cpuid
    call check_long_mode

    ; If all the checks succeed, we can switch to long mode and enable some other flags we need.
    ; This requires setting several bits in cr0/cr4, and setting up initial page tables.
    ; Note that our loader puts the kernel (in virtual memory) at 0xE000..., so we should set up the page tables
    ; to point from 0xE0000... -> 0x100000 (1 megabyte)
    ; as well as an identity map of the bootstrap code.
    call setup_page_tables
    call enable_paging

    ; After this, we're in 64-bit paging, but 32-bit "compatibility" mode. How annoying!
    ; So we load the 64-bit GDT.
    lgdt [gdt64.pointer]

    ; We can immediately update our data selectors to 64-bit.
    mov ax, gdt64.data 
    mov ss, ax
    mov ds, ax
    mov es, ax

    ; Finally, we're all raring to go, we just need to long jump
    ; to 64-bit code to update the code segment.
    jmp gdt64.code:asm_init64

; An error function which just prints out 'ERR: <letter>' to the VGA text buffer and
; peacefully dies; the letter parameter should be located in the %al register.
print_error:
    mov dword [0xb8000], 0x4f524f45 ; ER
    mov dword [0xb8004], 0x4f3a4f52 ; R:
    mov dword [0xb8008], 0x4f204f20 ; '  '
    mov byte [0x800a], al ; The above space is overwritten.
.loop:
    hlt
    jmp .loop

; Check if we were booted using multiboot by checking the magic
; number in eax.
check_multiboot:
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret
.no_multiboot:
    mov al, "0"
    jmp print_error

; Adapted from the OSDev code, which has proven highly invaluable.
; Check if CPUID is supported by attempting to flip the ID bit (bit 21)
; in the FLAGS register. If we can flip it, CPUID is available.
check_cpuid:
    ; Copy FLAGS in to EAX via stack
    pushfd
    pop eax

    ; Copy to ECX as well for comparing later on
    mov ecx, eax

    ; Flip the ID bit
    xor eax, 1 << 21

    ; Copy EAX to FLAGS via the stack
    push eax
    popfd

    ; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
    pushfd
    pop eax

    ; Restore FLAGS from the old version stored in ECX (i.e. flipping the
    ; ID bit back if it was ever flipped).
    push ecx
    popfd

    ; Compare EAX and ECX. If they are equal then that means the bit
    ; wasn't flipped, and CPUID isn't supported.
    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
    mov al, "1"
    jmp print_error

; Also adapted from OSDev code. Check if the processor supports 64-bit long mode.
check_long_mode:
    ; test if extended processor info in available
    mov eax, 0x80000000    ; implicit argument for cpuid
    cpuid                  ; get highest supported argument
    cmp eax, 0x80000001    ; it needs to be at least 0x80000001
    jb .no_long_mode       ; if it's less, the CPU is too old for long mode

    ; use extended info to test if long mode is available
    mov eax, 0x80000001    ; argument for extended processor info
    cpuid                  ; returns various feature bits in ecx and edx
    test edx, 1 << 29      ; test if the LM-bit is set in the D-register
    jz .no_long_mode       ; If it's not set, there is no long mode
    ret
.no_long_mode:
    mov al, "2"
    jmp print_error

; This code assumes that the virtual target address is aligned to 
; 512GB-boundaries or is within 1 GB of said boundaries, for now. 
; We can make this nicer, or make a rust bootloader which does all this for us.
setup_page_tables:
    ; Set up the p2 table.
    call setup_page_tables_p2

    ; Set up the p3 table, which should just map the 1st entry to the p2 table.
    mov eax, page_tables.p2
    or eax, 0b11 ; Add writable, present.
    mov [page_tables.p3], eax

    ; Set up identity mappings in p4, sets 1st index to point to p3.
    mov eax, page_tables.p3
    or eax, 0b11
    mov [page_tables.p4], eax

    ; Set up the high virtual memory mappings, pretty much exactly as before.
    ; We extract the index requested for the virtual mapping and place another entry there.
    mov ecx, (KERNEL_VIRTUAL & 0xFF8000000000) >> 39
    mov [page_tables.p4 + 8 * ecx], eax

    ret

; A utility method for filling the p2 with mappings to the 1st gigabyte of memory.
setup_page_tables_p2:
    mov ecx, 0 ; Our counter for counting the tables.
.loop:
    ; We map the nth entry to n * 2MB, eg, identity map.
    mov eax, 0x200000
    mul ecx

    ; Add the large page, present, writable flags.
    or eax, 0b10000011

    ; Place into table at correct position.
    mov [page_tables.p2 + 8 * ecx], eax

    inc ecx

    ; If we haven't gone through every entry, keep trucking...
    cmp ecx, 512
    jne .loop

    ret

; Enables paging proper once we have some page tables.
enable_paging:
    ; Load P4 to cr3 register (cpu uses this to access the P4 table)
    mov eax, page_tables.p4
    mov cr3, eax

    ; Enable PAE-flag in cr4 (Physical Address Extension)
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; Set the long mode bit in the EFER MSR (model specific register)
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; Enable paging in the cr0 register
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret

; TODO: Figure out how to reclaim this memory?
section .rodata

; Our jank temporary GDT...
gdt64:
.null: equ $ - gdt64
    dq 0 ; Null Entry
.code: equ $ - gdt64
    dq (1 << 53) | (1 << 47) | (1 << 44) | (1 << 43) | (1 << 41) ; 64-bit, present, exec, code segment, read/write
.data: equ $ - gdt64
    dq (1 << 47) | (1 << 44) | (1 << 41) ; present, data segment, read/write

; The pointer is the wierd descriptor we have to pass to lgdt to get it to actually load things properly.
; It contains the length and the pointer to the GDT (a 10 byte fat pointer).
.pointer:
    dw $ - gdt64 - 1
    dq gdt64

section .bss
align 0x1000

; The page tables used for temporary paging until we hand off to the kernel.
page_tables:
.p4:
    resb 0x1000
.p3:
    resb 0x1000
.p2:
    resb 0x1000

; The stack used during initialization and the kernel init phase;
; this stack will be dropped in favor of thread-managed stack once
; initialization is complete.
init_stack_bottom:
    resb INIT_STACK_SIZE
init_stack_top:


; ===============================
; EVERYTHING BELOW IS 64-bit code.
; ===============================

section .text
bits 64

extern rust_init

global asm_init64
asm_init64:
    ; I think we pass the 1st integer/ptr argument in rdi for System V? I hope.
    mov rdi, rbx

    ; Go into rust land.
    mov rax, rust_init
    call rax

	; If we return, we'll just halt. We shouldn't return, of course.
    hlt
