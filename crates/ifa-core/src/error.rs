//! # Ifá Error Types (Ọ̀kànràn)
//!
//! Re-exports from ifa-types. ifa-types is the single source of truth.
//! SpannedError and format_error live here for convenience (they need std).

// THE canonical error type — re-exported from ifa-types
pub use ifa_types::{IfaError, IfaResult, SpannedError, format_error};
