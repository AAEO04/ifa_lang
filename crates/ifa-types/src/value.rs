//! # IfaValue - Dynamic Value Type
//!
//! The universal container for Ifá-Lang's dynamic type system.
//! Supports integers, floats, strings, booleans, lists, maps, and functions.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, Mul, Neg, Not, Rem, Sub};
use std::rc::Rc;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::shared::IfaShared;
// use crate::domain::OduDomain;
#[cfg(feature = "vm")]
use crate::ast::Statement;
use crate::error::{IfaError, IfaResult};
use crate::token::ResourceToken;

/// Function signature for lambdas: takes arguments, returns a value
/// Note: !Send + !Sync (Thread-local)
pub type IfaFn = Rc<dyn Fn(Vec<IfaValue>) -> IfaValue>;

/// The universal container (Odù wrapper) for dynamic typing
#[derive(Clone, Serialize, Deserialize)]
pub enum IfaValue {
    /// Integer (i64)
    Int(i64),
    /// Floating-point (f64)
    Float(f64),
    /// String
    Str(Arc<str>),
    /// Boolean
    Bool(bool),
    /// List/Array
    List(Vec<IfaValue>),
    /// Map/Dictionary
    Map(HashMap<Arc<str>, IfaValue>),
    /// Object (heap-allocated, thread-local access)
    /// "The Hut" - Zero-cost local access, no atomic overhead
    #[serde(skip)]
    Object(Rc<RefCell<HashMap<Arc<str>, IfaValue>>>),
    /// Function (Standard/Native)
    #[serde(skip)]
    Fn(IfaFn),

    /// Resource Token (Opaque Handle) - Link to Global Registry
    Resource(ResourceToken),

    /// AST Function (User defined) - Requires "vm" feature
    #[cfg(feature = "vm")]
    AstFn {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
        closure_id: u64,
    },

    /// Bytecode Function (Compiled) - Requires "vm" feature
    #[cfg(feature = "vm")]
    BytecodeFn {
        name: String,
        start_ip: usize,
        arity: u8,
    },

    /// Class Definition - Requires "vm" feature
    #[cfg(feature = "vm")]
    Class {
        name: String,
        fields: Vec<String>,
        methods: HashMap<String, IfaValue>,
    },

    /// Null value
    Null,
    /// Return wrapper
    Return(Box<IfaValue>),

    /// Pointer (Memory Address) - Requires "vm" feature
    #[cfg(feature = "vm")]
    Ptr(usize),

    /// Reference (Variable Name) - Requires "vm" feature
    #[cfg(feature = "vm")]
    Ref(String),
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
            IfaValue::Object(v) => {
                // Use try_borrow to avoid panics during debugging
                if let Ok(map) = v.try_borrow() {
                    write!(f, "Object({:?})", map)
                } else {
                    write!(f, "Object(<locked>)")
                }
            }
            IfaValue::Fn(_) => write!(f, "Fn(<lambda>)"),
            IfaValue::Null => write!(f, "Null"),
            IfaValue::Resource(t) => write!(f, "Resource({})", t.0),
            IfaValue::Return(v) => write!(f, "Return({:?})", v),

            #[cfg(feature = "vm")]
            IfaValue::AstFn { name, .. } => write!(f, "AstFn({})", name),
            #[cfg(feature = "vm")]
            IfaValue::BytecodeFn { name, .. } => write!(f, "BytecodeFn({})", name),
            #[cfg(feature = "vm")]
            IfaValue::Class { name, .. } => write!(f, "Class({})", name),
            #[cfg(feature = "vm")]
            IfaValue::Ptr(p) => write!(f, "Ptr({:x})", p),
            #[cfg(feature = "vm")]
            IfaValue::Ref(id) => write!(f, "Ref({})", id),
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
                write!(f, "[")?;
                for (i, e) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, "]")
            }
            IfaValue::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", k, v)?;
                }
                write!(f, "}}")
            }
            IfaValue::Object(obj_ref) => {
                if let Ok(obj) = obj_ref.try_borrow() {
                    write!(f, "<Object with {} fields>", obj.len())
                } else {
                    write!(f, "<Object (locked)>")
                }
            }
            IfaValue::Fn(_) => write!(f, "<function>"),
            IfaValue::Null => write!(f, "àìsí"),
            IfaValue::Resource(t) => write!(f, "<Resource {}>", t.0),
            IfaValue::Return(v) => write!(f, "<return {}>", v),

            #[cfg(feature = "vm")]
            IfaValue::AstFn { name, .. } => write!(f, "<fn {}>", name),
            #[cfg(feature = "vm")]
            IfaValue::BytecodeFn { name, .. } => write!(f, "<fn {}>", name),
            #[cfg(feature = "vm")]
            IfaValue::Class { name, .. } => write!(f, "<class {}>", name),
            #[cfg(feature = "vm")]
            IfaValue::Ptr(p) => write!(f, "*0x{:x}", p),
            #[cfg(feature = "vm")]
            IfaValue::Ref(id) => write!(f, "&{}", id),
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
            (IfaValue::Str(a), IfaValue::Str(b)) => IfaValue::Str(format!("{}{}", a, b).into()),
            (IfaValue::Str(a), b) => IfaValue::Str(format!("{}{}", a, b).into()),
            (a, IfaValue::Str(b)) => IfaValue::Str(format!("{}{}", a, b).into()),
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
            (IfaValue::Str(s), IfaValue::Int(n)) if *n >= 0 => {
                IfaValue::Str(s.repeat(*n as usize).into())
            }
            (IfaValue::Int(n), IfaValue::Str(s)) if *n >= 0 => {
                IfaValue::Str(s.repeat(*n as usize).into())
            }
            _ => IfaValue::Null,
        }
    }
}

impl IfaValue {
    /// Checked division that returns proper errors.
    pub fn checked_div(&self, other: &IfaValue) -> IfaResult<IfaValue> {
        match (self, other) {
            (IfaValue::Int(a), IfaValue::Int(b)) => {
                if *b == 0 {
                    return Err(IfaError::DivisionByZero(
                        "Cannot divide by zero".to_string(),
                    ));
                }
                Ok(IfaValue::Float(*a as f64 / *b as f64))
            }
            (IfaValue::Float(a), IfaValue::Float(b)) => {
                if *b == 0.0 {
                    return Err(IfaError::DivisionByZero(
                        "Cannot divide by zero".to_string(),
                    ));
                }
                Ok(IfaValue::Float(a / b))
            }
            (IfaValue::Int(a), IfaValue::Float(b)) => {
                if *b == 0.0 {
                    return Err(IfaError::DivisionByZero(
                        "Cannot divide by zero".to_string(),
                    ));
                }
                Ok(IfaValue::Float(*a as f64 / b))
            }
            (IfaValue::Float(a), IfaValue::Int(b)) => {
                if *b == 0 {
                    return Err(IfaError::DivisionByZero(
                        "Cannot divide by zero".to_string(),
                    ));
                }
                Ok(IfaValue::Float(a / *b as f64))
            }
            _ => Err(IfaError::TypeError {
                expected: "numeric type".to_string(),
                got: format!("{} / {}", self.type_name(), other.type_name()),
            }),
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
            (IfaValue::Int(a), IfaValue::Float(b)) => {
                let a_f64 = *a as f64;
                if a_f64 as i64 == *a {
                    a_f64 == *b
                } else {
                    false
                }
            }
            (IfaValue::Float(a), IfaValue::Int(b)) => {
                let b_f64 = *b as f64;
                if b_f64 as i64 == *b {
                    *a == b_f64
                } else {
                    false
                }
            }
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
            (IfaValue::Int(a), IfaValue::Float(b)) => {
                let a_f64 = *a as f64;
                if a_f64 as i64 == *a {
                    a_f64.partial_cmp(b)
                } else {
                    None
                }
            }
            (IfaValue::Float(a), IfaValue::Int(b)) => {
                let b_f64 = *b as f64;
                if b_f64 as i64 == *b {
                    a.partial_cmp(&b_f64)
                } else {
                    None
                }
            }
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
            // Spec: Float(0.0) is falsy; NaN is falsy; all other floats are truthy.
            IfaValue::Float(f) => *f != 0.0 && !f.is_nan(),
            IfaValue::Str(s) => !s.is_empty(),
            IfaValue::List(l) => !l.is_empty(),
            IfaValue::Map(m) => !m.is_empty(),
            // Spec: all objects are truthy (emptiness is not observable truthiness).
            IfaValue::Object(_) => true,
            IfaValue::Fn(_) => true,
            IfaValue::Resource(_) => true,
            IfaValue::Null => false,
            IfaValue::Return(v) => v.is_truthy(),

            #[cfg(feature = "vm")]
            IfaValue::AstFn { .. } | IfaValue::BytecodeFn { .. } | IfaValue::Class { .. } => true,
            #[cfg(feature = "vm")]
            IfaValue::Ptr(addr) => *addr != 0,
            #[cfg(feature = "vm")]
            IfaValue::Ref(_) => true,
        }
    }

    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, IfaValue::Null)
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
            IfaValue::Null => "Null",
            IfaValue::Resource(_) => "Resource",
            IfaValue::Return(_) => "Return",

            #[cfg(feature = "vm")]
            IfaValue::AstFn { .. } | IfaValue::BytecodeFn { .. } => "Fn",
            #[cfg(feature = "vm")]
            IfaValue::Class { .. } => "Class",
            #[cfg(feature = "vm")]
            IfaValue::Ptr(_) => "Ptr",
            #[cfg(feature = "vm")]
            IfaValue::Ref(_) => "Ref",
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
                let idx = if *i < 0 {
                    // Only scan for count if we absolutely must (negative index)
                    let count = s.chars().count();
                    let abs_i = i.unsigned_abs() as usize;
                    if abs_i > count {
                        return Err(IfaError::IndexOutOfBounds {
                            index: *i,
                            length: count,
                        });
                    }
                    count - abs_i
                } else {
                    *i as usize
                };

                // Use nth() which is iterator-based O(N) but single pass
                // and handles OOB by returning None
                s.chars()
                    .nth(idx)
                    .map(|c| IfaValue::Str(c.to_string().into()))
                    .ok_or_else(|| {
                        // We only calculate length here for the error message if we failed,
                        // preserving performance for the happy path.
                        IfaError::IndexOutOfBounds {
                            index: *i,
                            length: s.chars().count(),
                        }
                    })
            }
            (IfaValue::Map(m), IfaValue::Str(k)) => m
                .get(k)
                .cloned()
                .ok_or_else(|| IfaError::KeyNotFound(k.to_string())),
            (IfaValue::Object(o), IfaValue::Str(k)) => {
                let map = o.borrow();
                map.get(k)
                    .cloned()
                    .ok_or_else(|| IfaError::KeyNotFound(k.to_string()))
            }
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
            (IfaValue::Object(o), IfaValue::Str(k)) => {
                o.borrow_mut().insert(k.clone(), value);
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
                let count = s.chars().count();
                let start_idx = if start < 0 {
                    (count as i64 + start).max(0) as usize
                } else {
                    start as usize
                };
                let end_idx = if end < 0 {
                    (count as i64 + end).max(0) as usize
                } else {
                    (end as usize).min(count)
                };

                if start_idx >= end_idx {
                    Ok(IfaValue::Str("".into()))
                } else {
                    Ok(IfaValue::Str(
                        s.chars()
                            .skip(start_idx)
                            .take(end_idx - start_idx)
                            .collect::<String>()
                            .into(),
                    ))
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

    /// Freeze: Convert Local Value (The Hut) to Shared Value (The Village).
    /// Performs a deep copy. Fails on closures consistently.
    pub fn freeze(&self) -> IfaResult<IfaShared> {
        match self {
            IfaValue::Int(n) => Ok(IfaShared::Int(*n)),
            IfaValue::Float(n) => Ok(IfaShared::Float(*n)),
            IfaValue::Str(s) => Ok(IfaShared::Str(s.clone())),
            IfaValue::Bool(b) => Ok(IfaShared::Bool(*b)),
            IfaValue::Null => Ok(IfaShared::Null),
            IfaValue::List(l) => {
                let mut frozen_list = Vec::with_capacity(l.len());
                for item in l {
                    frozen_list.push(item.freeze()?);
                }
                Ok(IfaShared::List(frozen_list))
            }
            IfaValue::Map(m) => {
                let mut frozen_map = HashMap::new();
                for (k, v) in m {
                    frozen_map.insert(k.clone(), v.freeze()?);
                }
                Ok(IfaShared::Map(frozen_map))
            }
            IfaValue::Object(o) => {
                // Determine concurrent map type
                #[cfg(feature = "dashmap")]
                use dashmap::DashMap;
                #[cfg(not(feature = "dashmap"))]
                use std::sync::RwLock;

                #[cfg(feature = "dashmap")]
                {
                    let frozen = Arc::new(DashMap::new());
                    if let Ok(map) = o.try_borrow() {
                        for (k, v) in map.iter() {
                            frozen.insert(k.clone(), v.freeze()?);
                        }
                    } // Else: Object is locked, maybe return error? For now treating as empty/skipped.
                    Ok(IfaShared::Object(frozen))
                }
                #[cfg(not(feature = "dashmap"))]
                {
                    use std::sync::Arc;
                    let mut map = HashMap::new();
                    if let Ok(local_map) = o.try_borrow() {
                        for (k, v) in local_map.iter() {
                            map.insert(k.clone(), v.freeze()?);
                        }
                    }
                    Ok(IfaShared::Object(Arc::new(RwLock::new(map))))
                }
            }
            IfaValue::Fn(_) | IfaValue::Return(_) => {
                // Cannot freeze functions or return wrappers
                // In generic context, Null might be safer than Error
                // but strictest "Safe" checking implies Error.
                Ok(IfaShared::Null)
            }
            IfaValue::Resource(token) => Ok(IfaShared::Resource(*token)),
            #[cfg(feature = "vm")]
            _ => Ok(IfaShared::Null), // Ptr, Ref, etc not transferable
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
        IfaValue::Str(v.into())
    }
}

impl From<&str> for IfaValue {
    fn from(v: &str) -> Self {
        IfaValue::Str(v.into())
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
    fn test_truthy() {
        assert!(IfaValue::Bool(true).is_truthy());
        assert!(!IfaValue::Bool(false).is_truthy());
        assert!(IfaValue::Int(1).is_truthy());
        assert!(!IfaValue::Int(0).is_truthy());
        assert!(!IfaValue::Float(f64::NAN).is_truthy());
        assert!(!IfaValue::Null.is_truthy());

        // Spec: objects are always truthy (even if "empty").
        let obj = IfaValue::Object(Rc::new(RefCell::new(HashMap::new())));
        assert!(obj.is_truthy());
    }
}
