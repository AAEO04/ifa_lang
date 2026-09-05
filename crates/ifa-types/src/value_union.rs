//! # Unified Type System (Clean Enum Implementation)
//!
//! This module implements `IfaValue` as a safe, reference-counted enum.
//! No manual memory management. No unsafe unions. pure Rust.

use crate::gc::{IfaGc, Trace};
#[cfg(feature = "serde")]
use serde::de::Error as DeError;
#[cfg(feature = "serde")]
use serde::ser::Error as SerError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hasher;
use std::sync::Arc;
#[cfg(feature = "vm")]
use std::sync::Mutex;

#[cfg(feature = "std")]
// Dashmap removed for PR-28 / I-Stream (No global caching)
#[cfg(feature = "vm")]
use crate::ast::Statement;
use crate::error::{IfaError, IfaResult};
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
    Str(Box<String>),
    List(IfaGc<Vec<IfaValue>>),
    Map(IfaGc<HashMap<crate::CompactString, IfaValue>>),
    Set(Arc<std::collections::HashSet<IfaValue>>),

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
    Closure(IfaGc<ClosureData>),
    /// Async future value (VM/AST only).
    #[cfg(feature = "vm")]
    Future(FutureCell),
    #[cfg(feature = "vm")]
    NativeFuture(NativeFutureCell),
    /// H2: Actor handle — externalized to keep IfaValue at 16 bytes.
    /// Uses type-erased Arc so ifa-types has no dependency on ifa-vm's ActorHandle.
    /// Callers in ifa-vm downcast via `Arc::downcast` after cloning.
    #[cfg(feature = "vm")]
    Actor(Arc<ActorData>),

    // Legacy / Other
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
    /// Moved value (ownership transferred)
    #[cfg(feature = "vm")]
    Moved,

    // 4. Okanran (Error Handling)
    Result(Box<ResultPayload>),
}

// ============================================================================
// VM support types
// ============================================================================

/// Shared mutable cell used for closure capture (by-reference semantics).
#[cfg(feature = "vm")]
pub type UpvalueCell = IfaGc<Mutex<IfaValue>>;

/// Closure payload for the bytecode VM.
#[cfg(feature = "vm")]
#[derive(Clone, Debug)]
pub struct ClosureData {
    pub fn_data: Arc<BytecodeFnData>,
    pub env: Arc<Vec<UpvalueCell>>,
}

/// Actor handle payload — externalized to keep IfaValue at 16 bytes.
#[cfg(feature = "vm")]
#[derive(Clone, Debug)]
pub struct ActorData {
    /// Monotonic actor ID for routing and display.
    pub id: u64,
    /// Type-erased SyncSender<ActorMsg>. Downcast in ifa-vm.
    pub handle: Arc<dyn std::any::Any + Send + Sync>,
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
pub type FutureCell = Arc<std::sync::RwLock<FutureState>>;

#[cfg(feature = "vm")]
#[derive(Clone, Debug)]
pub enum NativeFutureState {
    Pending,
    Ready(Vec<u8>), // bincode serialized IfaValue
    Error(String),
}

#[cfg(feature = "vm")]
pub type NativeFutureCell = std::sync::Arc<std::sync::RwLock<NativeFutureState>>;

#[derive(Clone, Debug)]
pub enum ResultPayload {
    Ire(IfaValue),
    Ibi(IfaValue),
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
        IfaValue::Int(n)
    }

    #[inline(always)]
    pub const fn float(f: f64) -> Self {
        IfaValue::Float(f)
    }

    // --- Heap Types ---
    pub fn str(s: impl Into<String>) -> Self {
        IfaValue::Str(Box::new(s.into()))
    }

    pub fn list(items: Vec<IfaValue>) -> Self {
        IfaValue::List(IfaGc::new(items))
    }

    pub fn map(m: HashMap<String, IfaValue>) -> Self {
        let mut internal = HashMap::with_capacity(m.len());
        for (k, v) in m {
            internal.insert(crate::CompactString::new(&k), v);
        }
        IfaValue::Map(IfaGc::new(internal))
    }

    #[allow(clippy::mutable_key_type)]
    pub fn set(s: std::collections::HashSet<IfaValue>) -> Self {
        IfaValue::Set(Arc::new(s))
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
        IfaValue::Future(Arc::new(std::sync::RwLock::new(FutureState::Ready(val))))
    }

    #[cfg(feature = "vm")]
    pub fn future_pending() -> Self {
        IfaValue::Future(Arc::new(std::sync::RwLock::new(FutureState::Pending)))
    }

    pub fn ire(val: IfaValue) -> Self {
        IfaValue::Result(Box::new(ResultPayload::Ire(val)))
    }

    pub fn ibi(val: IfaValue) -> Self {
        IfaValue::Result(Box::new(ResultPayload::Ibi(val)))
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
            IfaValue::Set(_) => "Set",
            IfaValue::Fn(_) => "Fn",
            #[cfg(feature = "vm")]
            IfaValue::AstFn(_) => "Fn",
            IfaValue::Result(_) => "Result",
            #[cfg(feature = "vm")]
            IfaValue::Upvalue(_) => "Upvalue",
            #[cfg(feature = "vm")]
            IfaValue::Closure(_) => "Closure",
            #[cfg(feature = "vm")]
            #[cfg(feature = "vm")]
            IfaValue::NativeFuture(_) => "NativeFuture",
            #[cfg(feature = "vm")]
            IfaValue::Actor(_) => "Actor",
            #[cfg(feature = "vm")]
            IfaValue::Moved => "Moved",
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
            IfaValue::Set(s) => !s.is_empty(),
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
            #[cfg(feature = "vm")]
            IfaValue::Moved => false,
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
            (IfaValue::Float(a), IfaValue::Float(b)) => {
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b
                }
            }
            (IfaValue::Str(a), IfaValue::Str(b)) => a == b,
            (IfaValue::List(a), IfaValue::List(b)) => {
                // IfaGc eq fast paths ptr eq
                if IfaGc::ptr_eq(a, b) {
                    return true;
                }
                if a.len() != b.len() {
                    return false;
                }
                a.iter().zip(b.iter()).all(|(x, y)| x.is_equal(y))
            }
            (IfaValue::Set(a), IfaValue::Set(b)) => {
                if Arc::ptr_eq(a, b) {
                    return true;
                }
                if a.len() != b.len() {
                    return false;
                }
                a.iter().all(|x| b.contains(x))
            }
            (IfaValue::Map(a), IfaValue::Map(b)) => {
                if IfaGc::ptr_eq(a, b) {
                    return true;
                }
                if a.len() != b.len() {
                    return false;
                }
                a.iter()
                    .all(|(k, v)| b.get(k).is_some_and(|bv| v.is_equal(bv)))
            }
            (IfaValue::Result(a), IfaValue::Result(b)) => match (a.as_ref(), b.as_ref()) {
                (ResultPayload::Ire(av), ResultPayload::Ire(bv))
                | (ResultPayload::Ibi(av), ResultPayload::Ibi(bv)) => av.is_equal(bv),
                _ => false,
            },
            #[cfg(feature = "vm")]
            (IfaValue::Upvalue(a), IfaValue::Upvalue(b)) => IfaGc::ptr_eq(a, b),
            (IfaValue::Fn(a), IfaValue::Fn(b)) => Arc::ptr_eq(a, b),
            #[cfg(feature = "vm")]
            (IfaValue::AstFn(a), IfaValue::AstFn(b)) => Arc::ptr_eq(a, b),
            #[cfg(feature = "vm")]
            (IfaValue::Closure(a), IfaValue::Closure(b)) => IfaGc::ptr_eq(a, b),
            #[cfg(feature = "vm")]
            (IfaValue::Actor(a), IfaValue::Actor(b)) => {
                a.id == b.id && Arc::ptr_eq(&a.handle, &b.handle)
            }
            #[cfg(feature = "vm")]
            (IfaValue::Future(a), IfaValue::Future(b)) => Arc::ptr_eq(a, b),
            #[cfg(feature = "vm")]
            (IfaValue::NativeFuture(a), IfaValue::NativeFuture(b)) => Arc::ptr_eq(a, b),
            #[cfg(feature = "vm")]
            (IfaValue::Return(a), IfaValue::Return(b)) => a.is_equal(b),
            #[cfg(feature = "vm")]
            (IfaValue::Break, IfaValue::Break) => true,
            #[cfg(feature = "vm")]
            (IfaValue::Continue, IfaValue::Continue) => true,
            #[cfg(feature = "vm")]
            (IfaValue::Moved, IfaValue::Moved) => true,
            (IfaValue::Resource(a), IfaValue::Resource(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }

    /// Freeze: Convert Local Value (The Hut) to Shared Value (The Village).
    /// Performs a deep copy. Fails on closures/functions consistently.
    pub fn freeze(&self) -> IfaResult<IfaShared> {
        match self {
            IfaValue::Int(n) => Ok(IfaShared::Int(*n)),
            IfaValue::Float(n) => Ok(IfaShared::Float(*n)),
            IfaValue::Str(s) => Ok(IfaShared::Str(Arc::from(s.as_str()))),
            IfaValue::Bool(b) => Ok(IfaShared::Bool(*b)),
            IfaValue::Null => Ok(IfaShared::Null),
            IfaValue::List(l) => {
                let mut frozen_list = Vec::with_capacity(l.len());
                for item in l.iter() {
                    frozen_list.push(item.freeze()?);
                }
                Ok(IfaShared::List(frozen_list))
            }
            IfaValue::Set(_s) => {
                // Shared sets not supported yet
                Err(IfaError::Runtime("Cannot freeze Set".into()))
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

impl std::hash::Hash for IfaValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            IfaValue::Null => {}
            IfaValue::Bool(b) => b.hash(state),
            IfaValue::Int(i) => i.hash(state),
            IfaValue::Float(f) => {
                if f.is_nan() {
                    f64::NAN.to_bits().hash(state);
                } else {
                    f.to_bits().hash(state);
                }
            }
            IfaValue::Str(s) => s.hash(state),
            IfaValue::List(l) => l.hash(state),
            IfaValue::Set(s) => {
                // Sum of element hashes — order-independent, matches structural contains().
                let mut combined = 0u64;
                for elem in s.iter() {
                    let mut h = std::collections::hash_map::DefaultHasher::new();
                    elem.hash(&mut h);
                    combined = combined.wrapping_add(h.finish());
                }
                combined.hash(state);
            }
            IfaValue::Map(m) => {
                // Sum of (key_hash XOR value_hash) — order-independent, matches structural eq.
                let mut combined = 0u64;
                for (k, v) in m.iter() {
                    let mut h = std::collections::hash_map::DefaultHasher::new();
                    k.hash(&mut h);
                    let kv = h.finish();
                    v.hash(&mut h);
                    combined = combined.wrapping_add(kv ^ h.finish());
                }
                combined.hash(state);
            }
            IfaValue::Fn(f) => Arc::as_ptr(f).hash(state),
            #[cfg(feature = "vm")]
            IfaValue::AstFn(f) => Arc::as_ptr(f).hash(state),
            #[cfg(feature = "vm")]
            IfaValue::Upvalue(u) => u.ptr.as_ptr().hash(state),
            #[cfg(feature = "vm")]
            IfaValue::Closure(c) => c.ptr.as_ptr().hash(state),
            #[cfg(feature = "vm")]
            IfaValue::Future(f) => Arc::as_ptr(f).hash(state),
            #[cfg(feature = "vm")]
            IfaValue::NativeFuture(f) => Arc::as_ptr(f).hash(state),
            #[cfg(feature = "vm")]
            IfaValue::Actor(data) => data.id.hash(state),
            IfaValue::Resource(r) => Arc::as_ptr(r).hash(state),
            #[cfg(feature = "vm")]
            IfaValue::Return(r) => r.hash(state),
            #[cfg(feature = "vm")]
            IfaValue::Break => {}
            #[cfg(feature = "vm")]
            IfaValue::Continue => {}
            #[cfg(feature = "vm")]
            IfaValue::Moved => {}
            IfaValue::Result(r) => match r.as_ref() {
                ResultPayload::Ire(v) => {
                    0u8.hash(state);
                    v.hash(state);
                }
                ResultPayload::Ibi(v) => {
                    1u8.hash(state);
                    v.hash(state);
                }
            },
        }
    }
}

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
                a_f64.partial_cmp(b)
            }
            (IfaValue::Float(a), IfaValue::Int(b)) => {
                let b_f64 = *b as f64;
                a.partial_cmp(&b_f64)
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
                ResultPayload::Ire(val) => write!(f, "Ire({})", val),
                ResultPayload::Ibi(val) => write!(f, "Ibi({})", val),
            },
            #[cfg(feature = "vm")]
            IfaValue::Future(_) => write!(f, "<future>"),
            #[cfg(feature = "vm")]
            IfaValue::NativeFuture(_) => write!(f, "<native_future>"),
            #[cfg(feature = "vm")]
            IfaValue::Actor(data) => write!(f, "<actor:{}>", data.id),
            #[cfg(feature = "vm")]
            IfaValue::Moved => write!(f, "<moved>"),
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
    Map(HashMap<String, IfaValue>),
    /// Set is serialized as a Vec to avoid HashSet's non-deterministic ordering
    /// issues with some serialization formats. Order is not preserved, but
    /// correctness is: all elements survive a round-trip.
    Set(Vec<IfaValue>),
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
            IfaValue::Map(m) => {
                let mut inner = HashMap::new();
                for (k, v) in m.iter() {
                    inner.insert(k.to_string(), v.clone());
                }
                IfaValueSurrogate::Map(inner)
            }
            IfaValue::Set(s) => {
                // Serialize as a stable Vec; order is non-deterministic but all
                // elements survive the round-trip intact.
                let inner = s.iter().cloned().collect();
                IfaValueSurrogate::Set(inner)
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
            IfaValueSurrogate::Map(m) => IfaValue::map(m),
            IfaValueSurrogate::Set(v) => {
                // Reconstruct the HashSet from the serialized Vec.
                IfaValue::set(v.into_iter().collect())
            }
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
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    use crate::CompactString;
    use std::hash::{Hash, Hasher};

    #[test]
    fn ifa_value_stays_within_16_bytes_on_64_bit() {
        assert!(
            std::mem::size_of::<IfaValue>() <= 16,
            "IfaValue is {} bytes, expected <= 16",
            std::mem::size_of::<IfaValue>()
        );
    }

    #[test]
    fn set_of_maps_hashes_by_content() {
        use std::collections::hash_map::DefaultHasher;
        use std::collections::HashSet;

        let mut map1 = HashMap::new();
        map1.insert(CompactString::new("a"), IfaValue::Int(1));
        let mut map2 = HashMap::new();
        map2.insert(CompactString::new("a"), IfaValue::Int(1));

        let mut set = HashSet::new();
        set.insert(IfaValue::Map(IfaGc::new(map1)));

        let lookup = IfaValue::Map(IfaGc::new(map2));
        assert!(
            set.contains(&lookup),
            "Set::contains must find structurally equal map"
        );

        // Verify hash equality
        let h1 = {
            let mut s = DefaultHasher::new();
            set.iter().next().unwrap().hash(&mut s);
            s.finish()
        };
        let h2 = {
            let mut s = DefaultHasher::new();
            lookup.hash(&mut s);
            s.finish()
        };
        assert_eq!(h1, h2, "structurally equal maps must hash identically");
    }

    #[test]
    fn set_of_sets_hashes_by_content() {
        use std::collections::hash_map::DefaultHasher;
        use std::collections::HashSet;

        let inner1: HashSet<IfaValue> = [IfaValue::Int(1), IfaValue::Int(2)].into_iter().collect();
        let inner2: HashSet<IfaValue> = [IfaValue::Int(2), IfaValue::Int(1)].into_iter().collect();

        let mut outer = HashSet::new();
        outer.insert(IfaValue::Set(Arc::new(inner1)));

        let lookup = IfaValue::Set(Arc::new(inner2));
        assert!(
            outer.contains(&lookup),
            "Set::contains must find structurally equal inner set"
        );

        let h1 = {
            let mut s = DefaultHasher::new();
            outer.iter().next().unwrap().hash(&mut s);
            s.finish()
        };
        let h2 = {
            let mut s = DefaultHasher::new();
            lookup.hash(&mut s);
            s.finish()
        };
        assert_eq!(h1, h2, "structurally equal sets must hash identically");
    }

    #[test]
    fn reference_equality_reflexive() {
        let data = Arc::new(BytecodeFnData {
            name: "f".into(),
            start_ip: 0,
            arity: 0,
            is_async: false,
        });
        let a = IfaValue::Fn(data.clone());
        let b = IfaValue::Fn(data);
        assert!(a == b, "same Arc must be equal");
        assert!(a == a.clone(), "clone must be equal");
    }

    #[test]
    fn cross_variant_equality_false() {
        let fn_val = IfaValue::Fn(Arc::new(BytecodeFnData {
            name: "f".into(),
            start_ip: 0,
            arity: 0,
            is_async: false,
        }));
        let int_val = IfaValue::Int(42);
        assert!(fn_val != int_val);
        assert!(IfaValue::Null != IfaValue::Bool(false));
        assert!(IfaValue::Int(0) != IfaValue::Float(0.0));
    }

    #[test]
    fn partial_ord_int_float_consistent() {
        use std::cmp::Ordering;

        // Equal numeric values
        assert_eq!(
            IfaValue::Int(1).partial_cmp(&IfaValue::Float(1.0)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            IfaValue::Float(1.0).partial_cmp(&IfaValue::Int(1)),
            Some(Ordering::Equal)
        );

        // Int < Float
        assert_eq!(
            IfaValue::Int(1).partial_cmp(&IfaValue::Float(2.0)),
            Some(Ordering::Less)
        );
        assert_eq!(
            IfaValue::Float(1.0).partial_cmp(&IfaValue::Int(2)),
            Some(Ordering::Less)
        );

        // Int > Float
        assert_eq!(
            IfaValue::Int(5).partial_cmp(&IfaValue::Float(2.5)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            IfaValue::Float(5.0).partial_cmp(&IfaValue::Int(2)),
            Some(Ordering::Greater)
        );

        // Large int that is lossy in f64 — still comparable (cast is lossy but consistent).
        // i64::MAX as f64 rounds up to 2^63, so Int(i64::MAX) and Float(2^63) are equal as f64.
        let big = i64::MAX;
        let big_f = big as f64;
        assert_eq!(
            IfaValue::Int(big).partial_cmp(&IfaValue::Float(big_f)),
            Some(Ordering::Equal),
            "lossy int→float round-trip still compares consistently"
        );
        // Float that is strictly larger than i64::MAX
        assert_eq!(
            IfaValue::Int(big).partial_cmp(&IfaValue::Float(f64::MAX)),
            Some(Ordering::Less)
        );
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

// ============================================================================
// 6. Trace Implementations
// ============================================================================

impl Trace for IfaValue {
    fn trace(&self, cb: crate::gc::TraceCallback) {
        match self {
            IfaValue::List(l) => {
                cb(l.ptr.cast());
            }
            IfaValue::Map(m) => {
                cb(m.ptr.cast());
            }
            #[cfg(feature = "vm")]
            IfaValue::Closure(c) => {
                cb(c.ptr.cast());
            }
            #[cfg(feature = "vm")]
            IfaValue::Upvalue(u) => {
                cb(u.ptr.cast());
            }
            _ => {}
        }
    }
}

impl Trace for Vec<IfaValue> {
    fn trace(&self, cb: crate::gc::TraceCallback) {
        for v in self {
            v.trace(cb);
        }
    }
}

impl Trace for HashMap<crate::CompactString, IfaValue> {
    fn trace(&self, cb: crate::gc::TraceCallback) {
        for v in self.values() {
            v.trace(cb);
        }
    }
}

#[cfg(feature = "vm")]
impl Trace for ClosureData {
    fn trace(&self, cb: crate::gc::TraceCallback) {
        for upvalue in self.env.iter() {
            cb(upvalue.ptr.cast());
        }
    }
}

#[cfg(feature = "vm")]
impl Trace for std::sync::Mutex<IfaValue> {
    fn trace(&self, cb: crate::gc::TraceCallback) {
        // Use lock() not try_lock(): silently skipping edges breaks the
        // Bacon-Rajan invariant. On the single-threaded VM no locks are held
        // during collection, so this always succeeds immediately.
        let guard = self.lock().unwrap_or_else(|e| e.into_inner());
        guard.trace(cb);
    }
}
