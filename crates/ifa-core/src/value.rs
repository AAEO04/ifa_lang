//! # IfaValue - Dynamic Value Type
//!
//! The universal container for Ifá-Lang's dynamic type system.
//! Supports integers, floats, strings, booleans, lists, maps, objects, and functions.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Not, Rem, Sub};
use std::rc::Rc;

use crate::error::{IfaError, IfaResult};

/// Function signature for lambdas: takes arguments, returns a value
pub type IfaFn = Rc<dyn Fn(Vec<IfaValue>) -> IfaValue>;

/// The universal container (Odù wrapper) for dynamic typing
#[derive(Clone)]
pub enum IfaValue {
    /// Integer (i64)
    Int(i64),
    /// Floating-point (f64)
    Float(f64),
    /// String
    Str(String),
    /// Boolean
    Bool(bool),
    /// List/Array
    List(Vec<IfaValue>),
    /// Map/Dictionary
    Map(HashMap<String, IfaValue>),
    /// Object (heap-allocated, reference-counted)
    Object(Rc<RefCell<HashMap<String, IfaValue>>>),
    /// Lambda/First-class function (closure)
    Fn(IfaFn),
    /// AST-based function (for interpreter)
    AstFn {
        name: String,
        params: Vec<String>,
        body: Vec<crate::ast::Statement>,
    },
    /// Null/None
    Null,
}

// =============================================================================
// DEBUG & DISPLAY
// =============================================================================

impl fmt::Debug for IfaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IfaValue::Int(v) => write!(f, "Int({})", v),
            IfaValue::Float(v) => write!(f, "Float({})", v),
            IfaValue::Str(v) => write!(f, "Str({:?})", v),
            IfaValue::Bool(v) => write!(f, "Bool({})", v),
            IfaValue::List(v) => write!(f, "List({:?})", v),
            IfaValue::Map(v) => write!(f, "Map({:?})", v),
            IfaValue::Object(v) => write!(f, "Object({:?})", v.borrow()),
            IfaValue::Fn(_) => write!(f, "Fn(<lambda>)"),
            IfaValue::AstFn { name, .. } => write!(f, "AstFn({})", name),
            IfaValue::Null => write!(f, "Null"),
        }
    }
}

impl fmt::Display for IfaValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IfaValue::Int(v) => write!(f, "{}", v),
            IfaValue::Float(v) => write!(f, "{}", v),
            IfaValue::Str(v) => write!(f, "{}", v),
            IfaValue::Bool(v) => write!(f, "{}", if *v { "òtítọ́" } else { "èké" }),
            IfaValue::List(v) => {
                let elems: Vec<String> = v.iter().map(|e| e.to_string()).collect();
                write!(f, "[{}]", elems.join(", "))
            }
            IfaValue::Map(m) => {
                let items: Vec<String> =
                    m.iter().map(|(k, v)| format!("\"{}\": {}", k, v)).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            IfaValue::Object(obj_ref) => {
                let obj = obj_ref.borrow();
                write!(f, "<Object with {} fields>", obj.len())
            }
            IfaValue::Fn(_) => write!(f, "<function>"),
            IfaValue::AstFn { name, .. } => write!(f, "<fn {}>", name),
            IfaValue::Null => write!(f, "àìsí"),
        }
    }
}

// =============================================================================
// MATH OPERATIONS (Ọ̀bàrà & Òtúúrúpọ̀n)
// =============================================================================

impl Add for IfaValue {
    type Output = IfaValue;

    fn add(self, other: IfaValue) -> IfaValue {
        match (&self, &other) {
            (IfaValue::Int(a), IfaValue::Int(b)) => a
                .checked_add(*b)
                .map(IfaValue::Int)
                .unwrap_or_else(|| IfaValue::Float(*a as f64 + *b as f64)),
            (IfaValue::Float(a), IfaValue::Float(b)) => IfaValue::Float(a + b),
            (IfaValue::Int(a), IfaValue::Float(b)) => IfaValue::Float(*a as f64 + b),
            (IfaValue::Float(a), IfaValue::Int(b)) => IfaValue::Float(a + *b as f64),
            (IfaValue::Str(a), IfaValue::Str(b)) => IfaValue::Str(format!("{}{}", a, b)),
            (IfaValue::Str(a), b) => IfaValue::Str(format!("{}{}", a, b)),
            (a, IfaValue::Str(b)) => IfaValue::Str(format!("{}{}", a, b)),
            (IfaValue::List(a), IfaValue::List(b)) => {
                let mut result = a.clone();
                result.extend(b.clone());
                IfaValue::List(result)
            }
            _ => IfaValue::Null,
        }
    }
}

impl Sub for IfaValue {
    type Output = IfaValue;

    fn sub(self, other: IfaValue) -> IfaValue {
        match (&self, &other) {
            (IfaValue::Int(a), IfaValue::Int(b)) => a
                .checked_sub(*b)
                .map(IfaValue::Int)
                .unwrap_or_else(|| IfaValue::Float(*a as f64 - *b as f64)),
            (IfaValue::Float(a), IfaValue::Float(b)) => IfaValue::Float(a - b),
            (IfaValue::Int(a), IfaValue::Float(b)) => IfaValue::Float(*a as f64 - b),
            (IfaValue::Float(a), IfaValue::Int(b)) => IfaValue::Float(a - *b as f64),
            _ => IfaValue::Null,
        }
    }
}

impl Mul for IfaValue {
    type Output = IfaValue;

    fn mul(self, other: IfaValue) -> IfaValue {
        match (&self, &other) {
            (IfaValue::Int(a), IfaValue::Int(b)) => a
                .checked_mul(*b)
                .map(IfaValue::Int)
                .unwrap_or_else(|| IfaValue::Float(*a as f64 * *b as f64)),
            (IfaValue::Float(a), IfaValue::Float(b)) => IfaValue::Float(a * b),
            (IfaValue::Int(a), IfaValue::Float(b)) => IfaValue::Float(*a as f64 * b),
            (IfaValue::Float(a), IfaValue::Int(b)) => IfaValue::Float(a * *b as f64),
            (IfaValue::Str(s), IfaValue::Int(n)) if *n >= 0 => IfaValue::Str(s.repeat(*n as usize)),
            (IfaValue::Int(n), IfaValue::Str(s)) if *n >= 0 => IfaValue::Str(s.repeat(*n as usize)),
            _ => IfaValue::Null,
        }
    }
}

impl Div for IfaValue {
    type Output = IfaValue;

    fn div(self, other: IfaValue) -> IfaValue {
        match (&self, &other) {
            (IfaValue::Int(a), IfaValue::Int(b)) => {
                if *b == 0 {
                    return IfaValue::Null;
                }
                IfaValue::Float(*a as f64 / *b as f64)
            }
            (IfaValue::Float(a), IfaValue::Float(b)) => {
                if *b == 0.0 {
                    return IfaValue::Null;
                }
                IfaValue::Float(a / b)
            }
            (IfaValue::Int(a), IfaValue::Float(b)) => {
                if *b == 0.0 {
                    return IfaValue::Null;
                }
                IfaValue::Float(*a as f64 / b)
            }
            (IfaValue::Float(a), IfaValue::Int(b)) => {
                if *b == 0 {
                    return IfaValue::Null;
                }
                IfaValue::Float(a / *b as f64)
            }
            _ => IfaValue::Null,
        }
    }
}

impl Rem for IfaValue {
    type Output = IfaValue;

    fn rem(self, other: IfaValue) -> IfaValue {
        match (&self, &other) {
            (IfaValue::Int(a), IfaValue::Int(b)) => {
                if *b == 0 {
                    return IfaValue::Null;
                }
                IfaValue::Int(a % b)
            }
            (IfaValue::Float(a), IfaValue::Float(b)) => {
                if *b == 0.0 {
                    return IfaValue::Null;
                }
                IfaValue::Float(a % b)
            }
            _ => IfaValue::Null,
        }
    }
}

impl Neg for IfaValue {
    type Output = IfaValue;

    fn neg(self) -> IfaValue {
        match self {
            IfaValue::Int(a) => IfaValue::Int(-a),
            IfaValue::Float(a) => IfaValue::Float(-a),
            _ => IfaValue::Null,
        }
    }
}

impl Not for IfaValue {
    type Output = IfaValue;

    fn not(self) -> IfaValue {
        IfaValue::Bool(!self.is_truthy())
    }
}

// =============================================================================
// COMPARISON
// =============================================================================

impl PartialEq for IfaValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (IfaValue::Int(a), IfaValue::Int(b)) => a == b,
            (IfaValue::Float(a), IfaValue::Float(b)) => a == b,
            (IfaValue::Int(a), IfaValue::Float(b)) => (*a as f64) == *b,
            (IfaValue::Float(a), IfaValue::Int(b)) => *a == (*b as f64),
            (IfaValue::Str(a), IfaValue::Str(b)) => a == b,
            (IfaValue::Bool(a), IfaValue::Bool(b)) => a == b,
            (IfaValue::Null, IfaValue::Null) => true,
            (IfaValue::List(a), IfaValue::List(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for IfaValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (IfaValue::Int(a), IfaValue::Int(b)) => a.partial_cmp(b),
            (IfaValue::Float(a), IfaValue::Float(b)) => a.partial_cmp(b),
            (IfaValue::Int(a), IfaValue::Float(b)) => (*a as f64).partial_cmp(b),
            (IfaValue::Float(a), IfaValue::Int(b)) => a.partial_cmp(&(*b as f64)),
            (IfaValue::Str(a), IfaValue::Str(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

// =============================================================================
// HELPER METHODS
// =============================================================================

impl IfaValue {
    /// Check if value is truthy (for control flow)
    pub fn is_truthy(&self) -> bool {
        match self {
            IfaValue::Bool(b) => *b,
            IfaValue::Int(n) => *n != 0,
            IfaValue::Float(f) => *f != 0.0,
            IfaValue::Str(s) => !s.is_empty(),
            IfaValue::List(l) => !l.is_empty(),
            IfaValue::Map(m) => !m.is_empty(),
            IfaValue::Object(o) => !o.borrow().is_empty(),
            IfaValue::Fn(_) => true,
            IfaValue::AstFn { .. } => true,
            IfaValue::Null => false,
        }
    }

    /// Get length (for strings, lists, maps)
    pub fn len(&self) -> usize {
        match self {
            IfaValue::Str(s) => s.chars().count(),
            IfaValue::List(l) => l.len(),
            IfaValue::Map(m) => m.len(),
            _ => 0,
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get type name as string
    pub fn type_name(&self) -> &'static str {
        match self {
            IfaValue::Int(_) => "Int",
            IfaValue::Float(_) => "Float",
            IfaValue::Str(_) => "Str",
            IfaValue::Bool(_) => "Bool",
            IfaValue::List(_) => "List",
            IfaValue::Map(_) => "Map",
            IfaValue::Object(_) => "Object",
            IfaValue::Fn(_) => "Fn",
            IfaValue::AstFn { .. } => "AstFn",
            IfaValue::Null => "Null",
        }
    }

    /// Index access with Python-style negative indexing
    pub fn get(&self, index: &IfaValue) -> IfaResult<IfaValue> {
        match (self, index) {
            (IfaValue::List(l), IfaValue::Int(i)) => {
                let len = l.len() as i64;
                let idx = if *i < 0 { len + *i } else { *i };
                if idx < 0 || idx >= len {
                    Err(IfaError::IndexOutOfBounds {
                        index: *i,
                        length: l.len(),
                    })
                } else {
                    Ok(l[idx as usize].clone())
                }
            }
            (IfaValue::Str(s), IfaValue::Int(i)) => {
                let chars: Vec<char> = s.chars().collect();
                let len = chars.len() as i64;
                let idx = if *i < 0 { len + *i } else { *i };
                if idx < 0 || idx >= len {
                    Err(IfaError::IndexOutOfBounds {
                        index: *i,
                        length: chars.len(),
                    })
                } else {
                    Ok(IfaValue::Str(chars[idx as usize].to_string()))
                }
            }
            (IfaValue::Map(m), IfaValue::Str(k)) => m
                .get(k)
                .cloned()
                .ok_or_else(|| IfaError::KeyNotFound(k.clone())),
            _ => Err(IfaError::TypeError {
                expected: "indexable type".to_string(),
                got: self.type_name().to_string(),
            }),
        }
    }

    /// Set value at index
    pub fn set(&mut self, index: &IfaValue, value: IfaValue) -> IfaResult<()> {
        match (self, index) {
            (IfaValue::List(l), IfaValue::Int(i)) => {
                let idx = *i as usize;
                if idx < l.len() {
                    l[idx] = value;
                    Ok(())
                } else {
                    Err(IfaError::IndexOutOfBounds {
                        index: *i,
                        length: l.len(),
                    })
                }
            }
            (IfaValue::Map(m), IfaValue::Str(k)) => {
                m.insert(k.clone(), value);
                Ok(())
            }
            _ => Err(IfaError::TypeError {
                expected: "mutable indexable type".to_string(),
                got: "unknown".to_string(),
            }),
        }
    }

    /// Push to list
    pub fn push(&mut self, value: IfaValue) -> IfaResult<()> {
        if let IfaValue::List(l) = self {
            l.push(value);
            Ok(())
        } else {
            Err(IfaError::TypeError {
                expected: "List".to_string(),
                got: self.type_name().to_string(),
            })
        }
    }

    /// Pop from list
    pub fn pop(&mut self) -> IfaResult<IfaValue> {
        if let IfaValue::List(l) = self {
            l.pop().ok_or(IfaError::IndexOutOfBounds {
                index: -1,
                length: 0,
            })
        } else {
            Err(IfaError::TypeError {
                expected: "List".to_string(),
                got: self.type_name().to_string(),
            })
        }
    }

    /// Slice (for strings and lists)
    pub fn slice(&self, start: i64, end: i64) -> IfaResult<IfaValue> {
        match self {
            IfaValue::Str(s) => {
                let chars: Vec<char> = s.chars().collect();
                let len = chars.len() as i64;
                let start_idx = start.max(0) as usize;
                let end_idx = end.min(len) as usize;
                if start_idx >= end_idx {
                    Ok(IfaValue::Str(String::new()))
                } else {
                    Ok(IfaValue::Str(chars[start_idx..end_idx].iter().collect()))
                }
            }
            IfaValue::List(l) => {
                let len = l.len() as i64;
                let start_idx = start.max(0) as usize;
                let end_idx = end.min(len) as usize;
                if start_idx >= end_idx {
                    Ok(IfaValue::List(vec![]))
                } else {
                    Ok(IfaValue::List(l[start_idx..end_idx].to_vec()))
                }
            }
            _ => Err(IfaError::TypeError {
                expected: "sliceable type".to_string(),
                got: self.type_name().to_string(),
            }),
        }
    }
}

// =============================================================================
// CONVERSION HELPERS
// =============================================================================

impl From<i64> for IfaValue {
    fn from(v: i64) -> Self {
        IfaValue::Int(v)
    }
}

impl From<f64> for IfaValue {
    fn from(v: f64) -> Self {
        IfaValue::Float(v)
    }
}

impl From<String> for IfaValue {
    fn from(v: String) -> Self {
        IfaValue::Str(v)
    }
}

impl From<&str> for IfaValue {
    fn from(v: &str) -> Self {
        IfaValue::Str(v.to_string())
    }
}

impl From<bool> for IfaValue {
    fn from(v: bool) -> Self {
        IfaValue::Bool(v)
    }
}

impl<T: Into<IfaValue>> From<Vec<T>> for IfaValue {
    fn from(v: Vec<T>) -> Self {
        IfaValue::List(v.into_iter().map(Into::into).collect())
    }
}

impl From<()> for IfaValue {
    fn from(_: ()) -> Self {
        IfaValue::Null
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic() {
        let a = IfaValue::Int(10);
        let b = IfaValue::Int(5);
        assert_eq!(a.clone() + b.clone(), IfaValue::Int(15));
        assert_eq!(a.clone() - b.clone(), IfaValue::Int(5));
        assert_eq!(a.clone() * b.clone(), IfaValue::Int(50));
    }

    #[test]
    fn test_checked_overflow() {
        let max = IfaValue::Int(i64::MAX);
        let one = IfaValue::Int(1);
        // Should promote to float instead of panic
        if let IfaValue::Float(_) = max + one {
            // Expected behavior
        } else {
            panic!("Should have promoted to Float on overflow");
        }
    }

    #[test]
    fn test_negative_indexing() {
        let list = IfaValue::List(vec![IfaValue::Int(1), IfaValue::Int(2), IfaValue::Int(3)]);
        assert_eq!(list.get(&IfaValue::Int(-1)).unwrap(), IfaValue::Int(3));
        assert_eq!(list.get(&IfaValue::Int(-2)).unwrap(), IfaValue::Int(2));
    }

    #[test]
    fn test_truthy() {
        assert!(IfaValue::Bool(true).is_truthy());
        assert!(!IfaValue::Bool(false).is_truthy());
        assert!(IfaValue::Int(1).is_truthy());
        assert!(!IfaValue::Int(0).is_truthy());
        assert!(!IfaValue::Null.is_truthy());
    }
}
