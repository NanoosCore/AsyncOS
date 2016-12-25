//! Provides a definition for the MADT, or Multiple APIC Description Table, which allows for enumerating the
//! Interrupt Controllers on the system as well as some other important CPU peripherals such as all of the CPU
//! processors available.

use super::table::*;
use core::convert::From;
use core::mem;

/// The Multiple APIC Description Table, which contains information about the Interrupt Controllers on the system
/// as well as other CPU peripherals such as available processors.
#[derive(Debug)]
#[repr(packed)]
pub struct MADT {
    /// The header of this ACPI table.
    header: SDTHeader,

    /// The address of the local controller, eg, LAPIC. TODO Expand on what this is.
    pub controller_address: u32,

    /// Extra flags detailing any particular features of the hardware.
    pub flags: u32
}

// Go ahead and make the MADT a valid system table so it can be searched for.
impl SystemTable for MADT {
    fn raw_header(&self) -> *const SDTHeader {
        &self.header as *const SDTHeader
    }

    fn signature() -> &'static [u8] { b"APIC" }
}

impl MADT {
    /// Return an iterator over all of the MADT entries.
    pub fn entries(&self) -> MADTEntryIterator {
        let table_start = self as *const MADT as *const u8;

        // UNSAFE: Safe, as we're operating in valid physical memory (as otherwise I'm not sure how this
        // table would exist).
        let table_end = unsafe { table_start.offset(self.header.length as isize) };

        // UNSAFE: Safe for same reason as above. Or at least, as safe as we can be.
        let entries_start = unsafe { table_start.offset(mem::size_of::<Self>() as isize) };

        MADTEntryIterator { location: entries_start, end: table_end }
    }

    /// Retuurn an iterator over all of the processors in the MADT table.
    pub fn processors(&self) -> impl Iterator<Item=Processor> {
        self.entries().filter_map(|entry| {
            match entry {
                MADTEntry::Processor(pro) => Some(pro),
                _ => None
            }
        })
    }

    /// Return an iterator over all of the IO APICs in the MADT table.
    pub fn io_apics(&self) -> impl Iterator<Item=IOAPIC> {
        self.entries().filter_map(|entry| {
            match entry {
                MADTEntry::IOAPIC(apic) => Some(apic),
                _ => None
            }
        })
    }

    /// Return an iterator over all of the interrupt source overrides in the MADT table.
    pub fn interrupt_source_overrides(&self) -> impl Iterator<Item=InterruptSourceOverride> {
        self.entries().filter_map(|entry| {
            match entry {
                MADTEntry::InterruptSourceOverride(iso) => Some(iso),
                _ => None
            }
        })
    }
}

#[derive(Debug)]
pub struct MADTEntryIterator {
    /// The address at which the table, and thus the entries, end.
    end: *const u8,

    /// The address of the next entry to parse & return.
    location: *const u8
}

impl Iterator for MADTEntryIterator {
    type Item = MADTEntry;
    
    fn next(&mut self) -> Option<Self::Item> {
        // If we've reached the end we can quit immediately.
        if self.location == self.end { return None; }

        let header_ptr = self.location as *const MADTEntryHeader;

        let res = match unsafe { MADTEntryType::from((*header_ptr).entry_type) } {
            MADTEntryType::Processor => {
                let processor = unsafe { &*(self.location as *const MADTProcessorEntry) };

                Some(MADTEntry::Processor(Processor {
                    acpi_id: processor.processor_id,
                    apic_id: processor.apic_id,
                    flags: processor.flags
                }))
            },
            MADTEntryType::IOAPIC => {
                let ioapic = unsafe { &*(self.location as *const MADTIOAPICEntry) };

                Some(MADTEntry::IOAPIC(IOAPIC {
                    apic_id: ioapic.apic_id,
                    address: ioapic.address as *mut u8,
                    interrupt_base: ioapic.interrupt_base
                }))
            },
            MADTEntryType::InterruptSourceOverride => {
                let iso = unsafe { &*(self.location as *const MADTInterruptSourceEntry) };

                Some(MADTEntry::InterruptSourceOverride(InterruptSourceOverride {
                    bus_source: iso.bus_source,
                    irq_source: iso.irq_source,
                    interrupt: iso.interrupt,
                    flags: iso.flags
                }))
            },
            _ => {
                Some(MADTEntry::Unknown)
            }
        };

        self.location = unsafe { self.location.offset((*header_ptr).length as isize) };

        res
    }
}

/// A higher-level enumeration of the possible entries in the MADT.
#[derive(Debug, Clone, Copy)]
pub enum MADTEntry {
    /// A processor entry describing a physical processor.
    Processor(Processor),

    /// An IO APIC entry describing an IO Interrupt Controller.
    IOAPIC(IOAPIC),

    /// An Interrupt Source Override entry describing, well... that.
    InterruptSourceOverride(InterruptSourceOverride),

    /// An unknown MADT entry which we cannot parse.
    Unknown
}

/// A useful abstraction over a processor as described in the MADT.
#[derive(Debug, Clone, Copy)]
pub struct Processor {
    /// The id of the processor in the ACPI tables.
    pub acpi_id: u8,

    /// The id of the processor according to the APIC (interrupt controller); this is the one
    /// that should be used for sending interrupts to given processors.
    pub apic_id: u8,

    /// Any extra flags denoting the state or properties of the processor.
    pub flags: u32
}

impl Processor {
    /// Returns true if this processor is enabled and can be used.
    pub fn is_enabled(&self) -> bool {
        // TODO: Make a constant for this.
        (self.flags & 0x1) != 0
    }
}

/// A useful abstraction over an IO APIC as described in the MADT.
#[derive(Debug, Clone, Copy)]
pub struct IOAPIC {
    /// The id of this IO APIC according to the APIC (interrupt controller); this is the one
    /// that should be used for handling interrupts.
    pub apic_id: u8,

    /// The memory address of the start of the IO APIC's memory mapped registers.
    address: *mut u8,

    /// The interrupt base for this IO APIC. TODO: What is this?    
    interrupt_base: u32
}

/// A useful abstraction over an InterruptSourceOverride as described in the MADT.
#[derive(Debug, Clone, Copy)]
pub struct InterruptSourceOverride {
    /// The bus which the interrupt source override originates from.
    bus_source: u8,

    /// The interrupt vector which the override originates from.
    irq_source: u8,

    /// TODO: Not sure what this is ;)
    interrupt: u32,

    /// An extra flags describing the interrupt source.
    flags: u16
}

/// An enumeration of the possible types of entries in the MADT.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum MADTEntryType {
    /// A processor entry containing a processor ID and LAPIC location.
    Processor,

    /// An IO APIC entry containing the location of the IO APIC.
    IOAPIC,

    /// An interrupt source override entry containing the location of
    /// global system interrupt controllers. (TODO: Double check this one.)
    InterruptSourceOverride,

    Unknown
}

impl From<u8> for MADTEntryType {
    fn from(entry_type: u8) -> MADTEntryType {
        match entry_type {
            0 => MADTEntryType::Processor,
            1 => MADTEntryType::IOAPIC,
            2 => MADTEntryType::InterruptSourceOverride,
            _ => MADTEntryType::Unknown
        }
    }
}

/// Represents the header field of an entry in the MADT table, containing
/// a type and a total entry length.
#[derive(Debug)]
#[repr(packed)]
struct MADTEntryHeader {
    /// The type of the entry.
    entry_type: u8,

    /// The length, in bytes, of the entry.
    length: u8
}

/// A processor entry in the MADT.
#[derive(Debug)]
#[repr(packed)]
struct MADTProcessorEntry {
    /// The header of this entry, should have type Processor.
    header: MADTEntryHeader,

    /// The ACPI ID of the processor being described.
    processor_id: u8,

    /// The APIC id of the processor being described.
    apic_id: u8,

    /// Any specific flags about this processor, including whether or not
    /// it's enabled.
    flags: u32
}

/// An IO APIC entry in the MADT.
#[derive(Debug)]
#[repr(packed)]
struct MADTIOAPICEntry {
    /// The header of this entry, should have type IOAPIC.
    header: MADTEntryHeader,

    /// The APIC id of the processor being described.
    apic_id: u8,

    /// Used for padding.
    _reserved: u8,

    /// The 32-bit address of the IO APIC's memory mapped hardware registers.
    address: u32,

    /// The 32-bit global system interrupt base for this IO APIC.
    interrupt_base: u32
}

/// An Interrupt Source Override entry in the MADT.
#[derive(Debug)]
#[repr(packed)]
struct MADTInterruptSourceEntry {
    /// The header of this entry, should have type InterruptSourceOverride.
    header: MADTEntryHeader,

    /// The bus which the interrupt source override originates from.
    bus_source: u8,

    /// The interrupt vector which the override originates from.
    irq_source: u8,

    /// TODO: Not sure what this is ;)
    interrupt: u32,

    /// An extra flags describing the interrupt source.
    flags: u16
}