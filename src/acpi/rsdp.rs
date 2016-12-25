//! Provides the definition of the Root System Description Pointer, as well as it's 64-bit variant.

use core::slice;

/// The unique signature which identifies the RSDP.
pub const RSDP_SIGNATURE: &'static [u8] = b"RSD PTR ";

/// The location where the segment pointer to the extended bios location can be found.
pub const EXTENDED_BIOS_AREA_POINTER_LOC: *const u16 = 0x40E as *const u16;

/// The maximum size to look at in the extended bios data area.
pub const EXTENDED_BIOS_AREA_MAX_SIZE: usize = 1 << 10;

/// The starting location to look for the RSDP at.
pub const RSDP_LOCATION_START: usize = 0xE0000;

/// The ending location to look for the RSDP at.
pub const RSDP_LOCATION_END: usize = 0x100000;

/// The version number corresponding to version 1 of ACPI.
pub const RSDP_VERSION_1: u8 = 0;

/// The version number corresponding to version 2.0 and above of ACPI.
pub const RSDP_VERSION_2: u8 = 2;

/// The Root System Description Pointer for ACPI v1.0, which is located somewhere in low memory and
/// is the entry point for the ACPI interface between the operating system and hardware
/// peripherals/power control.
///
/// The RSDP should be 16-byte aligned.
#[derive(Debug)]
#[repr(packed)]
pub struct RSDP {
    /// The unique signature for finding the RSDP, is equal to RSDP_SIGNATURE.
    pub signature: [u8; 8],

    /// A checksum used for verifying the table is valid. Verification is done
    /// by checking that the sum of the bytes in this structure (including
    /// the checksum) are equal to 0 (mod 255).
    pub checksum: u8,

    /// The unique OEMID for the company/individuals who created the device.
    pub oem_id: [u8; 6],

    /// The revision/version of ACPI. A value of 0 implies ACPI 1.0,
    /// and a value of 2 implies ACPI 2.0 to ACPI 6.1.
    pub revision: u8,

    /// A physical pointer to the Root System Description Table; this is only 32-bits,
    /// which may not be large enough for 64-bit systems.
    pub address: u32
}

/// The eXtended Root System Description Pointer for ACPI v2.0 and above; it
/// contains all of the same fields as the RSDP, except it adds a length field
/// and provides a 64-bit pointer to the XSDT.
#[derive(Debug)]
#[repr(packed)]
pub struct XSDP {
    /// The unique signature for finding the RSDP, is equal to RSDP_SIGNATURE.
    pub signature: [u8; 8],

    /// A checksum used for verifying the table is valid. Verification is done
    /// by checking that the sum of the bytes in this structure (including
    /// the checksum) are equal to 0 (mod 255).
    pub checksum: u8,

    /// The unique OEMID for the company/individuals who created the device.
    pub oem_id: [u8; 6],

    /// The revision/version of ACPI. A value of 0 implies ACPI 1.0,
    /// and a value of 2 implies ACPI 2.0 to ACPI 6.1.
    pub revision: u8,

    /// On version 2.0 of ACPI and above, this should not be used in favor of the
    /// 64-bit XSDT address.
    _rsdt_address: u32,

    /// The length of the entire XSDT table (as far as I'm aware).
    pub length: u32,

    /// The 64-bit address of the XSDT table.
    pub address: u64,

    /// An additional checksum to balance out the additional information in this.
    pub extended_checksum: u8,

    /// Reserved space to 4-byte align the size of this structure.
    _reserved: [u8; 3]
}


/// Obtains the starting memory location of the extended bios data area.
pub unsafe fn extended_bios_data_area_start() -> *mut u8 {
    let actual_ptr = ((*EXTENDED_BIOS_AREA_POINTER_LOC) as usize) << 4;

    actual_ptr as *mut u8
}

/// Attempts to find the RSDP by looking at the defined regions
/// in memory where it should be located (see RSDP_LOCATION_START, and extended_bios_data_area_start).
pub unsafe fn find_rsdp() -> Option<*mut RSDP> {
    let ebda_start = extended_bios_data_area_start() as usize;

    // Yay for iterators; this steps in 16-byte intervals looking for the 8-byte signature
    // of the RSDP, first checking the RSDP location and then checking the extended bios area.
    // This is much better than anything I would get to write in C++ without some horrifying
    // template metaprogramming hacks...
    (RSDP_LOCATION_START .. RSDP_LOCATION_END).step_by(16)
        .chain((ebda_start .. (ebda_start + EXTENDED_BIOS_AREA_MAX_SIZE)).step_by(16))
        .find(|&mem_location| {
            // Make up a slice out of nothing at the given memory location, comparing it against the
            // RSDP signature.
            let raw_slice = slice::from_raw_parts(mem_location as *const u8, RSDP_SIGNATURE.len());

            raw_slice == RSDP_SIGNATURE
        })
        .map(|loc| loc as *mut RSDP)
}