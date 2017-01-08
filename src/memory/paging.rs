//! Provides utilities for dealing with amd64 paging, including an abstraction for a "virtual address space"
//! which contains all virtual memory mappings and which can be swapped in/out.

use bit_field::BitField;
use core::ops::Range;

/// The bit index of the present bit in a page entry.
const PAGE_ENTRY_PRESENT_BIT: u8 = 0;

/// The bit index of the read/write bit in a page entry.
const PAGE_ENTRY_WRITABLE_BIT: u8 = 1;

/// The bit index of the user/supervisor bit in a page entry.
const PAGE_ENTRY_USER_BIT: u8 = 2;

/// The bit index of the write-through bit in a page entry.
const PAGE_ENTRY_WRITE_THROUGH_BIT: u8 = 3;

/// The bit index of the cache disable bit in a page entry.
const PAGE_ENTRY_CACHE_DISABLE_BIT: u8 = 4;

/// The bit index of the accessed bit in a page entry.
const PAGE_ENTRY_ACCESSED_BIT: u8 = 5;

/// The bit index of the dirty bit in a page entry.
const PAGE_ENTRY_DIRTY_BIT: u8 = 6;

/// The bit index of the page size bit in a page entry.
const PAGE_ENTRY_PAGE_SIZE_BIT: u8 = 7;

/// The bit index of the global bit in a page entry.
const PAGE_ENTRY_GLOBAL_BIT: u8 = 8;

/// The bit range where the actual physical page address is located in a page entry.
const PAGE_ENTRY_PAGE_BITS: Range<u8> = 12 .. 51;


/// The size of an x86_64 page, in bytes.
const PAGE_SIZE: usize = 4096;

/// The number of page entries which can fit into a page.
/// TODO: Usage of 8 as a constant is bad, bad, bad.
const PAGE_ENTRY_COUNT: usize = PAGE_SIZE / 8;

/// A struct representing a page entry in one of the page tables.
/// These entries are mutable.
#[derive(Debug, Clone, Copy)]
pub struct PageEntry(u64);

impl PageEntry {
    /// Returns true if this page entry is present (eg, points to a valid
    /// page), and false otherwise.
    pub fn is_present(&self) -> bool {
        self.0.get_bit(PAGE_ENTRY_PRESENT_BIT)
    }

    /// Sets the present status of the entry to the given value `value`
    pub fn set_present(&mut self, value: bool) -> &mut Self {
        self.0.set_bit(PAGE_ENTRY_PRESENT_BIT, value);
        self
    }

    /// Returns true if this page allows writes, and false if it is read-only.
    pub fn is_writable(&self) -> bool {
        self.0.get_bit(PAGE_ENTRY_WRITABLE_BIT)
    }

    /// Sets the writable status of this page to the given value.
    pub fn set_writable(&mut self, value: bool) -> &mut Self {
        self.0.set_bit(PAGE_ENTRY_WRITABLE_BIT, value);
        self
    }

    /// Returns true if this page is usermode accessible, and false otherwise.
    pub fn is_user(&self) -> bool {
        self.0.get_bit(PAGE_ENTRY_USER_BIT)
    }

    /// Sets the page usermode accessibility to the given value.
    pub fn set_user(&mut self, value: bool) -> &mut Self {
        self.0.set_bit(PAGE_ENTRY_WRITABLE_BIT, value);
        self
    }

    /// Returns true if all writes to this page are written through the cache
    /// to main memory, and false otherwise.
    pub fn is_write_through(&self) -> bool {
        self.0.get_bit(PAGE_ENTRY_WRITE_THROUGH_BIT)
    }

    /// Sets the cache write-through status of this page to the given value.
    pub fn set_write_through(&mut self, value: bool) -> &mut Self {
        self.0.set_bit(PAGE_ENTRY_WRITE_THROUGH_BIT, value);
        self
    }

    /// Returns true if no caching is done for this page, and false otherwise.
    pub fn is_cache_disabled(&self) -> bool {
        self.0.get_bit(PAGE_ENTRY_CACHE_DISABLE_BIT)
    }

    /// Sets the caching status of the page to the given value.
    pub fn set_cache_disabled(&mut self, value: bool) -> &mut Self {
        self.0.set_bit(PAGE_ENTRY_CACHE_DISABLE_BIT, value);
        self
    }

    /// Returns true if the system has accessed this page by either a
    /// write OR a read, and false otherwise.
    pub fn is_accessed(&self) -> bool {
        self.0.get_bit(PAGE_ENTRY_ACCESSED_BIT)
    }

    /// Updates the 'accessed' flag of this page entry to the given value.
    pub fn set_access(&mut self, value: bool) -> &mut Self {
        self.0.set_bit(PAGE_ENTRY_ACCESSED_BIT, value);
        self
    }

    /// Returns true if the system has written data to this page, and false otherwise.
    pub fn is_dirty(&self) -> bool {
        self.0.get_bit(PAGE_ENTRY_DIRTY_BIT)
    }

    /// Updates the 'dirty' status of this page entry to the given value.
    pub fn set_dirty(&mut self, value: bool) -> &mut Self {
        self.0.set_bit(PAGE_ENTRY_DIRTY_BIT, value);
        self
    }

    /// Returns true if this entry (if it's located in a table other than the 
    /// bottom level Page Tables) is a mapping to a large page, and false otherwise.
    pub fn is_large_page(&self) -> bool {
        self.0.get_bit(PAGE_ENTRY_PAGE_SIZE_BIT)
    }

    /// Sets the 'large page' status of this page entry to the given value.
    pub fn set_large_page(&mut self, value: bool) -> &mut Self {
        self.0.set_bit(PAGE_ENTRY_PAGE_SIZE_BIT, value);
        self
    }

    /// Returns true if this entry is global, eg, will not be evicted when the
    /// TLB is flushed on a cr3 change; and false otherwise.
    pub fn is_global(&self) -> bool {
        self.0.get_bit(PAGE_ENTRY_GLOBAL_BIT)
    }

    /// Sets the 'global' status of this page entry to the given value.
    pub fn set_global(&mut self, value: bool) -> &mut Self {
        self.0.set_bit(PAGE_ENTRY_GLOBAL_BIT, value);
        self
    }

    /// Returns the frame _number_ that this mapping maps to.
    pub fn frame_number(&self) -> u64 {
        self.0.get_range(PAGE_ENTRY_PAGE_BITS)
    }

    /// Updates the frame _number_ that this mapping maps to.
    pub fn set_frame_number(&mut self, frame: u64) -> &mut Self {
        self.0.set_range(PAGE_ENTRY_PAGE_BITS, frame);
        self
    }
}

/// A struct representing a page table at some level in the paging structure.
pub struct PageTable {
    /// The array of all page entries in this table.
    entries: [PageEntry; PAGE_ENTRY_COUNT]
}