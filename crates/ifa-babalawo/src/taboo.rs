//! # Èèwọ̀ Enforcer - Taboo / Architectural Constraints
//!
//! Validates architectural constraints to prevent forbidden dependencies.
//! "Whoever touches a taboo will see the consequences"
//! Ported from legacy/src/validator.py TabooEnforcer class

/// A taboo rule - forbidden dependency pattern
#[derive(Debug, Clone)]
pub struct Taboo {
    pub source_domain: String,
    pub source_context: String,
    pub target_domain: String,
    pub target_context: String,
    pub is_wildcard: bool,
}

/// A taboo violation
#[derive(Debug, Clone)]
pub struct TabooViolation {
    pub taboo: Taboo,
    pub caller: String,
    pub callee: String,
    pub context: String,
    pub line: usize,
    pub column: usize,
}

/// The Èèwọ̀ Enforcer - validates architectural constraints
#[derive(Debug)]
pub struct TabooEnforcer {
    pub taboos: Vec<Taboo>,
    pub violations: Vec<TabooViolation>,
    pub current_context: String,
}

impl Default for TabooEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

impl TabooEnforcer {
    pub fn new() -> Self {
        Self {
            taboos: Vec::new(),
            violations: Vec::new(),
            current_context: String::new(),
        }
    }

    /// Register a taboo rule
    ///
    /// Example: UI cannot call DB directly
    /// ```ignore
    /// enforcer.add_taboo("ose", "UI", "odi", "DB", false);
    /// ```
    pub fn add_taboo(
        &mut self,
        source_domain: &str,
        source_context: &str,
        target_domain: &str,
        target_context: &str,
        is_wildcard: bool,
    ) {
        self.taboos.push(Taboo {
            source_domain: source_domain.to_lowercase(),
            source_context: source_context.to_string(),
            target_domain: target_domain.to_lowercase(),
            target_context: target_context.to_string(),
            is_wildcard,
        });
    }

    /// Add a wildcard taboo - blocks ALL calls to a domain
    pub fn add_wildcard_taboo(&mut self, domain: &str) {
        self.add_taboo(domain, "", "", "", true);
    }

    /// Set current code context (e.g., "UI", "Backend")
    pub fn set_context(&mut self, context: &str) {
        self.current_context = context.to_string();
    }

    /// Check if a call violates any taboo
    /// Returns true if allowed, false if forbidden
    pub fn check_call(
        &mut self,
        caller_domain: &str,
        callee_domain: &str,
        line: usize,
        column: usize,
    ) -> bool {
        let caller = caller_domain.to_lowercase();
        let callee = callee_domain.to_lowercase();

        for taboo in &self.taboos {
            // Wildcard taboo: Block all calls from this domain
            if taboo.is_wildcard {
                if callee == taboo.source_domain {
                    self.violations.push(TabooViolation {
                        taboo: taboo.clone(),
                        caller: caller.clone(),
                        callee: callee.clone(),
                        context: self.current_context.clone(),
                        line,
                        column,
                    });
                    return false;
                }
            } else {
                // Specific taboo: Block source -> target
                let source_match = caller == taboo.source_domain || taboo.source_domain.is_empty();
                let context_match =
                    self.current_context == taboo.source_context || taboo.source_context.is_empty();
                let target_match = callee == taboo.target_domain;

                if source_match && context_match && target_match {
                    self.violations.push(TabooViolation {
                        taboo: taboo.clone(),
                        caller: caller.clone(),
                        callee: callee.clone(),
                        context: self.current_context.clone(),
                        line,
                        column,
                    });
                    return false;
                }
            }
        }

        true
    }

    /// Check if no taboos were violated
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }

    /// Get all violations
    pub fn get_violations(&self) -> &[TabooViolation] {
        &self.violations
    }

    /// Format violations for output
    pub fn format_violations(&self) -> String {
        if self.violations.is_empty() {
            return String::new();
        }

        let mut output = String::new();
        output.push_str("ÈÈWỌ̀ VIOLATIONS (Taboo Broken):\n\n");

        for v in &self.violations {
            if v.taboo.is_wildcard {
                output.push_str(&format!(
                    "  Line {}: Called forbidden domain '{}'\n",
                    v.line, v.callee
                ));
                output.push_str(&format!(
                    "    -> Taboo: {}.* is not allowed\n\n",
                    v.taboo.source_domain
                ));
            } else {
                output.push_str(&format!(
                    "  Line {}: '{}' called '{}'\n",
                    v.line, v.caller, v.callee
                ));
                if !v.context.is_empty() {
                    output.push_str(&format!(
                        "    -> Context '{}' cannot access '{}'\n\n",
                        v.context, v.callee
                    ));
                } else {
                    output.push_str("    -> This dependency is forbidden\n\n");
                }
            }
        }

        output.push_str("Proverb: \"Ẹni tó bá fọwọ́ kan èèwọ̀, yóò rí àṣèdá\"\n");
        output.push_str("(Whoever touches a taboo will see the consequences)\n");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_taboo() {
        let mut enforcer = TabooEnforcer::new();
        enforcer.add_wildcard_taboo("otura"); // Block all network

        assert!(!enforcer.check_call("ose", "otura", 10, 1));
        assert!(!enforcer.is_clean());
    }

    #[test]
    fn test_specific_taboo() {
        let mut enforcer = TabooEnforcer::new();
        enforcer.add_taboo("ose", "UI", "odi", "", false); // UI can't call file

        enforcer.set_context("UI");
        assert!(!enforcer.check_call("ose", "odi", 10, 1));

        enforcer.set_context("Backend");
        // Reset violations for clean test
        let mut enforcer2 = TabooEnforcer::new();
        enforcer2.add_taboo("ose", "UI", "odi", "", false);
        enforcer2.set_context("Backend");
        assert!(enforcer2.check_call("ose", "odi", 10, 1)); // Backend CAN call file
    }

    #[test]
    fn test_no_taboo() {
        let mut enforcer = TabooEnforcer::new();
        assert!(enforcer.check_call("irosu", "obara", 10, 1));
        assert!(enforcer.is_clean());
    }
}
