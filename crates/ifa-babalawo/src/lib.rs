//! # Ifá-Lang Babalawo
//!
//! The Babalawo (Priest) - Compile-time error checker with proverb-based messages.
//! Ported from legacy/src/errors.py

mod wisdom;
mod diagnose;
mod checks;
mod iwa;
mod taboo;
mod history;

pub use wisdom::{OduWisdom, ODU_WISDOM, ERROR_TO_ODU};
pub use diagnose::{Babalawo, IfaError, Severity, Diagnostic};
pub use checks::{LintContext, check_program};
pub use iwa::{IwaEngine, ResourceDebt, LIFECYCLE_RULES};
pub use taboo::{TabooEnforcer, Taboo, TabooViolation};
pub use history::{StateHistoryBuffer, StateSnapshot};
