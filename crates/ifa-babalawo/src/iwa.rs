//! # Ìwà Engine - Resource Lifecycle Validation
//!
//! Ensures every opening action has a corresponding closing action.
//! "The Babalawo at the Gate" - ported from legacy/src/validator.py

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Lifecycle rules: opener -> closer
/// Resources that are opened must be closed
pub static LIFECYCLE_RULES: Lazy<HashMap<&'static str, Option<&'static str>>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // File I/O (Òdí)
    m.insert("odi.si", Some("odi.pa")); // Open -> Close
    m.insert("odi.ko", None); // Write is auto-closing

    // Network (Òtúrá)
    m.insert("otura.de", Some("otura.pa")); // Bind -> Close
    m.insert("otura.so", Some("otura.pa")); // Connect -> Close

    // Memory (Ògúndá/Ìrẹtẹ̀)
    m.insert("ogunda.ge", Some("irete.tu")); // Alloc -> Free
    m.insert("ogunda.da", Some("irete.tu")); // Create -> Free

    // Objects (Òfún)
    m.insert("ofun.da", Some("ofun.pa")); // Create -> Delete

    // System (Ogbè/Ọ̀yẹ̀kú)
    m.insert("ogbe.bi", Some("oyeku.duro")); // Init -> Halt
    m.insert("ogbe.bere", Some("oyeku.duro")); // Start -> Stop

    // Graphics (no close needed)
    m.insert("ose.nu", None);

    // Loops
    m.insert("iwori.yipo", Some("iwori.pada")); // Loop -> Return

    // Ẹbọ-aware (Sacrifice blocks)
    m.insert("ebo.begin", Some("ebo.sacrifice"));
    m.insert("ase.begin", Some("ase.end"));

    m
});

/// Resources that auto-close at program end
pub static AUTO_CLOSE: &[&str] = &["ogbe.bi", "ogbe.bere"];

/// A resource debt - something opened that needs closing
#[derive(Debug, Clone)]
pub struct ResourceDebt {
    pub opener: String,
    pub required: String,
    pub line: usize,
    pub column: usize,
}

/// A borrow debt - a reference that must be released before reborrow
/// Implements simplified Rust-like borrow checking rules
#[derive(Debug, Clone)]
pub struct BorrowDebt {
    /// Variable being borrowed
    pub var_name: String,
    /// True if mutable borrow (&mut), false if immutable (&)
    pub is_mutable: bool,
    /// Line where borrow occurred
    pub line: usize,
    /// Column where borrow occurred
    pub column: usize,
    /// Scope depth when borrow started
    pub scope_depth: usize,
}

/// Borrow checking result
#[derive(Debug, Clone, PartialEq)]
pub enum BorrowError {
    /// Already mutably borrowed
    AlreadyMutablyBorrowed { var: String, existing_line: usize },
    /// Already immutably borrowed, cannot mutably borrow
    ImmutableBorrowExists { var: String, existing_line: usize },
    /// Cannot use value while borrowed
    ValueBorrowed { var: String, borrow_line: usize },
}

/// The Ìwà Engine - ensures resource lifecycle balance
/// Also tracks borrow lifetimes for Ref/RefMut types
#[derive(Debug)]
pub struct IwaEngine {
    pub strict_mode: bool,
    /// Resource lifecycle debts (open -> close)
    pub debt_ledger: Vec<ResourceDebt>,
    /// Active borrows (references)
    pub borrow_ledger: Vec<BorrowDebt>,
    /// Current scope depth for borrow tracking
    pub scope_depth: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Default for IwaEngine {
    fn default() -> Self {
        Self::new(true)
    }
}

impl IwaEngine {
    pub fn new(strict_mode: bool) -> Self {
        Self {
            strict_mode,
            debt_ledger: Vec::new(),
            borrow_ledger: Vec::new(),
            scope_depth: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Enter a new scope (function, block, ailewu)
    pub fn enter_scope(&mut self) {
        self.scope_depth += 1;
    }

    /// Exit current scope - releases all borrows in this scope
    pub fn exit_scope(&mut self) {
        self.borrow_ledger
            .retain(|b| b.scope_depth < self.scope_depth);
        self.scope_depth = self.scope_depth.saturating_sub(1);
    }

    /// Try to create an immutable borrow (&T)
    /// Returns error if there's an existing mutable borrow
    pub fn borrow(&mut self, var_name: &str, line: usize, col: usize) -> Result<(), BorrowError> {
        // Check for existing mutable borrow
        if let Some(existing) = self
            .borrow_ledger
            .iter()
            .find(|b| b.var_name == var_name && b.is_mutable)
        {
            return Err(BorrowError::AlreadyMutablyBorrowed {
                var: var_name.to_string(),
                existing_line: existing.line,
            });
        }

        // OK to add immutable borrow
        self.borrow_ledger.push(BorrowDebt {
            var_name: var_name.to_string(),
            is_mutable: false,
            line,
            column: col,
            scope_depth: self.scope_depth,
        });

        Ok(())
    }

    /// Try to create a mutable borrow (&mut T)
    /// Returns error if there's ANY existing borrow (mutable or immutable)
    pub fn borrow_mut(
        &mut self,
        var_name: &str,
        line: usize,
        col: usize,
    ) -> Result<(), BorrowError> {
        // Check for existing borrows
        if let Some(existing) = self.borrow_ledger.iter().find(|b| b.var_name == var_name) {
            if existing.is_mutable {
                return Err(BorrowError::AlreadyMutablyBorrowed {
                    var: var_name.to_string(),
                    existing_line: existing.line,
                });
            } else {
                return Err(BorrowError::ImmutableBorrowExists {
                    var: var_name.to_string(),
                    existing_line: existing.line,
                });
            }
        }

        // OK to add mutable borrow
        self.borrow_ledger.push(BorrowDebt {
            var_name: var_name.to_string(),
            is_mutable: true,
            line,
            column: col,
            scope_depth: self.scope_depth,
        });

        Ok(())
    }

    /// Release a borrow (reference goes out of scope)
    pub fn release_borrow(&mut self, var_name: &str) {
        if let Some(pos) = self
            .borrow_ledger
            .iter()
            .position(|b| b.var_name == var_name)
        {
            self.borrow_ledger.remove(pos);
        }
    }

    /// Check if a variable is currently borrowed
    pub fn is_borrowed(&self, var_name: &str) -> bool {
        self.borrow_ledger.iter().any(|b| b.var_name == var_name)
    }

    /// Check if a variable is mutably borrowed
    pub fn is_mutably_borrowed(&self, var_name: &str) -> bool {
        self.borrow_ledger
            .iter()
            .any(|b| b.var_name == var_name && b.is_mutable)
    }

    /// Get all active borrows
    pub fn active_borrows(&self) -> &[BorrowDebt] {
        &self.borrow_ledger
    }

    /// Normalize Yoruba text to ASCII for matching
    pub fn normalize(text: &str) -> String {
        text.to_lowercase()
            .replace('ọ', "o")
            .replace('ẹ', "e")
            .replace('ṣ', "s")
            .replace(['à', 'á', 'â'], "a")
            .replace(['è', 'é', 'ê'], "e")
            .replace(['ì', 'í', 'î'], "i")
            .replace(['ò', 'ó', 'ô'], "o")
            .replace(['ù', 'ú', 'û'], "u")
    }

    /// Record opening a resource
    pub fn open_resource(&mut self, domain: &str, method: &str, line: usize, col: usize) {
        let key = format!("{}.{}", Self::normalize(domain), Self::normalize(method));

        if let Some(Some(closer)) = LIFECYCLE_RULES.get(key.as_str()) {
            self.debt_ledger.push(ResourceDebt {
                opener: key,
                required: closer.to_string(),
                line,
                column: col,
            });
        }
    }

    /// Record closing a resource
    pub fn close_resource(&mut self, domain: &str, method: &str) -> bool {
        let key = format!("{}.{}", Self::normalize(domain), Self::normalize(method));

        // Find and remove matching debt
        if let Some(pos) = self.debt_ledger.iter().position(|d| d.required == key) {
            self.debt_ledger.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check for unclosed resources
    pub fn check_balance(&mut self) -> bool {
        // Remove auto-close items
        self.debt_ledger
            .retain(|d| !AUTO_CLOSE.contains(&d.opener.as_str()));

        if self.debt_ledger.is_empty() {
            true
        } else {
            for debt in &self.debt_ledger {
                self.errors.push(format!(
                    "Resource '{}' opened at line {} was never closed (needs '{}')",
                    debt.opener, debt.line, debt.required
                ));
            }
            false
        }
    }

    /// Get unclosed resources
    pub fn unclosed_resources(&self) -> &[ResourceDebt] {
        &self.debt_ledger
    }

    /// Check if balanced
    pub fn is_balanced(&self) -> bool {
        self.debt_ledger
            .iter()
            .filter(|d| !AUTO_CLOSE.contains(&d.opener.as_str()))
            .count()
            == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_lifecycle() {
        let mut engine = IwaEngine::new(true);

        // Open file
        engine.open_resource("Odi", "si", 10, 1);
        assert_eq!(engine.debt_ledger.len(), 1);

        // Close file
        engine.close_resource("Odi", "pa");
        assert!(engine.is_balanced());
    }

    #[test]
    fn test_unclosed_file() {
        let mut engine = IwaEngine::new(true);

        engine.open_resource("Odi", "si", 10, 1);
        assert!(!engine.check_balance());
        assert!(!engine.errors.is_empty());
    }

    #[test]
    fn test_network_lifecycle() {
        let mut engine = IwaEngine::new(true);

        engine.open_resource("Otura", "de", 5, 1);
        engine.close_resource("Otura", "pa");

        assert!(engine.is_balanced());
    }

    // ═══════════════════════════════════════════════════════════════════
    // Borrow Checking Tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn test_immutable_borrow() {
        let mut engine = IwaEngine::new(true);

        // Can create immutable borrow
        assert!(engine.borrow("x", 1, 1).is_ok());
        assert!(engine.is_borrowed("x"));
        assert!(!engine.is_mutably_borrowed("x"));
    }

    #[test]
    fn test_multiple_immutable_borrows() {
        let mut engine = IwaEngine::new(true);

        // Can have multiple immutable borrows of same variable
        assert!(engine.borrow("x", 1, 1).is_ok());
        assert!(engine.borrow("x", 2, 1).is_ok());
        assert_eq!(engine.borrow_ledger.len(), 2);
    }

    #[test]
    fn test_mutable_borrow() {
        let mut engine = IwaEngine::new(true);

        // Can create mutable borrow
        assert!(engine.borrow_mut("x", 1, 1).is_ok());
        assert!(engine.is_borrowed("x"));
        assert!(engine.is_mutably_borrowed("x"));
    }

    #[test]
    fn test_cannot_borrow_mut_while_borrowed() {
        let mut engine = IwaEngine::new(true);

        // Create immutable borrow
        engine.borrow("x", 1, 1).unwrap();

        // Cannot create mutable borrow
        let result = engine.borrow_mut("x", 2, 1);
        assert!(matches!(
            result,
            Err(BorrowError::ImmutableBorrowExists { .. })
        ));
    }

    #[test]
    fn test_cannot_borrow_while_mutably_borrowed() {
        let mut engine = IwaEngine::new(true);

        // Create mutable borrow
        engine.borrow_mut("x", 1, 1).unwrap();

        // Cannot create another borrow (mutable or immutable)
        let result = engine.borrow("x", 2, 1);
        assert!(matches!(
            result,
            Err(BorrowError::AlreadyMutablyBorrowed { .. })
        ));
    }

    #[test]
    fn test_scope_releases_borrows() {
        let mut engine = IwaEngine::new(true);

        engine.enter_scope();
        engine.borrow("x", 1, 1).unwrap();
        assert!(engine.is_borrowed("x"));

        engine.exit_scope();
        assert!(!engine.is_borrowed("x"));
    }

    #[test]
    fn test_release_borrow() {
        let mut engine = IwaEngine::new(true);

        engine.borrow("x", 1, 1).unwrap();
        assert!(engine.is_borrowed("x"));

        engine.release_borrow("x");
        assert!(!engine.is_borrowed("x"));

        // Now can mutably borrow
        assert!(engine.borrow_mut("x", 2, 1).is_ok());
    }
}
