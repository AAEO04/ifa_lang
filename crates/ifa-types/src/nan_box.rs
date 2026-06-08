//! NaN-boxed primitive representation for Ifa values.
//!
//! This module intentionally exposes only a safe API. Bit layout knowledge
//! stays here so the VM can migrate opcode handlers without open-coding masks.

const QUIET_NAN_BITS: u64 = 0x7ff8_0000_0000_0000;
const TAG_MASK: u64 = 0x0007_0000_0000_0000;
const PAYLOAD_MASK: u64 = 0x0000_ffff_ffff_ffff;

const TAG_NULL: u64 = 0x0001_0000_0000_0000;
const TAG_BOOL_FALSE: u64 = 0x0002_0000_0000_0000;
const TAG_BOOL_TRUE: u64 = 0x0003_0000_0000_0000;
const TAG_INT: u64 = 0x0004_0000_0000_0000;
const TAG_FLOAT_NAN: u64 = 0x0005_0000_0000_0000;

const INT_PAYLOAD_MASK: u64 = 0x0000_ffff_ffff_ffff;
const INT_SIGN_BIT: u64 = 1 << 47;
const MAX_BOXED_INT: i64 = (1i64 << 47) - 1;
const MIN_BOXED_INT: i64 = -(1i64 << 47);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxedPrimitive {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NanBoxError {
    IntOutOfRange(i64),
    NotABoxedPrimitive(u64),
    /// Integer division by zero. VM translates this to IfaError::DivisionByZero.
    DivisionByZero,
    /// Operand types are not compatible for this operation (e.g. Null op Null).
    TypeMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NanBox(u64);
impl NanBox {
    #[inline(always)]
    pub const fn from_null() -> Self {
        Self(QUIET_NAN_BITS | TAG_NULL)
    }

    #[inline(always)]
    pub const fn from_bool(value: bool) -> Self {
        Self(QUIET_NAN_BITS | if value { TAG_BOOL_TRUE } else { TAG_BOOL_FALSE })
    }

    #[inline(always)]
    pub fn from_int(value: i64) -> Result<Self, NanBoxError> {
        if !(MIN_BOXED_INT..=MAX_BOXED_INT).contains(&value) {
            return Err(NanBoxError::IntOutOfRange(value));
        }
        let payload = (value as i128 & INT_PAYLOAD_MASK as i128) as u64;
        Ok(Self(QUIET_NAN_BITS | TAG_INT | payload))
    }

    #[inline(always)]
    pub fn from_float(value: f64) -> Self {
        if value.is_nan() {
            return Self(QUIET_NAN_BITS | TAG_FLOAT_NAN);
        }
        Self(value.to_bits())
    }

    #[inline(always)]
    pub fn from_primitive(value: BoxedPrimitive) -> Result<Self, NanBoxError> {
        match value {
            BoxedPrimitive::Null => Ok(Self::from_null()),
            BoxedPrimitive::Bool(b) => Ok(Self::from_bool(b)),
            BoxedPrimitive::Int(i) => Self::from_int(i),
            BoxedPrimitive::Float(f) => Ok(Self::from_float(f)),
        }
    }

    #[inline(always)]
    pub const fn raw_bits(self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub const fn is_boxed_float(self) -> bool {
        (self.0 & QUIET_NAN_BITS) != QUIET_NAN_BITS
    }

    #[inline(always)]
    pub const fn is_null(self) -> bool {
        !self.is_boxed_float() && (self.0 & TAG_MASK) == TAG_NULL
    }

    #[inline(always)]
    pub const fn is_bool(self) -> bool {
        !self.is_boxed_float()
            && ((self.0 & TAG_MASK) == TAG_BOOL_FALSE || (self.0 & TAG_MASK) == TAG_BOOL_TRUE)
    }

    #[inline(always)]
    pub const fn is_int(self) -> bool {
        !self.is_boxed_float() && (self.0 & TAG_MASK) == TAG_INT
    }

    #[inline(always)]
    pub const fn is_float(self) -> bool {
        self.is_boxed_float() || (!self.is_boxed_float() && (self.0 & TAG_MASK) == TAG_FLOAT_NAN)
    }

    #[inline(always)]
    pub fn as_bool(self) -> Option<bool> {
        match self.0 & TAG_MASK {
            TAG_BOOL_FALSE if !self.is_boxed_float() => Some(false),
            TAG_BOOL_TRUE if !self.is_boxed_float() => Some(true),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn as_int(self) -> Option<i64> {
        if !self.is_int() {
            return None;
        }

        let payload = self.0 & PAYLOAD_MASK;
        let signed = if (payload & INT_SIGN_BIT) != 0 {
            payload | !INT_PAYLOAD_MASK
        } else {
            payload
        };
        Some(signed as i64)
    }

    #[inline(always)]
    pub fn as_float(self) -> Option<f64> {
        if self.is_boxed_float() {
            Some(f64::from_bits(self.0))
        } else if (self.0 & TAG_MASK) == TAG_FLOAT_NAN {
            Some(f64::NAN)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn to_primitive(self) -> Result<BoxedPrimitive, NanBoxError> {
        if self.is_null() {
            return Ok(BoxedPrimitive::Null);
        }
        if let Some(b) = self.as_bool() {
            return Ok(BoxedPrimitive::Bool(b));
        }
        if let Some(i) = self.as_int() {
            return Ok(BoxedPrimitive::Int(i));
        }
        if let Some(f) = self.as_float() {
            return Ok(BoxedPrimitive::Float(f));
        }
        Err(NanBoxError::NotABoxedPrimitive(self.0))
    }

    /// Check truthiness of the boxed value.
    /// Follows Ifa-Lang semantics: Null/False/0/0.0/NaN are falsy.
    #[inline(always)]
    pub fn is_truthy(self) -> bool {
        if self.is_null() {
            return false;
        }
        if let Some(b) = self.as_bool() {
            return b;
        }
        if let Some(i) = self.as_int() {
            return i != 0;
        }
        if let Some(f) = self.as_float() {
            return f != 0.0 && !f.is_nan();
        }
        true
    }

    // --- Arithmetic ---

    #[allow(clippy::should_implement_trait)]
    pub fn add(self, other: Self) -> Option<Self> {
        let pa = self.to_primitive().ok()?;
        let pb = other.to_primitive().ok()?;
        match (pa, pb) {
            (BoxedPrimitive::Int(a), BoxedPrimitive::Int(b)) => {
                let sum = a.checked_add(b)?;
                Some(NanBox::from_int(sum).unwrap_or_else(|_| NanBox::from_float(sum as f64)))
            }
            (BoxedPrimitive::Float(a), BoxedPrimitive::Float(b)) => Some(NanBox::from_float(a + b)),
            (BoxedPrimitive::Int(a), BoxedPrimitive::Float(b)) => {
                Some(NanBox::from_float(a as f64 + b))
            }
            (BoxedPrimitive::Float(a), BoxedPrimitive::Int(b)) => {
                Some(NanBox::from_float(a + b as f64))
            }
            _ => None,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, other: Self) -> Option<Self> {
        let pa = self.to_primitive().ok()?;
        let pb = other.to_primitive().ok()?;
        match (pa, pb) {
            (BoxedPrimitive::Int(a), BoxedPrimitive::Int(b)) => {
                let res = a.checked_sub(b)?;
                Some(NanBox::from_int(res).unwrap_or_else(|_| NanBox::from_float(res as f64)))
            }
            (BoxedPrimitive::Float(a), BoxedPrimitive::Float(b)) => Some(NanBox::from_float(a - b)),
            (BoxedPrimitive::Int(a), BoxedPrimitive::Float(b)) => {
                Some(NanBox::from_float(a as f64 - b))
            }
            (BoxedPrimitive::Float(a), BoxedPrimitive::Int(b)) => {
                Some(NanBox::from_float(a - b as f64))
            }
            _ => None,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, other: Self) -> Option<Self> {
        let pa = self.to_primitive().ok()?;
        let pb = other.to_primitive().ok()?;
        match (pa, pb) {
            (BoxedPrimitive::Int(a), BoxedPrimitive::Int(b)) => {
                let res = a.checked_mul(b)?;
                Some(NanBox::from_int(res).unwrap_or_else(|_| NanBox::from_float(res as f64)))
            }
            (BoxedPrimitive::Float(a), BoxedPrimitive::Float(b)) => Some(NanBox::from_float(a * b)),
            (BoxedPrimitive::Int(a), BoxedPrimitive::Float(b)) => {
                Some(NanBox::from_float(a as f64 * b))
            }
            (BoxedPrimitive::Float(a), BoxedPrimitive::Int(b)) => {
                Some(NanBox::from_float(a * b as f64))
            }
            _ => None,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn div(self, other: Self) -> Result<NanBox, NanBoxError> {
        let pa = self.to_primitive().map_err(|_| NanBoxError::TypeMismatch)?;
        let pb = other
            .to_primitive()
            .map_err(|_| NanBoxError::TypeMismatch)?;
        match (pa, pb) {
            (BoxedPrimitive::Int(a), BoxedPrimitive::Int(b)) => {
                if b == 0 {
                    return Err(NanBoxError::DivisionByZero);
                }
                // Compute once. i64::MIN / -1 overflows i64 — promote to Float in that case.
                let result = a.checked_div(b).ok_or(NanBoxError::IntOutOfRange(a))?;
                NanBox::from_int(result).or_else(|_| Ok(NanBox::from_float(result as f64)))
            }
            (BoxedPrimitive::Float(a), BoxedPrimitive::Float(b)) => {
                if b == 0.0 {
                    return Err(NanBoxError::DivisionByZero);
                }
                Ok(NanBox::from_float(a / b))
            }
            (BoxedPrimitive::Int(a), BoxedPrimitive::Float(b)) => {
                if b == 0.0 {
                    return Err(NanBoxError::DivisionByZero);
                }
                Ok(NanBox::from_float(a as f64 / b))
            }
            (BoxedPrimitive::Float(a), BoxedPrimitive::Int(b)) => {
                if b == 0 {
                    return Err(NanBoxError::DivisionByZero);
                }
                Ok(NanBox::from_float(a / b as f64))
            }
            _ => Err(NanBoxError::TypeMismatch),
        }
    }

    // --- Comparison ---

    pub fn lt(self, other: Self) -> Option<bool> {
        let pa = self.to_primitive().ok()?;
        let pb = other.to_primitive().ok()?;
        match (pa, pb) {
            (BoxedPrimitive::Int(a), BoxedPrimitive::Int(b)) => Some(a < b),
            (BoxedPrimitive::Float(a), BoxedPrimitive::Float(b)) => Some(a < b),
            (BoxedPrimitive::Int(a), BoxedPrimitive::Float(b)) => Some((a as f64) < b),
            (BoxedPrimitive::Float(a), BoxedPrimitive::Int(b)) => Some(a < (b as f64)),
            _ => None,
        }
    }

    pub fn le(self, other: Self) -> Option<bool> {
        let pa = self.to_primitive().ok()?;
        let pb = other.to_primitive().ok()?;
        match (pa, pb) {
            (BoxedPrimitive::Int(a), BoxedPrimitive::Int(b)) => Some(a <= b),
            (BoxedPrimitive::Float(a), BoxedPrimitive::Float(b)) => Some(a <= b),
            (BoxedPrimitive::Int(a), BoxedPrimitive::Float(b)) => Some((a as f64) <= b),
            (BoxedPrimitive::Float(a), BoxedPrimitive::Int(b)) => Some(a <= (b as f64)),
            _ => None,
        }
    }

    pub fn gt(self, other: Self) -> Option<bool> {
        let pa = self.to_primitive().ok()?;
        let pb = other.to_primitive().ok()?;
        match (pa, pb) {
            (BoxedPrimitive::Int(a), BoxedPrimitive::Int(b)) => Some(a > b),
            (BoxedPrimitive::Float(a), BoxedPrimitive::Float(b)) => Some(a > b),
            (BoxedPrimitive::Int(a), BoxedPrimitive::Float(b)) => Some((a as f64) > b),
            (BoxedPrimitive::Float(a), BoxedPrimitive::Int(b)) => Some(a > (b as f64)),
            _ => None,
        }
    }

    pub fn ge(self, other: Self) -> Option<bool> {
        let pa = self.to_primitive().ok()?;
        let pb = other.to_primitive().ok()?;
        match (pa, pb) {
            (BoxedPrimitive::Int(a), BoxedPrimitive::Int(b)) => Some(a >= b),
            (BoxedPrimitive::Float(a), BoxedPrimitive::Float(b)) => Some(a >= b),
            (BoxedPrimitive::Int(a), BoxedPrimitive::Float(b)) => Some((a as f64) >= b),
            (BoxedPrimitive::Float(a), BoxedPrimitive::Int(b)) => Some(a >= (b as f64)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_null_bool_int_and_float() {
        let values = [
            BoxedPrimitive::Null,
            BoxedPrimitive::Bool(false),
            BoxedPrimitive::Bool(true),
            BoxedPrimitive::Int(0),
            BoxedPrimitive::Int(42),
            BoxedPrimitive::Int(-42),
            BoxedPrimitive::Float(3.5),
        ];

        for value in values {
            let boxed = NanBox::from_primitive(value).expect("boxing should succeed");
            assert_eq!(
                boxed.to_primitive().expect("unboxing should succeed"),
                value
            );
        }
    }

    #[test]
    fn rejects_integers_outside_47_bit_payload() {
        assert_eq!(
            NanBox::from_int(MAX_BOXED_INT + 1),
            Err(NanBoxError::IntOutOfRange(MAX_BOXED_INT + 1))
        );
        assert_eq!(
            NanBox::from_int(MIN_BOXED_INT - 1),
            Err(NanBoxError::IntOutOfRange(MIN_BOXED_INT - 1))
        );
    }

    #[test]
    fn preserves_float_nan_payload_as_float() {
        let boxed = NanBox::from_float(f64::NAN);
        assert!(boxed.is_float());
        assert!(boxed.as_float().expect("float expected").is_nan());
    }
}
