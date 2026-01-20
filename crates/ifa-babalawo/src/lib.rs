//! # Ifá-Lang Babalawo
//!
//! The Babalawo (Priest) - Compile-time error checker with proverb-based messages.
//! Ported from legacy/src/errors.py

mod checks;
mod diagnose;
mod history;
mod infer;
mod iwa;
mod taboo;
mod wisdom;

pub use checks::{BabalawoConfig, LintContext, check_program, check_program_with_config};
pub use diagnose::{Babalawo, Diagnostic, IfaError, Severity};
pub use infer::infer_capabilities;
pub use history::{StateHistoryBuffer, StateSnapshot};
pub use iwa::{IwaEngine, LIFECYCLE_RULES, ResourceDebt};
pub use taboo::{Taboo, TabooEnforcer, TabooViolation};
pub use wisdom::{ERROR_TO_ODU, ODU_WISDOM, OduWisdom};

// Re-export Odu from core for tests
pub use ifa_core::lexer::OduDomain as Odu;
