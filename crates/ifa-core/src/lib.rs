#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_str_replace)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::type_complexity)]

//! # Ifá-Core
//!
//! Core VM and runtime for Ifá-Lang - The Yoruba Programming Language.
//!
//! ### 🚀 ARCHITECTURAL STATUS
//! Current release focus is **Tier 1 Conformance** and **Symmetric String Interpolation**.
//! The implementation utilizes an overloaded `Add` opcode as a TEMPORARY architecture.
//! Refer to `patch.md` for the Phase 7 Hardening Roadmap.
//!
//! ## Modules
//!
//! - `lexer` - Tokenization with logos
//! - `ast` - Abstract Syntax Tree types
//! - `parser` - Parsing with pest
//! - `interpreter` - Tree-walking interpreter
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

pub mod ajose;
pub mod ast;
pub mod bytecode;
pub mod compiler;
pub mod ebo;
pub mod error;
pub mod interpreter;
pub mod iwa_pele;
pub mod lexer;
pub mod module_resolver;
pub mod native;
pub mod opon;
pub mod parser;
pub mod parser_utils;
pub mod project_generator;
pub mod transpiler;
pub mod value;
pub mod vm;

// Enhanced VM modules with Ikin & Iroke optimizations
pub mod vm_ikin;
pub mod vm_iroke;

#[cfg(test)]
pub mod oracle;

// Re-exports for convenience
pub use ajose::{Ajose, Computed, RelContext, Relationship, Signal, effect};
pub use ast::{Expression, Program, Statement};
pub use bytecode::{Bytecode, OpCode};
pub use compiler::{Compiler, compile};
pub use ebo::{Ebo, EboScope};
pub use error::{IfaError, IfaResult};
pub use interpreter::Interpreter;
pub use iwa_pele::{IwaPele, IwaPeleError, IwaPeleErrorKind};
pub use lexer::{OduDomain, Token, tokenize};
pub use module_resolver::{ImportGuard, ModuleResolver};
pub use opon::{Opon, OponError, OponErrorKind, OponResult, OponSize};
pub use parser::parse;
pub use project_generator::generate_project;
pub use transpiler::transpile_to_rust;
pub use value::IfaValue;
pub use vm::IfaVM;
