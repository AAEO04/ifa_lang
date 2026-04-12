//! # Unified Type System (Clean Enum Implementation)
//!
//! This module implements `IfaValue` as a safe, reference-counted enum.
//! No manual memory management. No unsafe unions. pure Rust.

#[cfg(feature = "serde")]
use serde::de::Error as DeError;
#[cfg(feature = "serde")]
use serde::ser::Error as SerError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
#[cfg(feature = "vm")]
use std::sync::Mutex;

#[cfg(feature = "vm")]
use crate::ast::Statement;
use crate::token::ResourceToken;

// ============================================================================
// 1. Core Implementation (The "Nano-Boxed" Enum)
// ============================================================================

/// Universal value type for the Ifá-Lang Host Runtime.
///
/// Layout on 64-bit systems: 16 bytes.
/// - Tag: 1 byte
/// - Padding: 7 bytes
/// - Payload: 8 bytes (i64, f64, or Arc pointer)
#[derive(Clone, Debug)]
pub enum IfaValue {
    // 1. Primitives (Inline, No Alloc)
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),

    // 2. Heap Objects (Ref-Counted, Shared)
    Str(Arc<str>),
    List(Arc<Vec<IfaValue>>),
    Map(Arc<HashMap<Arc<str>, IfaValue>>),

    // 3. Special / VM Objects
    Fn(Arc<BytecodeFnData>),

    /// AST function (interpreter) with captured environment id.
    #[cfg(feature = "vm")]
    AstFn(Arc<AstFnData>),

    /// Boxed/captured binding cell (closure upvalue).
    #[cfg(feature = "vm")]
    Upvalue(UpvalueCell),

    /// Bytecode closure: function template + captured environment.
    #[cfg(feature = "vm")]
    Closure(Arc<ClosureData>),
    /// Async future value (VM/AST only).
    #[cfg(feature = "vm")]
    Future(FutureCell),

    // Legacy / Other
    #[allow(dead_code)]
    Resource(Arc<ResourceToken>),

    // VM Specific
    #[cfg(feature = "vm")]
    Return(Arc<IfaValue>),

    // 4. Okanran (Error Handling)
    Result(bool, Box<IfaValue>),
}

// ============================================================================
// VM support types
// ============================================================================

/// Shared mutable cell used for closure capture (by-reference semantics).
#[cfg(feature = "vm")]
pub type UpvalueCell = Arc<Mutex<IfaValue>>;

/// Closure payload for the bytecode VM.
#[cfg(feature = "vm")]
#[derive(Clone, Debug)]
pub struct ClosureData {
    pub fn_data: Arc<BytecodeFnData>,
    pub env: Arc<Vec<UpvalueCell>>,
}

// ========================================================================
// Async futures (minimal runtime)
// ========================================================================

#[cfg(feature = "vm")]
#[derive(Clone, Debug)]
pub enum FutureState {
    Pending,
    Ready(IfaValue),
}

#[cfg(feature = "vm")]
pub type FutureCell = Arc<Mutex<FutureState>>;

// ============================================================================
// 2. Constructors & Helpers
// ============================================================================

impl IfaValue {
    // --- Primitives ---
    #[inline(always)]
    pub const fn null() -> Self {
        IfaValue::Null
    }

    #[inline(always)]
    pub const fn bool(b: bool) -> Self {
        IfaValue::Bool(b)
    }

    #[inline(always)]
    pub const fn int(n: i64) -> Self {
        IfaValue::Int(n)
    }

    #[inline(always)]
    pub const fn float(f: f64) -> Self {
        IfaValue::Float(f)
    }

    // --- Heap Types ---
    pub fn str(s: impl Into<String>) -> Self {
        IfaValue::Str(Arc::from(s.into().into_boxed_str()))
    }

    pub fn list(items: Vec<IfaValue>) -> Self {
        IfaValue::List(Arc::new(items))
    }

    pub fn map(m: HashMap<String, IfaValue>) -> Self {
        let mut internal = HashMap::with_capacity(m.len());
        for (k, v) in m {
            internal.insert(Arc::from(k.into_boxed_str()), v);
        }
        IfaValue::Map(Arc::new(internal))
    }

    #[cfg(feature = "vm")]
    pub fn bytecode_fn(
        name: impl Into<String>,
        start_ip: usize,
        arity: u8,
        is_async: bool,
    ) -> Self {
        IfaValue::Fn(Arc::new(BytecodeFnData {
            name: name.into(),
            start_ip,
            arity,
            is_async,
        }))
    }

    #[cfg(feature = "vm")]
    pub fn return_value(val: IfaValue) -> Self {
        IfaValue::Return(Arc::new(val))
    }

    #[cfg(feature = "vm")]
    pub fn future_ready(val: IfaValue) -> Self {
        IfaValue::Future(Arc::new(Mutex::new(FutureState::Ready(val))))
    }

    #[cfg(feature = "vm")]
    pub fn future_pending() -> Self {
        IfaValue::Future(Arc::new(Mutex::new(FutureState::Pending)))
    }

    pub fn ok(val: IfaValue) -> Self {
        IfaValue::Result(true, Box::new(val))
    }

    pub fn err(val: IfaValue) -> Self {
        IfaValue::Result(false, Box::new(val))
    }

    pub fn is_return(&self) -> bool {
        #[cfg(feature = "vm")]
        {
            matches!(self, IfaValue::Return(_))
        }
        #[cfg(not(feature = "vm"))]
        {
            false
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            IfaValue::Null => "Null",
            IfaValue::Bool(_) => "Bool",
            IfaValue::Int(_) => "Int",
            IfaValue::Float(_) => "Float",
            IfaValue::Str(_) => "Str",
            IfaValue::List(_) => "List",
            IfaValue::Map(_) => "Map",
            IfaValue::Fn(_) => "Fn",
            #[cfg(feature = "vm")]
            IfaValue::AstFn(_) => "Fn",
            IfaValue::Result(_, _) => "Result",
            #[cfg(feature = "vm")]
            IfaValue::Upvalue(_) => "Upvalue",
            #[cfg(feature = "vm")]
            IfaValue::Closure(_) => "Closure",
            #[cfg(feature = "vm")]
            IfaValue::Future(_) => "Future",
            _ => "Unknown",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            IfaValue::Null => false,
            IfaValue::Bool(b) => *b,
            IfaValue::Int(i) => *i != 0,
            IfaValue::Float(f) => *f != 0.0 && !f.is_nan(),
            IfaValue::Str(s) => !s.is_empty(),
            IfaValue::List(l) => !l.is_empty(),
            IfaValue::Map(m) => !m.is_empty(),
            IfaValue::Fn(_) => true,
            #[cfg(feature = "vm")]
            IfaValue::AstFn(_) => true,
            #[cfg(feature = "vm")]
            IfaValue::Closure(_) => true,
            #[cfg(feature = "vm")]
            IfaValue::Return(v) => v.is_truthy(),
            IfaValue::Result(_, _) => true,
            #[cfg(feature = "vm")]
            IfaValue::Future(_) => true,
            #[cfg(feature = "vm")]
            IfaValue::Upvalue(cell) => cell
                .lock()
                .ok()
                .as_deref()
                .map(IfaValue::is_truthy)
                .unwrap_or(false),
            #[allow(unreachable_patterns)]
            _ => true,
        }
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (IfaValue::Null, IfaValue::Null) => true,
            (IfaValue::Bool(a), IfaValue::Bool(b)) => a == b,
            (IfaValue::Int(a), IfaValue::Int(b)) => a == b,
            (IfaValue::Float(a), IfaValue::Float(b)) => (a - b).abs() < f64::EPSILON,
            (IfaValue::Str(a), IfaValue::Str(b)) => a == b,
            (IfaValue::List(a), IfaValue::List(b)) => {
                if Arc::ptr_eq(a, b) {
                    return true;
                }
                if a.len() != b.len() {
                    return false;
                }
                a.iter().zip(b.iter()).all(|(x, y)| x.is_equal(y))
            }
            (IfaValue::Map(a), IfaValue::Map(b)) => {
                if Arc::ptr_eq(a, b) {
                    return true;
                }
                if a.len() != b.len() {
                    return false;
                }
                a.iter()
                    .all(|(k, v)| b.get(k).map_or(false, |bv| v.is_equal(bv)))
            }
            _ => false,
        }
    }
}

// ============================================================================
// 3. Trait Impls
// ============================================================================

impl PartialEq for IfaValue {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}

impl Eq for IfaValue {}

impl fmt::Display for IfaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IfaValue::Null => write!(f, "null"),
            IfaValue::Bool(b) => write!(f, "{}", b),
            IfaValue::Int(i) => write!(f, "{}", i),
            IfaValue::Float(fl) => write!(f, "{}", fl),
            IfaValue::Str(s) => write!(f, "{}", s),
            IfaValue::List(_) => write!(f, "[List]"),
            IfaValue::Map(_) => write!(f, "{{Map}}"),
            IfaValue::Fn(_) => write!(f, "<fn>"),
            #[cfg(feature = "vm")]
            IfaValue::AstFn(data) => write!(f, "<fn {}>", data.name),
            IfaValue::Result(ok, val) => {
                if *ok {
                    write!(f, "Ok({})", val)
                } else {
                    write!(f, "Err({})", val)
                }
            }
            #[cfg(feature = "vm")]
            IfaValue::Future(_) => write!(f, "<future>"),
            _ => write!(f, "<?>"),
        }
    }
}

// Support unary ! operator (Not)
impl std::ops::Not for IfaValue {
    type Output = Self;
    fn not(self) -> Self::Output {
        IfaValue::Bool(!self.is_truthy())
    }
}

// ============================================================================
// 4. Serde — bincode-safe surrogate enum
//
// bincode does NOT support deserialize_any (it is a non-self-describing format).
// We use a surrogate enum tagged by variant index, which bincode handles fine.
// ============================================================================

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
enum IfaValueSurrogate {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<IfaValue>),
    /// Placeholder for non-serializable variants (Fn, Closure, Class, etc.)
    Unsupported,
}

#[cfg(feature = "serde")]
impl Serialize for IfaValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let surrogate = match self {
            IfaValue::Null => IfaValueSurrogate::Null,
            IfaValue::Bool(b) => IfaValueSurrogate::Bool(*b),
            IfaValue::Int(i) => IfaValueSurrogate::Int(*i),
            IfaValue::Float(f) => IfaValueSurrogate::Float(*f),
            IfaValue::Str(s) => IfaValueSurrogate::Str(s.to_string()),
            IfaValue::List(l) => {
                let inner = l.iter().cloned().collect();
                IfaValueSurrogate::List(inner)
            }
            other => {
                return Err(S::Error::custom(format!(
                    "IfaValue variant '{}' is not serializable",
                    other.type_name()
                )));
            }
        };
        surrogate.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for IfaValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let surrogate = IfaValueSurrogate::deserialize(deserializer)?;
        Ok(match surrogate {
            IfaValueSurrogate::Null => IfaValue::null(),
            IfaValueSurrogate::Bool(b) => IfaValue::bool(b),
            IfaValueSurrogate::Int(i) => IfaValue::Int(i),
            IfaValueSurrogate::Float(f) => IfaValue::Float(f),
            IfaValueSurrogate::Str(s) => IfaValue::str(s),
            IfaValueSurrogate::List(l) => IfaValue::list(l),
            IfaValueSurrogate::Unsupported => {
                return Err(D::Error::custom(
                    "unsupported IfaValue surrogate in serialized data",
                ));
            }
        })
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;

    #[test]
    fn unsupported_values_fail_serialization() {
        let value = IfaValue::Fn(Arc::new(BytecodeFnData {
            name: "f".to_string(),
            start_ip: 0,
            arity: 0,
            is_async: false,
        }));

        let err = bincode::serialize(&value).expect_err("expected serialization failure");
        let msg = err.to_string();
        assert!(msg.contains("not serializable"));
    }
}

// ============================================================================
// 5. Supporting Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytecodeFnData {
    pub name: String,
    pub start_ip: usize,
    pub arity: u8,
    pub is_async: bool,
}

#[cfg(feature = "vm")]
#[derive(Debug, Clone)]
pub struct AstFnData {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
    pub closure_id: u64,
    pub is_async: bool,
}
