#![feature(lang_items)]
#![feature(step_by)]
#![feature(unique)]
#![feature(const_fn)]
#![feature(conservative_impl_trait)]
#![feature(asm)]
#![no_std]

extern crate rlibc;
extern crate spin;
extern crate volatile;
extern crate multiboot2;

pub mod acpi;

#[macro_use]
pub mod vga;

use core::str;

/// The rust entry point for the initial processor into the kernel.
#[no_mangle]
pub extern "C" fn rust_init(multiboot_header: *mut u8) {
    color_println!(vga::Color::Magenta, "AsyncOS Version {}\n", "0.0.1");

    color_println!(vga::Color::Magenta, "- Multiboot Metadata @ 0x{0:x}", multiboot_header as u64);

    if let Some(acpi) = unsafe { acpi::ACPI::find_in_memory() } {
        println!("- ACPI: Present");
        println!("- ACPI: {} tables available:", acpi.raw_tables().count());

        for table in acpi.raw_tables() {
            let header = unsafe { &*table };

            println!("\t- {} @ {1:x}", str::from_utf8(&header.signature).unwrap(), table as u64);
        }

        if let Some(madt) = unsafe { acpi.find_table::<acpi::MADT>() } {
            println!("- MADT: {} processors available", madt.processors().count());

            for entry in madt.processors() {
                println!("\t- {:?}", entry);
            }
        }
    } else {
        color_println!(vga::Color::Red, "- ACPI: Absent");
    }

    // The OS HAS CONTROL NOW. No premature exiting for us.
    loop { unsafe { asm!("hlt" :::: "volatile"); } }
}

/// Method used for the compilers personality, though I'm not sure what it is.
#[lang = "eh_personality"] 
pub extern fn eh_personality() {}

/// Formats panic messages; unused due to the kernel aborting on panic.
#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt() -> ! {
    loop { /* :( */ }
}

/// Some precompiled libaries assume the existence of this symbol, so we
/// provide a diverging implementation.
#[allow(non_snake_case)]
#[no_mangle]
pub extern fn _Unwind_Resume() -> ! {
    loop { /* :( */ }
}
