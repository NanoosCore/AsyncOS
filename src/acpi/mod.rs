//! Provides utilities for dealing with the ACPI System Tables, which provide a wealth of
//! information on power state, processor information, peripherals, and other hardware.
//! Note that most of the table definitions and other information were gleaned from
//! the ever useful OSDev Wiki and the most recent version of the ACPI standard, which
//! is managed by (and can be found on the website of) the UEFI committee.

mod tables;

// We do use all of the structs here and other people probably will too, so may as well import.
pub use self::tables::*;

/// Represents a handle into all of the ACPI data structures, and eases
/// information retrieval.
#[derive(Debug)]
pub enum ACPI {
    /// A version 1 ACPI table, which has 32-bit pointers.
    Version1(&'static RSDT),

    /// A version 2 (or above) ACPI table, which has 64-bit pointers.
    Version2(&'static XSDT)
}

impl ACPI {
    
    /// Attempts to locate the root ACPI table in the designated memory area and return
    /// a handle to it.
    /// UNSAFE: Unsafe, as it has to scan low physical memory to find the tables.
    pub unsafe fn find_in_memory() -> Option<ACPI> {
        // TODO: Change this to return a result, as there are multiple failure conditions.
        find_rsdp().and_then(|ptr| {
            match (*ptr).revision {
                RSDP_VERSION_1 => Some(ACPI::Version1(&*((*ptr).address as *const RSDT))),
                RSDP_VERSION_2 => {
                    // Version 2 means we're actually dealing with an XSDP.
                    let xptr = ptr as *mut XSDP;

                    Some(ACPI::Version2(&*((*xptr).address as *const XSDT)))
                },
                _ => None
            }
        })
    }

    /// Provides an iterator over all of the tables pointed to by the root system descriptor table.
    pub fn raw_tables(&self) -> RawTablesIter {
        match *self {
            ACPI::Version1(rsdt) => rsdt.raw_tables(),
            ACPI::Version2(xsdt) => xsdt.raw_tables()
        }
    }

    /// Attempt to find a table header in the root system descriptor table which has a signature
    /// matching the given signature; return a raw pointer to it.
    /// UNSAFE: Has to deference raw pointers in the root system description table and
    /// re-interpret them as pointers.
    pub unsafe fn find_raw_table(&self, signature: &[u8]) -> Option<*const SDTHeader> {
        // TODO: Use a trait for automatically borrowing as a slice.
        self.raw_tables().find(|&table_ptr| {
            &(*table_ptr).signature == signature
        })
    }

    /// Attempt to find the given system table and return a typed reference to it if it exists.
    /// UNSAFE: Has to deference raw pointers in the root system description table and
    /// re-interpet them as tables.
    pub unsafe fn find_table<T: SystemTable>(&self) -> Option<&T> {
        self.find_raw_table(T::signature()).map(|ptr| &*(ptr as *const T))
    }
}