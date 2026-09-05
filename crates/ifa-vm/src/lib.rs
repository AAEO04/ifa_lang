//! # Ifá-Core
//!
//! Core VM and runtime for Ifá-Lang - The Yoruba Programming Language.
//!
//! ### 🚀 ARCHITECTURAL STATUS
//! Current release focus is **Tier 1 Conformance** and **Symmetric String Interpolation**.
//! The implementation utilizes a dedicated `Concat` opcode (`0x27`) for string concatenation, resolving the temporary overloaded `Add` architecture.
//! Refer to `patch.md` for the Phase 7 Hardening Roadmap.
//!
//! ## Modules
//!
//! - `ast` - Abstract Syntax Tree types
//! - `compiler` - AST to bytecode compilation
//! - `transpiler` - AST to Rust source for native builds
//! - `value` - IfaValue type system
//! - `bytecode` - OpCode definitions and .ifab format
//! - `vm` - Virtual machine execution
//! - `opon` - Memory management (Calabash)
//! - `error` - Error types
//! - `ebo` - Ẹbọ resource lifecycle (RAII)
//! - `ajose` - Àjọṣe reactive relationships
//! - `iwa_pele` - Ìwà Pẹ̀lẹ́ graceful error handling

pub mod actor;
pub mod ajose;
pub mod ast;
pub mod bytecode;
pub mod ebo;
pub mod error;
pub mod iwa_pele;
pub mod module_resolver;
pub mod native;
pub mod opon;
pub mod vm;

// Enhanced VM modules with Ikin & Iroke optimizations
pub mod vm_ikin;
pub mod vm_iroke;

#[cfg(all(test, feature = "compiler"))]
pub mod oracle;

// Re-exports for convenience
pub use ajose::{Ajose, Computed, RelContext, Relationship, Signal, effect};
pub use ast::{Expression, Program, Statement};
pub use bytecode::{Bytecode, OpCode};
pub use ebo::{Ebo, EboScope};
pub use error::{IfaError, IfaResult};
#[cfg(feature = "compiler")]
pub use ifa_compiler::{Compiler, compile};
#[cfg(feature = "compiler")]
pub use ifa_parser::parse;
#[cfg(feature = "compiler")]
pub use ifa_transpiler::{
    ProjectConfig, RustTranspiler, generate_project, generate_project_with_types, transpile_to_rust,
};
pub use ifa_types::IfaValue;
pub use iwa_pele::{IwaPele, IwaPeleError, IwaPeleErrorKind};
pub use module_resolver::{ImportGuard, ModuleResolver};
pub use opon::{Opon, OponError, OponErrorKind, OponResult, OponSize};
pub use vm::IfaVM;

#[cfg(feature = "compiler")]
pub mod parser {
    pub use ifa_parser::parse;
}

#[cfg(feature = "compiler")]
pub mod compiler {
    pub use ifa_compiler::{Compiler, compile};
}

#[cfg(feature = "compiler")]
pub mod transpiler {
    pub use ifa_transpiler::{ProjectConfig, RustTranspiler, generate_project, transpile_to_rust};
}
