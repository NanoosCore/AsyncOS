//! Provides definitions for common ACPI tables, pointers, and other such structures.

use core::slice;
use core::mem;
use core::num::Wrapping;

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

/// The header for any System Description Table, containing identifying
/// information and other metadata.
#[derive(Debug)]
#[repr(packed)]
pub struct SDTHeader {
    /// The signature for this table which uniquely identifies it.
    pub signature: [u8; 4],

    /// The total length, in bytes, of this table (including entries).
    pub length: u32,

    /// The version of this table.
    pub revision: u8,

    /// The checksum used for validity; to check validity, simply sum up all the
    /// bytes in this table (with overflowing addition) and check that it equals 0.
    pub checksum: u8,

    /// A short string identifying the OEM who created this table, if any.
    pub oem_id: [u8; 6],

    /// The OEM-specific name for this table.
    pub oem_table_id: [u8; 8],

    /// The OEM-specific revision of this table.
    pub oem_revision: u32,

    /// Unsure - the id of this creator of this table, if any?
    pub creator_id: u32,

    /// Unsure - the revision according to the creator of this table, if any?
    pub creator_revision: u32
}

impl SDTHeader {
    /// Verify the checksum of the table this header belongs to, by summing up all the bytes in the header.
    /// The sum should equal 0 for the checksum to be valid.
    pub fn verify_checksum(&self) -> bool {
        let self_start = self as *const SDTHeader as *const u8;

        let mut sum = Wrapping(0u8);
        for offset in 0 .. self.length {
            sum += Wrapping(unsafe { *(self_start.offset(offset as isize)) });
        }

        sum == Wrapping(0u8)
    }
}

/// An abstract trait representing a system table; provides methods for verifying the table,
/// getting it's expected signature, and finding it's header.
pub trait SystemTable {
    /// Obtains a raw pointer to the header of the table.
    fn raw_header(&self) -> *const SDTHeader;

    /// Obtains a reference with statuc lifetime to the table.
    fn header(&self) -> &SDTHeader {
        unsafe { &*self.raw_header() }
    }

    /// Verifies the checksum of the table, ensuring it's valid.
    fn verify_checksum(&self) -> bool {
        self.header().verify_checksum()
    }

    /// Obtains the _expected_ signature this system table should have.
    /// TODO: I'd like this to return a static array [u8; 4].
    fn signature() -> &'static [u8];
}

/// The root system description table, which describes where all the other ACPI tables are in memory.
/// This is the older, 32-bit version used in ACPI v1.0 (which almost certainly is never going to show up
/// in any real 64-bit system), included for completeness.
#[repr(packed)]
#[derive(Debug)]
pub struct RSDT {
    /// The header of the RSDT table.
    pub header: SDTHeader
}

impl SystemTable for RSDT {
    fn raw_header(&self) -> *const SDTHeader {
        (&self.header) as *const SDTHeader
    }

    fn signature() -> &'static [u8] { b"RSDT" }
}

impl RSDT {
    /// Returns an iterator which iterates over all of the table entries in this root table.
    pub fn raw_tables(&self) -> RawTablesIter {
        // TODO: Almost exactly the same as XSDT raw_tables().

        let table_start = self as *const RSDT as *const u8;

        // Pointers start at the end of the table and go for the rest of the "length" field.
        // UNSAFE: Safe, as these pointers will be under 1 MB.
        let pointer_start = unsafe { table_start.offset(mem::size_of::<Self>() as isize) };
        let pointer_count = (self.header.length as usize - mem::size_of::<Self>()) / mem::size_of::<u32>();

        RawTablesIter { location: pointer_start, remaining: pointer_count, is_64_bit: false }
    }
}

/// The extended root system description table, which describes where all the other ACPI tables are in memory.
/// This is similar to the RSDT, except all the pointers are 64-bits here.
#[repr(packed)]
#[derive(Debug)]
pub struct XSDT {
    /// The header of the XSDT table.
    pub header: SDTHeader
}

impl SystemTable for XSDT {
    fn raw_header(&self) -> *const SDTHeader {
        (&self.header) as *const SDTHeader
    }

    fn signature() -> &'static [u8] { b"XSDT" }
}

impl XSDT {
    /// Returns an iterator which iterates over all of the table entries in this root table.
    pub fn raw_tables(&self) -> RawTablesIter {
        let table_start = self as *const XSDT as *const u8;

        // Pointers start at the end of the table and go for the rest of the "length" field.
        // UNSAFE: Safe, as these pointers will be under 1 MB.
        let pointer_start = unsafe { table_start.offset(mem::size_of::<Self>() as isize) };
        let pointer_count = (self.header.length as usize - mem::size_of::<Self>()) / mem::size_of::<u64>();

        RawTablesIter { location: pointer_start, remaining: pointer_count, is_64_bit: true }
    }
}

/// Provides iteration over the pointers to other tables in the RSDT/XSDT.
#[derive(Debug)]
pub struct RawTablesIter {
    /// The memory location of the next pointer to return.
    location: *const u8,

    /// The number of pointers remaining.
    remaining: usize,

    /// If true, then we're interpreting 64-bit pointers; otherwise, 32-bit pointers.
    is_64_bit: bool
}

impl Iterator for RawTablesIter {
    type Item = *const SDTHeader;

    fn next(&mut self) -> Option<Self::Item> {
        // If there are none remaining we return immediately.
        if self.remaining == 0 { return None; }

        // Otherwise, interpret the value properly and advance.
        if self.is_64_bit { 
            let ptr64 = self.location as *const u64;

            // UNSAFE: Safe, as the pointers are in physical memory under 1 MB.
            let value = unsafe { (*ptr64) as *const SDTHeader };

            self.location = unsafe { self.location.offset(mem::size_of::<u64>() as isize) };
            self.remaining = self.remaining - 1;

            Some(value)
        } else {
            let ptr32 = self.location as *const u32;

            // UNSAFE: Safe, as the pointers are in physical memory under 1 MB.
            let value = unsafe { (*ptr32) as *const SDTHeader };

            self.location = unsafe { self.location.offset(mem::size_of::<u32>() as isize) };
            self.remaining = self.remaining - 1;

            Some(value)
        }
    }
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