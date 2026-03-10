//! # Embedded Ikin - The Flash Nuts
//!
//! Ikin adapted for no_std.
//! Stores strings in a contiguous byte slice (Flash memory) to avoid RAM allocation.
//!
//! "The Immutable Words are carved in stone (Flash), not written in sand (RAM)."

use crate::EmbeddedError;
use core::str;

/// The Flash Nuts - Code-Space Constant Pool
#[derive(Debug, Clone, Copy)]
pub struct EmbeddedIkin<'a> {
    /// The contiguous table of strings.
    /// Format: [Len(u8) | Bytes ... | Len(u8) | Bytes ...]
    /// or [Offset Table | Bytes]
    /// Simple format: Contiguous string data.
    /// Access strategy: If we have an offset table, O(1).
    /// If not, linear scan (O(N)).
    /// For embedded, we usually link an offset table or use fixed indices.
    table: &'a [u8],

    /// Optional offset table (u16 offsets) if space allows.
    /// If None, we interpret `table` as [u16_count, u16_offset_0, u16_offset_1..., DATA]
    offsets: Option<&'a [u16]>,
}

impl<'a> EmbeddedIkin<'a> {
    /// Create new wrapper around pre-flashed data
    pub fn new(table: &'a [u8], offsets: Option<&'a [u16]>) -> Self {
        EmbeddedIkin { table, offsets }
    }

    /// Consult (Read) a string from Flash
    pub fn consult(&self, index: usize) -> Result<&'a str, EmbeddedError> {
        let (start, len) = if let Some(offsets) = self.offsets {
            if index >= offsets.len() {
                return Err(EmbeddedError::MemoryOutOfBounds);
            }
            let start = offsets[index] as usize;
            // Determine length by looking at next offset or explicit length prefix?
            // Strings in Ifá embedded should likely be Pascal Strings (Len + Bytes)
            // inside the blob to be self-describing.
            // Let's assume the table points to [Len(u8), Bytes...]
            if start >= self.table.len() {
                return Err(EmbeddedError::MemoryOutOfBounds);
            }
            let len = self.table[start] as usize;
            (start + 1, len)
        } else {
            // No offsets? We can't do random access efficiently explicitly.
            return Err(EmbeddedError::HalError("Missing offset table".into()));
        };

        if start + len > self.table.len() {
            return Err(EmbeddedError::MemoryOutOfBounds);
        }

        str::from_utf8(&self.table[start..start + len]).map_err(|_| EmbeddedError::InvalidBytecode)
    }
}
