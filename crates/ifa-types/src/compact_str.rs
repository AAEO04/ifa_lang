use std::fmt;
use std::ops::Deref;
use std::sync::Arc;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use serde::de::{self, Visitor};

#[derive(Clone)]
pub enum CompactString {
    Inline {
        len: u8,
        char_len: u8,
        data: [u8; 21],
    },
    Heap(Arc<str>),
}

impl CompactString {
    pub fn new(s: &str) -> Self {
        let bytes = s.as_bytes();
        let len = bytes.len();
        if len <= 21 {
            let mut data = [0u8; 21];
            // Safe copy without unsafe block
            data[..len].copy_from_slice(bytes);
            let char_len = s.chars().count();
            CompactString::Inline {
                len: len as u8,
                char_len: char_len as u8,
                data,
            }
        } else {
            CompactString::Heap(Arc::from(s))
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            CompactString::Inline { len, char_len: _, data } => {
                // Safe conversion, guaranteed valid UTF-8 by constructors
                std::str::from_utf8(&data[..(*len as usize)]).expect("CompactString contained invalid UTF-8")
            }
            CompactString::Heap(arc) => arc,
        }
    }

    /// Returns the character length of the CompactString, using precomputed char_len for Inline.
    pub fn char_len(&self) -> usize {
        match self {
            CompactString::Inline { char_len, .. } => *char_len as usize,
            CompactString::Heap(arc) => {
                if arc.is_ascii() {
                    arc.len()
                } else {
                    arc.chars().count()
                }
            }
        }
    }
}

impl Deref for CompactString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for CompactString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&str> for CompactString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for CompactString {
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

impl From<Arc<str>> for CompactString {
    fn from(arc: Arc<str>) -> Self {
        if arc.len() <= 21 {
            Self::new(&arc)
        } else {
            CompactString::Heap(arc)
        }
    }
}

impl fmt::Display for CompactString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for CompactString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl PartialEq for CompactString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for CompactString {}

impl PartialOrd for CompactString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CompactString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl std::hash::Hash for CompactString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl Serialize for CompactString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

struct CompactStringVisitor;

impl<'de> Visitor<'de> for CompactStringVisitor {
    type Value = CompactString;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(CompactString::new(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(CompactString::new(&v))
    }
}

impl<'de> Deserialize<'de> for CompactString {
    fn deserialize<D>(deserializer: D) -> Result<CompactString, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(CompactStringVisitor)
    }
}

impl std::borrow::Borrow<str> for CompactString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}
