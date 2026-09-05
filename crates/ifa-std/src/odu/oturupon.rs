//! # Òtúúrúpọ̀n Domain (0010)
//!
//! The Reducer - Mathematical Operations (Sub/Div)
//!
//! Handles subtraction, division, and reductive operations with checked arithmetic.

use crate::impl_odu_domain;
use ifa_vm::error::{IfaError, IfaResult};

/// Rounding mode for division operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundingMode {
    HalfEven, // Banker's rounding
    HalfUp,   // School rounding
    HalfDown,
    Down,     // Truncate towards zero
    Up,       // Away from zero
    Ceil,     // Towards +infinity
    Floor,    // Towards -infinity
    Truncate, // Alias for Down
    Ceiling,  // Alias for Ceil
}

pub fn round_div(dividend: i128, divisor: i128, mode: RoundingMode) -> IfaResult<i64> {
    if divisor == 0 {
        return Err(IfaError::DivisionByZero(
            "Division by zero in decimal math".into(),
        ));
    }
    let q = dividend / divisor;
    let r = dividend % divisor;
    if r == 0 {
        return to_i64(q);
    }
    let sign = (dividend > 0) == (divisor > 0);
    let r_abs_x2 = r.abs() * 2;
    let div_abs = divisor.abs();

    let round_away = if r_abs_x2 < div_abs {
        match mode {
            RoundingMode::Up => true,
            RoundingMode::Ceil => sign,
            RoundingMode::Floor => !sign,
            _ => false,
        }
    } else if r_abs_x2 > div_abs {
        match mode {
            RoundingMode::Down | RoundingMode::Truncate => false,
            RoundingMode::Floor => !sign,
            RoundingMode::Ceil => sign,
            _ => true,
        }
    } else {
        match mode {
            RoundingMode::HalfEven => q % 2 != 0,
            RoundingMode::HalfUp => true,
            RoundingMode::HalfDown => false,
            RoundingMode::Up => true,
            RoundingMode::Down | RoundingMode::Truncate => false,
            RoundingMode::Ceil => sign,
            RoundingMode::Floor => !sign,
            RoundingMode::Ceiling => sign,
        }
    };

    let result = if round_away {
        if sign { q + 1 } else { q - 1 }
    } else {
        q
    };

    to_i64(result)
}

fn to_i64(val: i128) -> IfaResult<i64> {
    i64::try_from(val).map_err(|_| IfaError::Overflow("Decimal math overflows i64".into()))
}

/// Òtúúrúpọ̀n - The Reducer (Math Sub/Div)
pub struct Oturupon;

impl_odu_domain!(Oturupon, "Òtúúrúpọ̀n", "0010", "The Reducer - Math Sub/Div");

impl Oturupon {
    /// Checked subtraction (dín)
    pub fn din(&self, a: i64, b: i64) -> IfaResult<i64> {
        a.checked_sub(b)
            .ok_or_else(|| IfaError::Overflow(format!("{} - {} overflows", a, b)))
    }

    /// Exact decimal subtraction
    pub fn din_odidi(&self, a: i64, b: i64, _scale: u32) -> IfaResult<i64> {
        a.checked_sub(b)
            .ok_or_else(|| IfaError::Overflow(format!("{} - {} overflows", a, b)))
    }

    /// Checked division (pín)
    pub fn pin(&self, a: i64, b: i64) -> IfaResult<f64> {
        if b == 0 {
            return Err(IfaError::DivisionByZero(format!("{} / 0", a)));
        }
        Ok(a as f64 / b as f64)
    }

    /// Exact scaled decimal division
    pub fn pin_odidi_scaled(
        &self,
        a: i64,
        b: i64,
        scale: u32,
        mode: RoundingMode,
    ) -> IfaResult<i64> {
        let divisor = b as i128;
        if divisor == 0 {
            return Err(IfaError::DivisionByZero(
                "Division by zero in decimal math".into(),
            ));
        }
        let scaling_factor = 10_i128.checked_pow(scale).ok_or_else(|| {
            IfaError::Overflow("Decimal division scaling factor overflows i128".into())
        })?;
        let dividend = (a as i128)
            .checked_mul(scaling_factor)
            .ok_or_else(|| IfaError::Overflow("Decimal dividend scaling overflows i128".into()))?;
        round_div(dividend, divisor, mode)
    }

    /// Integer division (pín_odidi)
    pub fn pin_odidi(&self, a: i64, b: i64) -> IfaResult<i64> {
        if b == 0 {
            return Err(IfaError::DivisionByZero(format!("{} / 0", a)));
        }
        a.checked_div(b)
            .ok_or_else(|| IfaError::Overflow(format!("{} / {} overflows", a, b)))
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
            RoundingMode::Truncate | RoundingMode::Down => result.trunc(),
            RoundingMode::Floor => result.floor(),
            RoundingMode::Ceiling | RoundingMode::Ceil => result.ceil(),
            RoundingMode::HalfEven => {
                let rounded = result.round();
                if (result - rounded).abs() == 0.5 {
                    if rounded as i64 % 2 != 0 {
                        if result > 0.0 {
                            rounded - 1.0
                        } else {
                            rounded + 1.0
                        }
                    } else {
                        rounded
                    }
                } else {
                    rounded
                }
            }
            RoundingMode::HalfUp => {
                if result > 0.0 {
                    (result + 0.5).floor()
                } else {
                    (result - 0.5).ceil()
                }
            }
            RoundingMode::HalfDown => {
                if result > 0.0 {
                    (result - 0.5).ceil()
                } else {
                    (result + 0.5).floor()
                }
            }
            RoundingMode::Up => {
                if result > 0.0 {
                    result.ceil()
                } else {
                    result.floor()
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
        x.checked_neg()
            .ok_or_else(|| IfaError::Overflow(format!("-{} overflows", x)))
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
        max.checked_sub(value)
            .ok_or_else(|| IfaError::Overflow(format!("{} - {} overflows", max, value)))
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

    #[test]
    fn test_decimal_math() {
        let oturupon = Oturupon;

        // division: 1 / 3 scaled to 2 decimal places: 100 / 3 = 33.33 -> 33
        let res1 = oturupon
            .pin_odidi_scaled(1, 3, 2, RoundingMode::HalfEven)
            .unwrap();
        assert_eq!(res1, 33);

        // division banker's rounding: 25 / 10 = 2.5. Scale 0. Even is 2, odd is 3. 2 is even -> 2
        let res2 = oturupon
            .pin_odidi_scaled(25, 10, 0, RoundingMode::HalfEven)
            .unwrap();
        assert_eq!(res2, 2);

        // division half up: 25 / 10 = 2.5 -> 3
        let res3 = oturupon
            .pin_odidi_scaled(25, 10, 0, RoundingMode::HalfUp)
            .unwrap();
        assert_eq!(res3, 3);

        // division floor: -25 / 10 = -2.5 -> -3
        let res4 = oturupon
            .pin_odidi_scaled(-25, 10, 0, RoundingMode::Floor)
            .unwrap();
        assert_eq!(res4, -3);

        // subtraction
        let res5 = oturupon.din_odidi(100, 30, 2).unwrap();
        assert_eq!(res5, 70);

        // overflow checked
        assert!(oturupon.din_odidi(i64::MIN, 1, 2).is_err());
    }
}
