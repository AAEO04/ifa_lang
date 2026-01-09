//! # Ifá-Lang Babalawo
//!
//! The Babalawo (Priest) - Compile-time error checker with proverb-based messages.
//! Ported from legacy/src/errors.py

mod checks;
mod diagnose;
mod history;
mod iwa;
mod taboo;
mod wisdom;

pub use checks::{LintContext, check_program};
pub use diagnose::{Babalawo, Diagnostic, IfaError, Severity};
pub use history::{StateHistoryBuffer, StateSnapshot};
pub use iwa::{IwaEngine, LIFECYCLE_RULES, ResourceDebt};
pub use taboo::{Taboo, TabooEnforcer, TabooViolation};
pub use wisdom::{ERROR_TO_ODU, ODU_WISDOM, OduWisdom};
