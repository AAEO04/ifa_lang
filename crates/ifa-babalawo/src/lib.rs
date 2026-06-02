//! # Ifá-Lang Babalawo
//!
//! The Babalawo (Priest) - Compile-time error checker with proverb-based messages.
//! Ported from legacy/src/errors.py

#![forbid(unsafe_code)]
#![allow(
    clippy::needless_range_loop,
    clippy::useless_format,
    clippy::if_same_then_else,
    clippy::needless_lifetimes,
    clippy::unused_enumerate_index,
    clippy::new_without_default,
    clippy::collapsible_if,
    clippy::single_match,
    clippy::collapsible_else_if,
    unused_imports,
    dead_code
)]
mod checks;
mod diagnose;
pub mod effects;
mod history;
mod infer;
mod inference;
mod iwa;
mod metadata;
mod movement;
mod scope;
mod taboo;
mod wisdom;

pub use checks::{
    BabalawoConfig, LintContext, analyze_program, check_program, check_program_with_config,
};
pub use diagnose::{Babalawo, Diagnostic, LintError, Severity};
pub use history::{StateHistoryBuffer, StateSnapshot};
pub use infer::infer_capabilities;
pub use inference::infer_expression_type;
pub use iwa::{IwaEngine, LIFECYCLE_RULES, ResourceDebt};
pub use metadata::{
    OduMethodDescriptor, domain_has_method, list_methods_for_domain, validate_odu_call,
};
pub use movement::{MoveCheckResult, MoveState, MoveTracker};
pub use scope::{Scope, ScopeChain, VarInfo};
pub use taboo::{Taboo, TabooEnforcer, TabooViolation};
pub use wisdom::{ERROR_TO_ODU, ODU_WISDOM, OduWisdom};

// Re-export Odu from core for tests
pub use ifa_types::OduDomain as Odu;
