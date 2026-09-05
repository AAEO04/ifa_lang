//! # Ọ̀bàrà Domain (1000)
//!
//! The Expander - Mathematical Operations (Add/Mul)
//!
//! Handles addition, multiplication, power, and positive math operations.

use crate::impl_odu_domain;
use ifa_vm::error::{IfaError, IfaResult};

/// Ọ̀bàrà - The Expander (Math Add/Mul)
pub struct Obara;

impl_odu_domain!(Obara, "Ọ̀bàrà", "1000", "The Expander - Math Add/Mul");

impl Obara {
    /// Add two numbers (fikun)
    pub fn fikun(&self, a: f64, b: f64) -> f64 {
        a + b
    }

    /// Exact decimal addition
    pub fn fikun_odidi(&self, a: i64, b: i64, _scale: u32) -> IfaResult<i64> {
        a.checked_add(b)
            .ok_or_else(|| IfaError::Overflow(format!("{} + {} overflows", a, b)))
    }

    /// Exact decimal addition (with rounding signature)
    pub fn fikun_rounded(
        &self,
        a: i64,
        b: i64,
        scale: u32,
        _mode: crate::odu::oturupon::RoundingMode,
    ) -> IfaResult<i64> {
        self.fikun_odidi(a, b, scale)
    }

    /// Multiply (ìsọdìpúpọ̀)
    pub fn isodipupo(&self, a: f64, b: f64) -> f64 {
        a * b
    }

    /// Exact decimal multiplication
    pub fn isodipupo_odidi(&self, a: i64, b: i64, scale: u32) -> IfaResult<i64> {
        let prod = (a as i128) * (b as i128);
        let divisor = 10_i128.checked_pow(scale).ok_or_else(|| {
            IfaError::Overflow("Decimal multiplication scale overflows i128".into())
        })?;
        crate::odu::oturupon::round_div(prod, divisor, crate::odu::oturupon::RoundingMode::HalfEven)
    }

    /// Power (agbára)
    pub fn agbara(&self, base: f64, exp: f64) -> f64 {
        base.powf(exp)
    }

    /// Square root (gbòǹgbò)
    pub fn gbongbo(&self, x: f64) -> f64 {
        x.sqrt()
    }

    /// Absolute value
    pub fn abs(&self, x: f64) -> f64 {
        x.abs()
    }

    /// Sum of list (àpapọ̀)
    pub fn apapo(&self, items: &[f64]) -> f64 {
        items.iter().sum()
    }

    /// Floor (ilé)
    pub fn ile(&self, x: f64) -> f64 {
        x.floor()
    }

    /// Ceiling (orúlé)
    pub fn orule(&self, x: f64) -> f64 {
        x.ceil()
    }

    /// Round (yíká)
    pub fn yika(&self, x: f64, decimals: i32) -> f64 {
        let factor = 10_f64.powi(decimals);
        (x * factor).round() / factor
    }

    /// Modulo (ìyọkù)
    pub fn iyoku(&self, a: f64, b: f64) -> f64 {
        a % b
    }

    // Trigonometry
    pub fn sin(&self, x: f64) -> f64 {
        x.sin()
    }
    pub fn cos(&self, x: f64) -> f64 {
        x.cos()
    }
    pub fn tan(&self, x: f64) -> f64 {
        x.tan()
    }
    pub fn asin(&self, x: f64) -> f64 {
        x.asin()
    }
    pub fn acos(&self, x: f64) -> f64 {
        x.acos()
    }
    pub fn atan(&self, x: f64) -> f64 {
        x.atan()
    }

    // Logarithms
    pub fn log(&self, x: f64) -> f64 {
        x.ln()
    }
    pub fn log10(&self, x: f64) -> f64 {
        x.log10()
    }
    pub fn exp(&self, x: f64) -> f64 {
        x.exp()
    }

    // Statistics
    pub fn aropin(&self, items: &[f64]) -> f64 {
        if items.is_empty() {
            return 0.0;
        }
        items.iter().sum::<f64>() / items.len() as f64
    }

    pub fn nla_julo(&self, items: &[f64]) -> f64 {
        items.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }

    pub fn kere_julo(&self, items: &[f64]) -> f64 {
        items.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    // Constants
    pub fn pi(&self) -> f64 {
        std::f64::consts::PI
    }
    pub fn e(&self) -> f64 {
        std::f64::consts::E
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_math() {
        let obara = Obara;
        assert_eq!(obara.fikun(5.0, 3.0), 8.0);
        assert_eq!(obara.isodipupo(4.0, 3.0), 12.0);
        assert_eq!(obara.agbara(2.0, 3.0), 8.0);
    }

    #[test]
    fn test_decimal_math() {
        let obara = Obara;

        // addition
        let res1 = obara.fikun_odidi(100, 200, 2).unwrap();
        assert_eq!(res1, 300);

        // overflow
        assert!(obara.fikun_odidi(i64::MAX, 1, 2).is_err());

        // multiplication: 1.50 * 2.00 scaled to 2 decimal places = 150 * 200 = 30000 -> /100 = 300 -> 3.00
        let res2 = obara.isodipupo_odidi(150, 200, 2).unwrap();
        assert_eq!(res2, 300);

        // multiplication banker's rounding: 1.25 * 2 = 2.50. Scale 1: 12 * 20 = 240 / 10 = 24 -> 2.4. Wait.
        // let's do: 15 * 15 = 225. Scale 1: 225 / 10 = 22.5. banker's rounding rounds 22.5 to 22.
        let res3 = obara.isodipupo_odidi(15, 15, 1).unwrap();
        assert_eq!(res3, 22);
    }

    #[test]
    fn test_statistics() {
        let obara = Obara;
        let items = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(obara.apapo(&items), 15.0);
        assert_eq!(obara.aropin(&items), 3.0);
        assert_eq!(obara.nla_julo(&items), 5.0);
        assert_eq!(obara.kere_julo(&items), 1.0);
    }
}
