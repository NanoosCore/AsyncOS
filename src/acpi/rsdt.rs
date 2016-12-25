//! Provides table definitions for the Root System Description Table, which allows enumerating all of the other
//! system tables that ACPI provides.

use core::mem;

use acpi::table::*;

/// The root system description table, which describes where all the other ACPI tables are in memory.
/// This is the older, 32-bit version used in ACPI v1.0 (which almost certainly is never going to show up
/// in any real 64-bit system), included for completeness.
#[repr(packed)]
#[derive(Debug)]
pub struct RSDT {
    /// The header of the RSDT table.
    header: SDTHeader
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
    header: SDTHeader
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