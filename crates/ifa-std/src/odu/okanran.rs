//! # Ọ̀kànràn Domain (0001)
//!
//! The Asserter - Error Handling and Assertions
//!
//! Custom error types with eyre for rich reports and Yoruba proverbs.

use crate::impl_odu_domain;
use ifa_vm::error::{IfaError, IfaResult};

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
    pub fn dogba(&self, a: &ifa_vm::IfaValue, b: &ifa_vm::IfaValue) -> IfaResult<()> {
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
    pub fn yato(&self, a: &ifa_vm::IfaValue, b: &ifa_vm::IfaValue) -> IfaResult<()> {
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
    pub fn ko_si(&self, value: &ifa_vm::IfaValue, name: &str) -> IfaResult<()> {
        if matches!(value, ifa_vm::IfaValue::Null) {
            Err(IfaError::AssertionFailed(format!("{} is null", name)))
        } else {
            Ok(())
        }
    }

    /// Panic with message (kú!)
    ///
    /// **DEPRECATED**: Panicking is an anti-pattern. Use `ku_bi` for Result-based errors.
    #[deprecated(since = "1.2.1", note = "Use ku_bi() instead which returns Result")]
    pub fn ku(&self, message: &str) -> IfaResult<()> {
        Err(IfaError::Custom(format!("[Ọ̀kànràn] {}", message)))
    }

    /// Fatal error (kú bí) - Returns error instead of panicking
    /// **DEPRECATED**: Use `ta` instead.
    #[deprecated(since = "1.3.0", note = "Use ta() which uses UserError")]
    pub fn ku_bi(&self, message: &str) -> IfaResult<()> {
        Err(IfaError::Custom(format!("[FATAL] {}", message)))
    }

    /// Throw exception (Tà) - Raises a recoverable UserError.
    /// The value passed to `ta` is preserved as a structured `IfaValue` in
    /// the catch handler — no lossy string coercion.
    pub fn ta(&self, message: &str) -> IfaResult<()> {
        Err(IfaError::UserError(Box::new(ifa_vm::IfaValue::Str(
            ifa_types::CompactString::new(message),
        ))))
    }

    /// Unreachable code marker
    ///
    /// **DEPRECATED**: Use `ko_le_de_bi` instead.
    #[deprecated(
        since = "1.2.1",
        note = "Use ko_le_de_bi() instead which returns Result"
    )]
    pub fn ko_le_de(&self) -> IfaResult<()> {
        Err(IfaError::Custom(
            "[Ọ̀kànràn] Unreachable code executed!".into(),
        ))
    }

    /// Unreachable code (returns error)
    pub fn ko_le_de_bi(&self) -> IfaResult<()> {
        Err(IfaError::Custom("Unreachable code executed".to_string()))
    }

    /// Panic with a "not yet implemented" message in Yoruba and English.
    ///
    /// **DEPRECATED**: Use `ko_ti_se_bi` instead which returns `Result` instead of panicking.
    #[deprecated(
        since = "1.2.1",
        note = "Use ko_ti_se_bi() instead which returns Result"
    )]
    pub fn ko_ti_se(&self, feature: &str) -> IfaResult<()> {
        Err(IfaError::NotImplemented(format!(
            "Ẹ̀yà '{}' kò tíì ṣé (Feature '{}' is not yet implemented)",
            feature, feature
        )))
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

    /* LEGACY MOCK DISABLED
    pub fn mole(&self, _return_value: ifa_vm::IfaValue) -> ifa_vm::IfaValue {
        ifa_vm::IfaValue::Null
    }
    */

    /* LEGACY INSPECT DISABLED
    pub fn wo_mole(&self, mock: &ifa_vm::IfaValue) -> IfaResult<ifa_vm::IfaValue> {
        Err(IfaError::NotImplemented("Mocking disabled".into()))
    }
    */

    // =========================================================================
    // TEST RUNNER (Sure)
    // =========================================================================

    /* LEGACY RUNNER DISABLED
    pub fn sure(&self, tests: &ifa_vm::IfaValue) -> IfaResult<bool> {
        // ... disabled ...
        Ok(true)
    }
    */

    /// Debug print value
    pub fn wo(&self, _label: &str, _value: &impl std::fmt::Debug) {
        // Debug output suppressed in library context
    }
}

/// Result extension trait for Yoruba-style error handling
pub trait IfaResultExt<T> {
    /// Unwrap or panic with Yoruba message
    fn tabi_ku(self, msg: &str) -> IfaResult<T>;

    /// Map error to custom message
    fn pelu_iroyin(self, msg: &str) -> IfaResult<T>;
}

impl<T> IfaResultExt<T> for IfaResult<T> {
    fn tabi_ku(self, msg: &str) -> IfaResult<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(IfaError::Custom(format!("[Ọ̀kànràn] {}: {}", msg, e))),
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
        assert!(
            okanran
                .dogba(&ifa_vm::IfaValue::Int(1), &ifa_vm::IfaValue::Int(1))
                .is_ok()
        );
        assert!(
            okanran
                .dogba(&ifa_vm::IfaValue::Int(1), &ifa_vm::IfaValue::Int(2))
                .is_err()
        );
        assert!(
            okanran
                .yato(&ifa_vm::IfaValue::Int(1), &ifa_vm::IfaValue::Int(2))
                .is_ok()
        );
        assert!(okanran.beko(false, "ok").is_ok());
    }

    /* LEGACY MOCK TEST DISABLED
    #[test]
    fn test_mocking() {
        // ... disabled ...
    }
    */
}
