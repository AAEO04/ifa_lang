//! # ifa-types - Shared Types for Ifá-Lang
//!
//! This crate provides the shared type system used across all Ifá-Lang runtimes:
//! - Interpreter (ifa-core)
//! - Native compilation (ifa-std)
//! - Bytecode VM (ifa-vm)
//! - Transpiler output
//!
//! ## Core Types
//!
//! - [`IfaValue`] - Dynamic value container
//! - [`IfaError`] / [`IfaResult`] - Error handling
//! - [`OduDomain`] - The 16 Odù domains + infrastructure/stacks

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

// Unused imports from std/alloc removed (re-exports handling types internally)

pub mod domain;
pub mod error;
pub mod shared;
pub mod token;
pub mod traits;
pub mod value;
pub mod value_union; // Unified Type System (Internal)

#[cfg(feature = "vm")]
pub mod ast;

#[cfg(feature = "vm")]
pub mod bytecode;

// Re-exports for convenience
pub use domain::OduDomain;
pub use error::{IfaError, IfaResult, SpannedError, format_error};
pub use shared::IfaShared;
pub use token::ResourceToken;
pub use traits::*;
// pub use value::IfaValue; // Old Enum
pub use value_union::IfaValue; // New Tagged Union

#[cfg(feature = "vm")]
pub use ast::Statement;

// Re-export OpCode from the micro-crate
pub use ifa_bytecode::{ErrorCode, InvalidOpCode, OpCode};

#[cfg(feature = "vm")]
pub use bytecode::Bytecode;
// std-only Extensions for OpCode
#[cfg(feature = "std")]
mod opcode_ext {
    // OpCode Display impl is now in ifa-bytecode directly
}
