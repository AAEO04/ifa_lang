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
#[cfg(feature = "std")]
use std::sync::OnceLock;

#[cfg(feature = "std")]
// Dashmap removed for PR-28 / I-Stream (No global caching)
#[cfg(feature = "vm")]
use crate::ast::Statement;
use crate::error::{IfaError, IfaResult};
use crate::nan_box::{BoxedPrimitive, NanBox};
use crate::shared::IfaShared;
use crate::token::ResourceToken;

// ============================================================================
// 1. Core Implementation (The "Nano-Boxed" Enum)
// ============================================================================

/// Universal value type for the Ifá-Lang Host Runtime.
///
/// Layout on 64-bit systems: currently 32 bytes.
/// This is a regular Rust enum, so the final size is driven by the
/// discriminator plus the largest variant payload, not a hand-packed union.
#[derive(Clone, Debug)]
pub enum IfaValue {
    // 1. Primitives (Inline, No Alloc)
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),

    // 2. Heap Objects (Ref-Counted, Shared)
    Str(crate::CompactString),
    List(Arc<Vec<IfaValue>>),
    Map(Arc<HashMap<crate::CompactString, IfaValue>>),

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
    /// H2: Actor handle — a reference to a running isolated VM thread.
    /// Uses type-erased Arc so ifa-types has no dependency on ifa-vm's ActorHandle.
    /// Callers in ifa-vm downcast via `Arc::downcast` after cloning.
    #[cfg(feature = "vm")]
    Actor {
        /// Monotonic actor ID for routing and display.
        id: u64,
        /// Type-erased SyncSender<ActorMsg>. Downcast in ifa-vm.
        handle: Arc<dyn std::any::Any + Send + Sync>,
    },

    // Legacy / Other
    #[allow(dead_code)]
    Resource(Arc<ResourceToken>),

    // VM Specific
    #[cfg(feature = "vm")]
    Return(Arc<IfaValue>),
    /// Loop break signal — consumed by the nearest While/For handler.
    #[cfg(feature = "vm")]
    Break,
    /// Loop continue signal — consumed by the nearest While/For handler.
    #[cfg(feature = "vm")]
    Continue,

    // 4. Okanran (Error Handling)
    Result(Box<ResultPayload>),
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

#[derive(Clone, Debug)]
pub enum ResultPayload {
    Ok(IfaValue),
    Err(IfaValue),
}

// ============================================================================
// 2. Constructors & Helpers
// ============================================================================

impl IfaValue {
    /// Unicode scalar length for a string value.
    ///
    /// The global DashMap cache has been removed to prevent memory exhaustion and
    /// mutex contention. Lengths are computed dynamically until the VM implements
    /// an O(1) integer-indexed local cache (PR-28).
    pub fn unicode_string_len(s: &str) -> usize {
        if s.is_ascii() {
            return s.len();
        }
        s.chars().count()
    }

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
    pub fn int(n: i64) -> Self {
        #[cfg(feature = "std")]
        {
            static SMALL_INT_POOL: OnceLock<[IfaValue; 256]> = OnceLock::new();

            if (0..=255).contains(&n) {
                let pool =
                    SMALL_INT_POOL.get_or_init(|| std::array::from_fn(|i| IfaValue::Int(i as i64)));
                return pool[n as usize].clone();
            }
        }

        IfaValue::Int(n)
    }

    #[inline(always)]
    pub const fn float(f: f64) -> Self {
        IfaValue::Float(f)
    }

    // --- Heap Types ---
    pub fn str(s: impl Into<String>) -> Self {
        IfaValue::Str(crate::CompactString::new(&s.into()))
    }

    pub fn list(items: Vec<IfaValue>) -> Self {
        IfaValue::List(Arc::new(items))
    }

    pub fn map(m: HashMap<String, IfaValue>) -> Self {
        let mut internal = HashMap::with_capacity(m.len());
        for (k, v) in m {
            internal.insert(crate::CompactString::new(&k), v);
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
        IfaValue::Result(Box::new(ResultPayload::Ok(val)))
    }

    pub fn err(val: IfaValue) -> Self {
        IfaValue::Result(Box::new(ResultPayload::Err(val)))
    }

    /// Convert an inline primitive into the initial NaN-boxed representation.
    ///
    /// Heap-backed variants are intentionally excluded until pointer tagging is
    /// migrated.
    pub fn to_nan_boxed_primitive(&self) -> Option<NanBox> {
        match self {
            IfaValue::Null => Some(NanBox::from_null()),
            IfaValue::Bool(b) => Some(NanBox::from_bool(*b)),
            IfaValue::Int(i) => NanBox::from_int(*i).ok(),
            IfaValue::Float(f) => Some(NanBox::from_float(*f)),
            _ => None,
        }
    }

    /// Reconstruct an `IfaValue` from the primitive NaN-boxed subset.
    pub fn from_nan_boxed_primitive(value: NanBox) -> Option<Self> {
        match value.to_primitive().ok()? {
            BoxedPrimitive::Null => Some(Self::null()),
            BoxedPrimitive::Bool(b) => Some(Self::bool(b)),
            BoxedPrimitive::Int(i) => Some(Self::int(i)),
            BoxedPrimitive::Float(f) => Some(Self::float(f)),
        }
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

    pub fn is_break(&self) -> bool {
        #[cfg(feature = "vm")]
        {
            matches!(self, IfaValue::Break)
        }
        #[cfg(not(feature = "vm"))]
        {
            false
        }
    }

    pub fn is_continue(&self) -> bool {
        #[cfg(feature = "vm")]
        {
            matches!(self, IfaValue::Continue)
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
            IfaValue::Result(_) => "Result",
            #[cfg(feature = "vm")]
            IfaValue::Upvalue(_) => "Upvalue",
            #[cfg(feature = "vm")]
            IfaValue::Closure(_) => "Closure",
            #[cfg(feature = "vm")]
            IfaValue::Future(_) => "Future",
            #[cfg(feature = "vm")]
            IfaValue::Actor { .. } => "Actor",
            _ => "Unknown",
        }
    }

    pub fn is_truthy(&self) -> bool {
        if let Some(boxed) = self.to_nan_boxed_primitive() {
            return boxed.is_truthy();
        }

        match self {
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
            IfaValue::Result(_) => true,
            #[cfg(feature = "vm")]
            IfaValue::Future(_) => true,
            #[cfg(feature = "vm")]
            IfaValue::Upvalue(cell) => cell
                .try_lock()
                .ok()
                .map(|value| value.is_truthy())
                .unwrap_or(false),
            #[allow(unreachable_patterns)]
            _ => true,
        }
    }

    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, IfaValue::Null)
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
                    .all(|(k, v)| b.get(k).is_some_and(|bv| v.is_equal(bv)))
            }
            (IfaValue::Result(a), IfaValue::Result(b)) => match (a.as_ref(), b.as_ref()) {
                (ResultPayload::Ok(av), ResultPayload::Ok(bv))
                | (ResultPayload::Err(av), ResultPayload::Err(bv)) => av.is_equal(bv),
                _ => false,
            },
            #[cfg(feature = "vm")]
            (IfaValue::Upvalue(a), IfaValue::Upvalue(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }

    /// Freeze: Convert Local Value (The Hut) to Shared Value (The Village).
    /// Performs a deep copy. Fails on closures/functions consistently.
    pub fn freeze(&self) -> IfaResult<IfaShared> {
        match self {
            IfaValue::Int(n) => Ok(IfaShared::Int(*n)),
            IfaValue::Float(n) => Ok(IfaShared::Float(*n)),
            IfaValue::Str(s) => Ok(IfaShared::Str(s.as_str().into())),
            IfaValue::Bool(b) => Ok(IfaShared::Bool(*b)),
            IfaValue::Null => Ok(IfaShared::Null),
            IfaValue::List(l) => {
                let mut frozen_list = Vec::with_capacity(l.len());
                for item in l.iter() {
                    frozen_list.push(item.freeze()?);
                }
                Ok(IfaShared::List(frozen_list))
            }
            IfaValue::Map(m) => {
                let mut frozen_map = HashMap::new();
                for (k, v) in m.iter() {
                    frozen_map.insert(k.as_str().into(), v.freeze()?);
                }
                Ok(IfaShared::Map(frozen_map))
            }
            IfaValue::Resource(token) => Ok(IfaShared::Resource(**token)),
            _ => Err(IfaError::Runtime(format!(
                "Cannot freeze value of type {} for thread-safe sharing",
                self.type_name()
            ))),
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

impl std::cmp::PartialOrd for IfaValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
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
            IfaValue::Result(payload) => match payload.as_ref() {
                ResultPayload::Ok(val) => write!(f, "Ok({})", val),
                ResultPayload::Err(val) => write!(f, "Err({})", val),
            },
            #[cfg(feature = "vm")]
            IfaValue::Future(_) => write!(f, "<future>"),
            #[cfg(feature = "vm")]
            IfaValue::Actor { id, .. } => write!(f, "<actor:{id}>"),
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

    #[test]
    fn unicode_string_len_counts_code_points() {
        assert_eq!(IfaValue::unicode_string_len("hello"), 5);
        assert_eq!(IfaValue::unicode_string_len("e\u{301}"), 2);
        assert_eq!(IfaValue::unicode_string_len("🔥a"), 2);
    }
    #[test]
    fn nan_boxed_primitive_roundtrip_matches_ifa_value() {
        let values = [
            IfaValue::null(),
            IfaValue::bool(false),
            IfaValue::bool(true),
            IfaValue::int(42),
            IfaValue::int(-42),
            IfaValue::float(2.5),
        ];

        for value in values {
            let boxed = value
                .to_nan_boxed_primitive()
                .expect("primitive should box");
            let roundtrip =
                IfaValue::from_nan_boxed_primitive(boxed).expect("boxed primitive should unbox");
            assert_eq!(roundtrip, value);
        }
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    #[test]
    fn ifa_value_stays_within_32_bytes_on_64_bit() {
        assert_eq!(std::mem::size_of::<IfaValue>(), 32);
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
