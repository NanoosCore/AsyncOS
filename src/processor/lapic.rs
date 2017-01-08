//! Provides an abstraction over the Local Advanced Programmable Interrupt Controller,
//! which is used for interrupt handling, timing, and other specifics.
//! This implementation is based on the older APIC definitions, and does not yet support
//! the xAPIC or x2APIC standards.

use bit_field::BitField;
use core::ops::Range;
use volatile::Volatile;

/// The offset of the APIC ID register from the base address of the LAPIC.
const LAPIC_ID_REGISTER_OFFSET: usize = 0x20;
const LAPIC_ID_RANGE: Range<u8> = 24 .. 27;

/// An abstraction over the Local Advanced Programmable Interrupt Controller.
pub struct LAPIC {
    /// The address that the LAPIC is located at; should be page-aligned.
    address: u64
}

impl LAPIC {

    pub fn from_address(address: u64) -> LAPIC {
        LAPIC { address: address }
    }

    /// Returns a volatile reference to a 32-bit register at the given byte offset
    /// from the APIC base address.
    pub unsafe fn register32(&self, offset: usize) -> &mut Volatile<u32> {
        let reg_addr = (self.address as usize) + offset;

        // This pointer deferencing is the obvious unsafe part.
        &mut *(reg_addr as *mut Volatile<u32>)
    }

    /// Returns a volatile reference to the 32-bit ID register.
    pub fn id_register(&self) -> &mut Volatile<u32> {

        // UNSAFE: This register is defined in the specification to exist.
        // At least for the original APIC specification.
        unsafe { self.register32(LAPIC_ID_REGISTER_OFFSET) }
    }

    /// Returns the APIC ID of this LAPIC.
    pub fn id(&self) -> u32 {
        // Get the register, read it from the volatile reference, extract the right range of bits.
        self.id_register().read().get_range(LAPIC_ID_RANGE)
    }
}