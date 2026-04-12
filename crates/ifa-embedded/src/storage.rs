//! # Storage Infrastructure (Tier 1)
//!
//! Adapted from `ifa-std/infra/storage.rs` for `no_std` + `alloc`.
//!
//! Uses a Flash Trait abstraction instead of `std::fs::File`.

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// flash storage trait
pub trait FlashStorage {
    fn read(&self, addr: u32, buf: &mut [u8]) -> Result<(), ()>;
    fn write(&mut self, addr: u32, data: &[u8]) -> Result<(), ()>;
    fn erase(&mut self, addr: u32) -> Result<(), ()>;
}

/// OduStore implementation for Embedded Flash
/// (Placeholder for porting logic)
pub struct OduStoreEmbedded<F: FlashStorage> {
    flash: F,
    // Index would be in RAM (HashMap requires std or hashbrown+alloc)
    // index: HashMap<String, u32>,
}
