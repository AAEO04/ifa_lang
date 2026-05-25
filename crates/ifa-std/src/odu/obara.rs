//! # Ọ̀bàrà Domain (1000)
//!
//! The Expander - Mathematical Operations (Add/Mul)
//!
//! Handles addition, multiplication, power, and positive math operations.

use crate::impl_odu_domain;

/// Ọ̀bàrà - The Expander (Math Add/Mul)
pub struct Obara;

impl_odu_domain!(Obara, "Ọ̀bàrà", "1000", "The Expander - Math Add/Mul");

impl Obara {
    /// Add two numbers (fikun)
    pub fn fikun(&self, a: f64, b: f64) -> f64 {
        a + b
    }

    /// Multiply (ìsọdìpúpọ̀)
    pub fn isodipupo(&self, a: f64, b: f64) -> f64 {
        a * b
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
    fn test_statistics() {
        let obara = Obara;
        let items = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(obara.apapo(&items), 15.0);
        assert_eq!(obara.aropin(&items), 3.0);
        assert_eq!(obara.nla_julo(&items), 5.0);
        assert_eq!(obara.kere_julo(&items), 1.0);
    }
}
