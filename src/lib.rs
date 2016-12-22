#![feature(lang_items)]
#![no_std]

extern crate rlibc;

#[no_mangle]
pub extern fn rust_init() {
    let okay = 0x2f592f412f4b2f4fu64;

    // Directly dump the data into the VGA buffer, which (for now) is identity mapped.
    unsafe { *(0xB8000 as *mut u64) = okay; }
}

// We have an empty personality for now.
#[lang = "eh_personality"] 
pub extern fn eh_personality() {}

// Our default implementation will simply diverge.
#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt() -> ! {
    loop { }
}

// To deal with some assumptions libraries make about
// unwinding being available.
#[allow(non_snake_case)]
#[no_mangle]
pub extern fn _Unwind_Resume() -> ! {
    loop {}
}

