//! # Bytecode Module
//!
//! OpCode definitions and .ifab bytecode format for Ifá-Lang VM.
//! Note: Core definitions have moved to `ifa-types`.

// Re-export everything from ifa-types::bytecode (Bytecode struct, constants)
pub use ifa_types::bytecode::*;

// Re-export OpCode explicitly for ifa-core internal usage
// This satisfies `use crate::bytecode::OpCode` in other modules
pub use ifa_types::OpCode;
