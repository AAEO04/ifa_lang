//! # Interpreter Module
//!
//! Tree-walking interpreter that executes AST directly.
//! This module is organized into:
//! - `core.rs` - Main Interpreter implementation
//! - `environment.rs` - GPC (Grandparent-Parent-Child) scope chain
//! - `canvas.rs` - OseCanvas for ASCII graphics
//! - `handlers/` - Modular domain-specific operation handlers

mod canvas;
mod core;
mod environment;
pub mod handlers;

// Re-export main types from core
pub use self::core::{CapabilitySet, Debugger, Interpreter, Ofun};

// Re-export extracted modules
pub use canvas::OseCanvas;
pub use environment::Environment;

// Re-export handler types
pub use handlers::{HandlerRegistry, OduHandler};
