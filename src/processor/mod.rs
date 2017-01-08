//! Provides abstractions over the lovely range of x86_64 processor assembly instructions,
//! data structures and controllers.

pub mod lapic;

pub use self::lapic::LAPIC;