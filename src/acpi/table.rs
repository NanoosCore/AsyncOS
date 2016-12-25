//! Provides the basic abstraction for system tables and SDT headers.

use core::num::Wrapping;

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