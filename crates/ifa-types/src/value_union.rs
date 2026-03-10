//! # Unified Type System (Clean Enum Implementation)
//!
//! This module implements `IfaValue` as a safe, reference-counted enum.
//! No manual memory management. No unsafe unions. pure Rust.

use std::sync::Arc;
use std::collections::HashMap;
use std::fmt;
use serde::{Deserialize, Serialize};

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
    // Using Arc for thread-safety (Send + Sync generally, if T is Send+Sync)
    Str(Arc<str>),
    List(Arc<Vec<IfaValue>>),
    Map(Arc<HashMap<Arc<str>, IfaValue>>), // Keys are interned strings (Arc<str>)
    
    // 3. Special / VM Objects
    Fn(Arc<BytecodeFnData>),
    
    // Legacy / Other
    #[allow(dead_code)]
    Resource(Arc<ResourceToken>),
    
    // VM Specific (Keeping simplified for now)
    #[cfg(feature = "vm")]
    Class(Arc<String>), // Placeholder
    #[cfg(feature = "vm")]
    Return(Arc<IfaValue>), // For returning from functions
    
    // 4. Okanran (Error Handling)
    // flag: true=Ok, false=Err. Payload boxed to reduce enum size variants.
    Result(bool, Box<IfaValue>),
} 

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
    pub fn bytecode_fn(name: impl Into<String>, start_ip: usize, arity: u8) -> Self {
        IfaValue::Fn(Arc::new(BytecodeFnData {
            name: name.into(),
            start_ip,
            arity
        }))
    }
    
    #[cfg(feature = "vm")]
    pub fn return_value(val: IfaValue) -> Self {
        IfaValue::Return(Arc::new(val))
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

    // --- Accessors ---



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
            IfaValue::Result(_, _) => "Result",
            _ => "Unknown",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            IfaValue::Null => false,
            IfaValue::Bool(b) => *b,
            IfaValue::Int(i) => *i != 0,
            IfaValue::Float(f) => *f != 0.0,
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

            // Arc equality checks pointer equality first usually.
            // But we derived PartialEq on IfaValue? No, we implemented manual Eq below.
            // For recursive equality:
            (IfaValue::List(a), IfaValue::List(b)) => {
                if Arc::ptr_eq(a, b) { return true; }
                if a.len() != b.len() { return false; }
                a.iter().zip(b.iter()).all(|(x, y)| x.is_equal(y))
            }
             (IfaValue::Map(a), IfaValue::Map(b)) => {
                if Arc::ptr_eq(a, b) { return true; }
                if a.len() != b.len() { return false; }
                a.iter().all(|(k, v)| b.get(k).map_or(false, |bv| v.is_equal(bv)))
            }
            _ => false
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
            IfaValue::Result(ok, val) => {
                if *ok {
                   write!(f, "Ok({})", val)
                } else {
                   write!(f, "Err({})", val)
                }
            }
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

#[cfg(feature = "serde")]
impl Serialize for IfaValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            IfaValue::Null => serializer.serialize_unit(),
            IfaValue::Bool(b) => serializer.serialize_bool(*b),
            IfaValue::Int(i) => serializer.serialize_i64(*i),
            IfaValue::Float(f) => serializer.serialize_f64(*f),
            IfaValue::Str(s) => serializer.serialize_str(s),
            IfaValue::List(l) => {
                 use serde::ser::SerializeSeq;
                 let mut seq = serializer.serialize_seq(Some(l.len()))?;
                 for item in l.iter() {
                     seq.serialize_element(item)?;
                 }
                 seq.end()
            }
            // ... Map ...
             _ => serializer.serialize_unit(),
        }
    }
}



#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for IfaValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, SeqAccess, Visitor};
        use std::fmt;

        struct IfaValueVisitor;

        impl<'de> Visitor<'de> for IfaValueVisitor {
            type Value = IfaValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("null, bool, integer, float, string, list, or map")
            }

            fn visit_unit<E: de::Error>(self) -> Result<IfaValue, E> {
                Ok(IfaValue::Null)
            }

            fn visit_none<E: de::Error>(self) -> Result<IfaValue, E> {
                Ok(IfaValue::Null)
            }

            fn visit_bool<E: de::Error>(self, v: bool) -> Result<IfaValue, E> {
                Ok(IfaValue::Bool(v))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<IfaValue, E> {
                Ok(IfaValue::Int(v))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<IfaValue, E> {
                // Saturate on overflow rather than error
                Ok(IfaValue::Int(v.min(i64::MAX as u64) as i64))
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<IfaValue, E> {
                Ok(IfaValue::Float(v))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<IfaValue, E> {
                Ok(IfaValue::str(v))
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<IfaValue, E> {
                Ok(IfaValue::str(v))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<IfaValue, A::Error> {
                let mut items = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(item) = seq.next_element::<IfaValue>()? {
                    items.push(item);
                }
                Ok(IfaValue::list(items))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<IfaValue, A::Error> {
                let mut m = HashMap::with_capacity(map.size_hint().unwrap_or(0));
                while let Some((key, value)) = map.next_entry::<String, IfaValue>()? {
                    m.insert(key, value);
                }
                Ok(IfaValue::map(m))
            }
        }

        deserializer.deserialize_any(IfaValueVisitor)
    }
}



// Ref Definitions

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytecodeFnData {
    pub name: String,
    pub start_ip: usize,
    pub arity: u8,
}


