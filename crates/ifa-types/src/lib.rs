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

pub mod error;
pub mod value;
pub mod domain;
pub mod traits;

// Re-exports for convenience
pub use error::{IfaError, IfaResult, SpannedError, format_error};
pub use value::IfaValue;
pub use domain::OduDomain;
pub use traits::*;
