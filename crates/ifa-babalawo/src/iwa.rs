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
    m.insert("odi.ko", Some("odi.pa")); // Write -> Close

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

/// The Ìwà Engine - ensures resource lifecycle balance
#[derive(Debug)]
pub struct IwaEngine {
    pub strict_mode: bool,
    pub debt_ledger: Vec<ResourceDebt>,
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
            errors: Vec::new(),
            warnings: Vec::new(),
        }
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
}
