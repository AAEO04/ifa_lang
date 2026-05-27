//! # IfaShared - Shared Value Type (The Village)
//!
//! The thread-safe container for Ifá-Lang's global state.
//! Used for inter-thread communication and the global registry.
//!
//! Key differences from IfaValue:
//! - Uses `Arc<DashMap>` for Objects (Concurrent)
//! - `Send + Sync` required
//! - No `RefCell` or `Rc`

use crate::IfaValue;
use crate::token::ResourceToken;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

#[cfg(feature = "dashmap")]
use dashmap::DashMap;
#[cfg(not(feature = "dashmap"))]
use std::sync::RwLock;

/// Shared Function: Thread-safe lambda
pub type IfaSharedFn = Arc<dyn Fn(Vec<IfaShared>) -> IfaShared + Send + Sync>;

/// The thread-safe value type
#[derive(Clone, Serialize, Deserialize)]
pub enum IfaShared {
    Int(i64),
    Float(f64),
    Str(Arc<str>),
    Bool(bool),
    /// List is value-type here (cloned on move)
    List(Vec<IfaShared>),
    /// Map is value-type here
    Map(HashMap<Arc<str>, IfaShared>),

    /// Object: The Village Data Store
    #[serde(skip)]
    #[cfg(feature = "dashmap")]
    Object(Arc<DashMap<Arc<str>, IfaShared>>),

    #[serde(skip)]
    #[cfg(not(feature = "dashmap"))]
    Object(Arc<RwLock<HashMap<Arc<str>, IfaShared>>>),

    #[serde(skip)]
    Fn(IfaSharedFn),

    Resource(ResourceToken),

    Null,
}

// Implement Thaw Logic
impl IfaShared {
    /// Thaw: Convert Shared Value (The Village) to Local Value (The Hut).
    /// Safe to use within a single thread.
    /// Strings are Zero-Copy (COW). Collections are Deep Copied.
    pub fn thaw(&self) -> IfaValue {
        match self {
            IfaShared::Int(n) => IfaValue::Int(*n),
            IfaShared::Float(n) => IfaValue::Float(*n),
            IfaShared::Str(s) => IfaValue::Str(crate::CompactString::new(s.as_ref())),
            IfaShared::Bool(b) => IfaValue::Bool(*b),
            IfaShared::Null => IfaValue::Null,
            IfaShared::Resource(token) => IfaValue::Resource(Arc::new(*token)),
            IfaShared::List(l) => {
                let mut thawed_list = Vec::with_capacity(l.len());
                for item in l {
                    thawed_list.push(item.thaw());
                }
                IfaValue::List(Arc::new(thawed_list))
            }
            IfaShared::Map(m) => {
                let mut thawed_map = HashMap::new();
                for (k, v) in m {
                    thawed_map.insert(crate::CompactString::new(k.as_ref()), v.thaw());
                }
                IfaValue::Map(Arc::new(thawed_map))
            }
            IfaShared::Object(o) => {
                let mut thawed_map = HashMap::new();
                #[cfg(feature = "dashmap")]
                {
                    for r in o.iter() {
                        thawed_map.insert(
                            crate::CompactString::new(r.key().as_ref()),
                            r.value().thaw(),
                        );
                    }
                }
                #[cfg(not(feature = "dashmap"))]
                {
                    if let Ok(guard) = o.read() {
                        for (k, v) in guard.iter() {
                            thawed_map.insert(crate::CompactString::new(k.as_ref()), v.thaw());
                        }
                    }
                }
                IfaValue::Map(Arc::new(thawed_map))
            }
            IfaShared::Fn(_) => IfaValue::Null, // Cannot thaw shared functions
        }
    }
}

// Implement Debug/Display
impl fmt::Debug for IfaShared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IfaShared::Int(v) => write!(f, "Int({})", v),
            IfaShared::Float(v) => write!(f, "Float({})", v),
            IfaShared::Str(v) => write!(f, "Str({:?})", v),
            IfaShared::Bool(v) => write!(f, "Bool({})", v),
            IfaShared::List(v) => write!(f, "List({:?})", v),
            IfaShared::Map(v) => write!(f, "Map({:?})", v),
            IfaShared::Object(v) => {
                #[cfg(feature = "dashmap")]
                {
                    write!(f, "SharedObject({:?})", v)
                }
                #[cfg(not(feature = "dashmap"))]
                {
                    write!(f, "SharedObject(<locked>)")
                }
            }
            IfaShared::Fn(_) => write!(f, "SharedFn"),
            IfaShared::Resource(t) => write!(f, "SharedResource({})", t.0),
            IfaShared::Null => write!(f, "Null"),
        }
    }
}

impl fmt::Display for IfaShared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IfaShared::Int(v) => write!(f, "{}", v),
            IfaShared::Float(v) => write!(f, "{}", v),
            IfaShared::Str(v) => write!(f, "{}", v),
            IfaShared::Bool(v) => write!(f, "{}", v),
            IfaShared::List(v) => write!(f, "{:?}", v),
            IfaShared::Map(v) => write!(f, "{:?}", v),
            IfaShared::Object(v) => {
                #[cfg(feature = "dashmap")]
                {
                    write!(f, "<SharedObject {}>", v.len())
                }
                #[cfg(not(feature = "dashmap"))]
                {
                    write!(f, "<SharedObject>")
                }
            }
            IfaShared::Fn(_) => write!(f, "<fn>"),
            IfaShared::Resource(t) => write!(f, "<resource {}>", t.0),
            IfaShared::Null => write!(f, "null"),
        }
    }
}
