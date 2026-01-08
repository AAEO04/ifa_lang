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

pub mod lexer;
pub mod ast;
pub mod parser;
pub mod interpreter;
pub mod compiler;
pub mod transpiler;
pub mod value;
pub mod bytecode;
pub mod vm;
pub mod opon;
pub mod error;
pub mod ebo;
pub mod ajose;
pub mod iwa_pele;

// Re-exports for convenience
pub use lexer::{Token, OduDomain, tokenize};
pub use ast::{Program, Statement, Expression};
pub use parser::parse;
pub use interpreter::Interpreter;
pub use compiler::{Compiler, compile};
pub use transpiler::transpile_to_rust;
pub use value::IfaValue;
pub use bytecode::{OpCode, Bytecode};
pub use vm::IfaVM;
pub use opon::{Opon, OponSize, OponError, OponErrorKind, OponResult};
pub use error::{IfaError, IfaResult};
pub use ebo::{Ebo, EboScope};
pub use ajose::{Signal, Computed, effect, Ajose, Relationship, RelContext};
pub use iwa_pele::{IwaPele, IwaPeleError, IwaPeleErrorKind};

