//! # Òtúúrúpọ̀n Domain (0010)
//! 
//! The Reducer - Mathematical Operations (Sub/Div)
//! 
//! Handles subtraction, division, and reductive operations with checked arithmetic.

use ifa_core::error::{IfaError, IfaResult};
use crate::impl_odu_domain;

/// Rounding mode for division operations
#[derive(Debug, Clone, Copy)]
pub enum RoundingMode {
    /// Round toward zero (truncate)
    Truncate,
    /// Round toward negative infinity (floor)
    Floor,
    /// Round toward positive infinity (ceiling)
    Ceiling,
    /// Round to nearest, ties to even (banker's rounding)
    HalfEven,
}

/// Òtúúrúpọ̀n - The Reducer (Math Sub/Div)
pub struct Oturupon;

impl_odu_domain!(Oturupon, "Òtúúrúpọ̀n", "0010", "The Reducer - Math Sub/Div");

impl Oturupon {
    /// Checked subtraction (dín)
    pub fn din(&self, a: i64, b: i64) -> IfaResult<i64> {
        a.checked_sub(b).ok_or_else(|| {
            IfaError::Overflow(format!("{} - {} overflows", a, b))
        })
    }
    
    /// Checked division (pín)
    pub fn pin(&self, a: i64, b: i64) -> IfaResult<f64> {
        if b == 0 {
            return Err(IfaError::DivisionByZero(format!("{} / 0", a)));
        }
        Ok(a as f64 / b as f64)
    }
    
    /// Integer division (pín_odidi)
    pub fn pin_odidi(&self, a: i64, b: i64) -> IfaResult<i64> {
        if b == 0 {
            return Err(IfaError::DivisionByZero(format!("{} / 0", a)));
        }
        a.checked_div(b).ok_or_else(|| {
            IfaError::Overflow(format!("{} / {} overflows", a, b))
        })
    }
    
    /// Float subtraction
    pub fn din_f(&self, a: f64, b: f64) -> f64 {
        a - b
    }
    
    /// Float division with rounding mode
    pub fn pin_f(&self, a: f64, b: f64, mode: RoundingMode) -> IfaResult<f64> {
        if b == 0.0 {
            return Err(IfaError::DivisionByZero(format!("{} / 0.0", a)));
        }
        let result = a / b;
        Ok(match mode {
            RoundingMode::Truncate => result.trunc(),
            RoundingMode::Floor => result.floor(),
            RoundingMode::Ceiling => result.ceil(),
            RoundingMode::HalfEven => {
                let rounded = result.round();
                // Banker's rounding: if exactly halfway, round to even
                if (result - rounded).abs() == 0.5 {
                    if rounded as i64 % 2 != 0 {
                        if result > 0.0 { rounded - 1.0 } else { rounded + 1.0 }
                    } else {
                        rounded
                    }
                } else {
                    rounded
                }
            }
        })
    }
    
    /// Modulo with remainder (kù)
    pub fn ku(&self, a: i64, b: i64) -> IfaResult<i64> {
        if b == 0 {
            return Err(IfaError::DivisionByZero(format!("{} % 0", a)));
        }
        Ok(a % b)
    }
    
    /// Euclidean modulo (always positive result)
    pub fn ku_euclidean(&self, a: i64, b: i64) -> IfaResult<i64> {
        if b == 0 {
            return Err(IfaError::DivisionByZero(format!("{} % 0", a)));
        }
        Ok(a.rem_euclid(b))
    }
    
    /// Negate (dákẹ́)
    pub fn dake(&self, x: i64) -> IfaResult<i64> {
        x.checked_neg().ok_or_else(|| {
            IfaError::Overflow(format!("-{} overflows", x))
        })
    }
    
    /// Reciprocal (1/x)
    pub fn idakeji(&self, x: f64) -> IfaResult<f64> {
        if x == 0.0 {
            return Err(IfaError::DivisionByZero("1 / 0".to_string()));
        }
        Ok(1.0 / x)
    }
    
    /// Difference from max (remaining)
    pub fn iyoku(&self, value: i64, max: i64) -> IfaResult<i64> {
        max.checked_sub(value).ok_or_else(|| {
            IfaError::Overflow(format!("{} - {} overflows", max, value))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_checked_division() {
        let oturupon = Oturupon;
        assert!(oturupon.pin(10, 3).is_ok());
        assert!(oturupon.pin(10, 0).is_err());
    }
    
    #[test]
    fn test_checked_subtraction() {
        let oturupon = Oturupon;
        assert_eq!(oturupon.din(10, 3).unwrap(), 7);
        // Test overflow
        assert!(oturupon.din(i64::MIN, 1).is_err());
    }
    
    #[test]
    fn test_rounding_modes() {
        let oturupon = Oturupon;
        let result = oturupon.pin_f(7.0, 2.0, RoundingMode::Floor).unwrap();
        assert_eq!(result, 3.0);
    }
}
