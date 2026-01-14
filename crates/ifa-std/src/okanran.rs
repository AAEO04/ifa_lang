//! # Ọ̀kànràn Domain (0001)
//!
//! The Asserter - Error Handling and Assertions
//!
//! Custom error types with eyre for rich reports and Yoruba proverbs.

use crate::impl_odu_domain;
use ifa_core::error::{IfaError, IfaResult};

/// Ọ̀kànràn - The Asserter (Errors/Assertions)
pub struct Okanran;

impl_odu_domain!(Okanran, "Ọ̀kànràn", "0001", "The Asserter - Errors");

impl Okanran {
    /// Assert condition is true (bẹ́ẹ̀ ni)
    pub fn beeni(&self, condition: bool, message: &str) -> IfaResult<()> {
        if condition {
            Ok(())
        } else {
            Err(IfaError::AssertionFailed(message.to_string()))
        }
    }

    /// Assert equal (dọ́gba)
    pub fn dogba(&self, a: &ifa_core::IfaValue, b: &ifa_core::IfaValue) -> IfaResult<()> {
        if a == b {
            Ok(())
        } else {
            Err(IfaError::AssertionFailed(format!(
                "Not equal: {:?} != {:?}",
                a, b
            )))
        }
    }

    /// Assert not equal (yàtọ̀)
    pub fn yato(&self, a: &ifa_core::IfaValue, b: &ifa_core::IfaValue) -> IfaResult<()> {
        if a != b {
            Ok(())
        } else {
            Err(IfaError::AssertionFailed(format!(
                "Equal: {:?} == {:?}",
                a, b
            )))
        }
    }

    /// Assert false (bẹ́ẹ̀ kọ́)
    pub fn beko(&self, condition: bool, message: &str) -> IfaResult<()> {
        if !condition {
            Ok(())
        } else {
            Err(IfaError::AssertionFailed(message.to_string()))
        }
    }

    /// Assert not null
    pub fn ko_si(&self, value: &ifa_core::IfaValue, name: &str) -> IfaResult<()> {
        if matches!(value, ifa_core::IfaValue::Null) {
            Err(IfaError::AssertionFailed(format!("{} is null", name)))
        } else {
            Ok(())
        }
    }

    /// Panic with message (kú!)
    /// 
    /// **DEPRECATED**: Panicking is an anti-pattern. Use `ku_bi` for Result-based errors.
    #[deprecated(since = "1.2.1", note = "Use ku_bi() instead which returns Result")]
    pub fn ku(&self, message: &str) -> ! {
        panic!("[Ọ̀kànràn] {}", message)
    }

    /// Fatal error (kú bí) - Returns error instead of panicking
    pub fn ku_bi(&self, message: &str) -> IfaResult<()> {
        Err(IfaError::Custom(format!("[FATAL] {}", message)))
    }

    /// Unreachable code marker
    /// 
    /// **DEPRECATED**: Use `ko_le_de_bi` instead.
    #[deprecated(since = "1.2.1", note = "Use ko_le_de_bi() instead which returns Result")]
    pub fn ko_le_de(&self) -> ! {
        panic!("[Ọ̀kànràn] Unreachable code executed!")
    }

    /// Unreachable code (returns error)
    pub fn ko_le_de_bi(&self) -> IfaResult<()> {
        Err(IfaError::Custom("Unreachable code executed".to_string()))
    }

    /// TODO marker (to be implemented)
    /// 
    /// **DEPRECATED**: Use `ko_ti_se_bi` instead.
    #[deprecated(since = "1.2.1", note = "Use ko_ti_se_bi() instead which returns Result")]
    pub fn ko_ti_se(&self, feature: &str) -> ! {
        panic!("Ẹ̀yà '{}' kò tíì ṣé (Feature '{}' is not yet implemented)", feature, feature);
    }

    /// Not implemented (returns error)
    pub fn ko_ti_se_bi(&self, feature: &str) -> IfaResult<()> {
        Err(IfaError::NotImplemented(feature.to_string()))
    }

    /// Get proverb for error type
    pub fn owe(&self, error: &IfaError) -> &'static str {
        error.proverb()
    }

    /// Try operation, return default on error
    pub fn gbiyanju<T, F: FnOnce() -> IfaResult<T>>(&self, f: F, default: T) -> T {
        f().unwrap_or(default)
    }

    // =========================================================================
    // MOCKING (Mole)
    // =========================================================================

    /// Create a mock (mòle)
    /// Returns an Object { "fn": <closure>, "calls": [] }
    pub fn mole(&self, return_value: ifa_core::IfaValue) -> ifa_core::IfaValue {
        use std::cell::RefCell;
        use std::collections::HashMap;
        use std::rc::Rc;
        use ifa_core::IfaValue;

        // Shared state for the mock
        let state = Rc::new(RefCell::new(HashMap::new()));
        state.borrow_mut().insert("calls".to_string(), IfaValue::List(vec![]));
        
        // Capture state for the closure
        let state_clone = state.clone();
        let ret_val_clone = return_value.clone();

        let closure = move |args: Vec<IfaValue>| {
            // Record call
            let mut map = state_clone.borrow_mut();
            if let Some(IfaValue::List(calls)) = map.get_mut("calls") {
                calls.push(IfaValue::List(args));
            }
            ret_val_clone.clone()
        };

        // Add closure to the object
        state.borrow_mut().insert("fn".to_string(), IfaValue::Fn(Rc::new(closure)));

        IfaValue::Object(state)
    }

    /// Inspect mock calls (wo_mole)
    pub fn wo_mole(&self, mock: &ifa_core::IfaValue) -> IfaResult<ifa_core::IfaValue> {
        if let ifa_core::IfaValue::Object(state) = mock {
            let map = state.borrow();
            if let Some(calls) = map.get("calls") {
                Ok(calls.clone())
            } else {
                Err(IfaError::Custom("Invalid mock object".to_string()))
            }
        } else {
            Err(IfaError::TypeError { expected: "Mock Object".into(), got: mock.type_name().into() })
        }
    }

    // =========================================================================
    // TEST RUNNER (Sure)
    // =========================================================================

    /// Run a suite of tests (ṣúre)
    /// Takes a Map of "test_name" -> fn
    pub fn sure(&self, tests: &ifa_core::IfaValue) -> IfaResult<bool> {
        if let ifa_core::IfaValue::Map(map) = tests {
            let mut passed = 0;
            let mut failed = 0;

            println!("\n=== Ọ̀kànràn Test Runner ===");
            
            for (name, test_fn) in map {
                print!("Running {} ... ", name);
                
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    if let ifa_core::IfaValue::Fn(f) = test_fn {
                        f(vec![]);
                        true
                    } else if let ifa_core::IfaValue::AstFn { .. } = test_fn {
                         // AST functions need interpreter context, simpler runner can't run them directly easily
                         // without full VM. Assuming Closure-based tests for now.
                         println!("SKIPPED (AST Fn)");
                         false
                    } else {
                        false
                    }
                }));

                match result {
                    Ok(true) => {
                        println!("PASSED");
                        passed += 1;
                    }
                    Ok(false) => {
                        println!("FAILED (Type)");
                        failed += 1;
                    }
                    Err(_) => {
                        println!("FAILED");
                        failed += 1;
                    }
                }
            }
            
            println!("---------------------------------------------------");
            println!("Tests: {}, Passed: {}, Failed: {}", passed + failed, passed, failed);
            
            Ok(failed == 0)
        } else {
             Err(IfaError::TypeError { expected: "Map of tests".into(), got: tests.type_name().into() })
        }
    }

    /// Debug print value
    pub fn wo(&self, label: &str, value: &impl std::fmt::Debug) {
        eprintln!("[Ọ̀kànràn DEBUG] {}: {:?}", label, value);
    }
}

/// Result extension trait for Yoruba-style error handling
pub trait IfaResultExt<T> {
    /// Unwrap or panic with Yoruba message
    fn tabi_ku(self, msg: &str) -> T;

    /// Map error to custom message
    fn pelu_iroyin(self, msg: &str) -> IfaResult<T>;
}

impl<T> IfaResultExt<T> for IfaResult<T> {
    fn tabi_ku(self, msg: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => panic!("[Ọ̀kànràn] {}: {}", msg, e),
        }
    }

    fn pelu_iroyin(self, msg: &str) -> IfaResult<T> {
        self.map_err(|e| IfaError::Custom(format!("{}: {}", msg, e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_pass() {
        let okanran = Okanran;
        assert!(okanran.beeni(true, "should pass").is_ok());
    }

    #[test]
    fn test_assert_fail() {
        let okanran = Okanran;
        assert!(okanran.beeni(false, "should fail").is_err());
    }

    #[test]
    fn test_assertions() {
        let okanran = Okanran;
        assert!(okanran.dogba(&ifa_core::IfaValue::Int(1), &ifa_core::IfaValue::Int(1)).is_ok());
        assert!(okanran.dogba(&ifa_core::IfaValue::Int(1), &ifa_core::IfaValue::Int(2)).is_err());
        assert!(okanran.yato(&ifa_core::IfaValue::Int(1), &ifa_core::IfaValue::Int(2)).is_ok());
        assert!(okanran.beko(false, "ok").is_ok());
    }

    #[test]
    fn test_mocking() {
        use ifa_core::IfaValue;
        let okanran = Okanran;
        
        // Create mole (mock)
        let mock = okanran.mole(IfaValue::Str("returned".to_string()));
        
        // Get function from mock
        let func = if let IfaValue::Object(obj) = &mock {
             let map = obj.borrow();
             if let Some(IfaValue::Fn(f)) = map.get("fn") {
                 f.clone()
             } else {
                 panic!("No fn in mock");
             }
        } else {
            panic!("Mock not object");
        };

        // Call the mock function
        let result = func(vec![IfaValue::Int(1), IfaValue::Int(2)]);
        assert_eq!(result, IfaValue::Str("returned".to_string()));

        // Check logs
        let logs = okanran.wo_mole(&mock).unwrap();
        if let IfaValue::List(calls) = logs {
            assert_eq!(calls.len(), 1);
            if let IfaValue::List(args) = &calls[0] {
                assert_eq!(args[0], IfaValue::Int(1));
                assert_eq!(args[1], IfaValue::Int(2));
            } else {
                panic!("Call args not list");
            }
        } else {
            panic!("Logs not list");
        }
    }
}
