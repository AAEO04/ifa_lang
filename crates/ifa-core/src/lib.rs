//! # Ifá-Core
//!
//! Core VM and runtime for Ifá-Lang - The Yoruba Programming Language.
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
pub mod opon;
pub mod parser;
pub mod transpiler;
pub mod value;
pub mod vm;

// Re-exports for convenience
pub use ajose::{effect, Ajose, Computed, RelContext, Relationship, Signal};
pub use ast::{Expression, Program, Statement};
pub use bytecode::{Bytecode, OpCode};
pub use compiler::{compile, Compiler};
pub use ebo::{Ebo, EboScope};
pub use error::{IfaError, IfaResult};
pub use interpreter::Interpreter;
pub use iwa_pele::{IwaPele, IwaPeleError, IwaPeleErrorKind};
pub use lexer::{tokenize, OduDomain, Token};
pub use opon::{Opon, OponError, OponErrorKind, OponResult, OponSize};
pub use parser::parse;
pub use transpiler::transpile_to_rust;
pub use value::IfaValue;
pub use vm::IfaVM;
