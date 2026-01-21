//! Comprehensive tests for ifa-babalawo wisdom system
//!
//! Tests the Babalawo static analysis components:
//! - Wisdom lookup
//! - Taboo enforcement (Èèwọ̀)
//! - Resource lifecycle (Ìwà)

use ifa_babalawo::*;

mod wisdom_lookup_tests {
    use super::*;

    #[test]
    fn test_odu_wisdom_exists() {
        // Verify we can look up wisdom for major Odù
        assert!(ODU_WISDOM.contains_key("OGBE"));
        assert!(ODU_WISDOM.contains_key("OYEKU"));
        assert!(ODU_WISDOM.contains_key("IWORI"));
    }

    #[test]
    fn test_error_mapping() {
        // Verify common errors map to Odù
        assert_eq!(ERROR_TO_ODU.get("UNINITIALIZED"), Some(&"OGBE"));
        assert_eq!(ERROR_TO_ODU.get("DIVISION_BY_ZERO"), Some(&"OTURUPON"));

        // Verify mapped Odù exists in wisdom DB
        let odu = ERROR_TO_ODU.get("UNINITIALIZED").unwrap();
        let wisdom = ODU_WISDOM.get(odu).unwrap();
        assert!(!wisdom.proverbs.is_empty());
        assert!(!wisdom.advice.is_empty());
    }
}

mod taboo_tests {
    use super::*;

    #[test]
    fn test_taboo_enforcer_creation() {
        let enforcer = TabooEnforcer::new();
        assert!(enforcer.is_clean());
    }

    #[test]
    fn test_wildcard_taboo() {
        let mut enforcer = TabooEnforcer::new();
        // Block all network calls (Otura)
        enforcer.add_wildcard_taboo("otura");

        let allowed = enforcer.check_call("ose", "otura", 1, 1);
        assert!(!allowed);
        assert!(!enforcer.is_clean());

        // Output should mention violation
        let violations = enforcer.get_violations();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].callee, "otura");
    }

    #[test]
    fn test_specific_taboo() {
        let mut enforcer = TabooEnforcer::new();
        // UI (ose) cannot call Database (odi)
        enforcer.add_taboo("ose", "UI", "odi", "", false);

        enforcer.set_context("UI");
        let allowed = enforcer.check_call("ose", "odi", 1, 1);
        assert!(!allowed);

        // But Backend can (different context)
        enforcer.set_context("Backend");
        // We need a fresh check or just inspect the return
        let allowed_backend = enforcer.check_call("backend", "odi", 2, 1);
        assert!(allowed_backend);
    }
}

mod iwa_tests {
    use super::*;

    #[test]
    fn test_resource_lifecycle() {
        let mut engine = IwaEngine::new(true);

        // Open a file (Odi.si)
        engine.open_resource("Odi", "si", 10, 1);
        assert!(!engine.is_balanced());

        // Check unstructured debt
        let debt = engine.unclosed_resources();
        assert_eq!(debt.len(), 1);
        assert_eq!(debt[0].required, "odi.pa");

        // Close the file (Odi.pa)
        let closed = engine.close_resource("Odi", "pa");
        assert!(closed);
        assert!(engine.is_balanced());
    }

    #[test]
    fn test_unbalanced_resource() {
        let mut engine = IwaEngine::new(true);
        engine.open_resource("Otura", "so", 1, 1); // Connect

        assert!(!engine.check_balance());
        // Verify it complains (implicit via check_balance return false)
    }

    #[test]
    fn test_auto_close_resources() {
        let mut engine = IwaEngine::new(true);
        // "ogbe.bi" is in AUTO_CLOSE list
        engine.open_resource("ogbe", "bi", 1, 1);

        // Should be considered balanced because it auto-closes
        assert!(engine.is_balanced());
    }
}
