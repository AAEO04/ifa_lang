//! # Ọ̀kànràn Domain (0001)
//! 
//! The Asserter - Error Handling and Assertions
//! 
//! Custom error types with eyre for rich reports and Yoruba proverbs.

use ifa_core::error::{IfaError, IfaResult};
use crate::impl_odu_domain;

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
    
    /// Assert not null
    pub fn ko_si(&self, value: &ifa_core::IfaValue, name: &str) -> IfaResult<()> {
        if matches!(value, ifa_core::IfaValue::Null) {
            Err(IfaError::AssertionFailed(format!("{} is null", name)))
        } else {
            Ok(())
        }
    }
    
    /// Panic with message (kú!)
    pub fn ku(&self, message: &str) -> ! {
        panic!("[Ọ̀kànràn] {}", message)
    }
    
    /// Unreachable code marker
    pub fn ko_le_de(&self) -> ! {
        panic!("[Ọ̀kànràn] Unreachable code executed!")
    }
    
    /// TODO marker (to be implemented)
    pub fn ko_ti_se(&self, feature: &str) -> ! {
        panic!("[Ọ̀kànràn] Not yet implemented: {}", feature)
    }
    
    /// Get proverb for error type
    pub fn owe(&self, error: &IfaError) -> &'static str {
        error.proverb()
    }
    
    /// Try operation, return default on error
    pub fn gbiyanju<T, F: FnOnce() -> IfaResult<T>>(&self, f: F, default: T) -> T {
        f().unwrap_or(default)
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
    fn test_gbiyanju() {
        let okanran = Okanran;
        let result = okanran.gbiyanju(|| Err(IfaError::Custom("test".into())), 42);
        assert_eq!(result, 42);
    }
}
