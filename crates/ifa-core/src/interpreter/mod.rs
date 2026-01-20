//! # Interpreter Module
//!
//! Tree-walking interpreter that executes AST directly.
//! This module is organized into:
//! - `core.rs` - Main Interpreter implementation
//! - `environment.rs` - GPC (Grandparent-Parent-Child) scope chain
//! - `canvas.rs` - OseCanvas for ASCII graphics
//! - `handlers/` - Modular domain-specific operation handlers

mod core;
mod environment;
mod canvas;
pub mod handlers;

// Re-export main types from core
pub use self::core::{Interpreter, CapabilitySet, Ofun, Debugger};

// Re-export extracted modules
pub use environment::Environment;
pub use canvas::OseCanvas;

// Re-export handler types
pub use handlers::{OduHandler, HandlerRegistry};
