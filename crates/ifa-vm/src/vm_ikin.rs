//! # Ikin - The Sacred Nuts (Constant Pool)
//!
//! Ikin implements the interned constant pool for the Ifá-Lang VM.
//! It replaces the previous "InstructionCache" (HashMap) with a flat, indexed storage.
//!
//! "The Ikin are immutable seeds of truth." - Cultural Metaphor for Constants.

use crate::bytecode::Bytecode;
use crate::error::{IfaError, IfaResult};
use crate::value::IfaValue;
use std::collections::HashMap;
use std::sync::Arc;

const MAX_INTERNED_STRINGS: usize = 65_536;

/// The Sacred Nuts - Immutable Constant Pool
///
/// Holds Strings, Numbers, and function references that do not change during execution.
/// Acts as a central repository for shared data to reduce duplication.
#[derive(Debug, Clone)]
pub struct Ikin {
    /// Interned Strings (Deduplicated)
    /// We use `Arc<str>` for O(1) cloning and shared ownership.
    /// The index in this vector is the "String ID".
    strings: Vec<Arc<str>>,

    /// Lookup map for deduplication (String -> ID)
    string_map: HashMap<Arc<str>, u32>,

    /// Precomputed lengths for interned strings (index matches string ID).
    string_lengths: Vec<usize>,

    /// Mapping from bytecode string index to local deduplicated string ID.
    bytecode_to_ikin: Vec<u32>,

    /// Cached Constants (Numbers, Arrays, Structs)
    /// Reserved for heavy constants that don't fit inline in bytecode.
    constants: Vec<IfaValue>,

    /// Pointer to length cache for fast string_len lookups
    ptr_to_len: HashMap<usize, usize>,
}

impl Ikin {
    /// Create new empty Ikin
    pub fn new() -> Self {
        Ikin {
            strings: Vec::with_capacity(256),
            string_map: HashMap::with_capacity(256),
            string_lengths: Vec::with_capacity(256),
            bytecode_to_ikin: Vec::new(),
            constants: Vec::with_capacity(64),
            ptr_to_len: HashMap::with_capacity(256),
        }
    }

    /// Intern a string (Turn it into a sacred nut)
    /// Returns the Unique ID (u32) for the string.
    pub fn intern(&mut self, s: &str) -> IfaResult<u32> {
        if let Some(&id) = self.string_map.get(s) {
            return Ok(id);
        }

        if self.strings.len() >= MAX_INTERNED_STRINGS {
            return Err(IfaError::Runtime(format!(
                "Ikin string pool exhausted (limit = {})",
                MAX_INTERNED_STRINGS
            )));
        }

        let arc: Arc<str> = s.into();
        let id = self.strings.len() as u32;

        self.strings.push(arc.clone());
        self.string_map.insert(arc.clone(), id);

        let len = ifa_types::value_union::IfaValue::unicode_string_len(s);
        self.string_lengths.push(len);
        self.ptr_to_len.insert(arc.as_ptr() as usize, len);

        Ok(id)
    }

    /// Consult the nuts (Get constant by ID)
    #[inline(always)]
    pub fn consult_string(&self, idx: usize) -> Option<&Arc<str>> {
        let actual_idx = self
            .bytecode_to_ikin
            .get(idx)
            .copied()
            .map(|i| i as usize)?;
        self.strings.get(actual_idx)
    }

    /// Return the cached Unicode scalar length for an interned string.
    ///
    /// The cache key is the allocation pointer for the `Arc<str>`, which is
    /// stable across clones of the same interned string.
    pub fn string_len(&self, s: &ifa_types::CompactString) -> usize {
        let s_str = s.as_str();
        if s_str.is_ascii() {
            return s_str.len();
        }

        match s {
            ifa_types::CompactString::Inline { char_len, .. } => *char_len as usize,
            ifa_types::CompactString::Heap(arc) => {
                let ptr = arc.as_ptr() as usize;
                if let Some(&len) = self.ptr_to_len.get(&ptr) {
                    len
                } else {
                    s_str.chars().count()
                }
            }
        }
    }

    /// Store a heavy constant (like a Struct, Array, or Large Integer).
    ///
    /// Currently, standard integers and floats fit inline inside the bytecode stream,
    /// so this is primarily reserved for future architectural expansion.
    ///
    /// Returns the unique ID for the constant.
    pub fn store_constant(&mut self, value: IfaValue) -> u32 {
        let id = self.constants.len() as u32;
        self.constants.push(value);
        id
    }

    /// Consult the Sacred Nuts for a heavy constant by its ID.
    #[inline(always)]
    pub fn consult_constant(&self, id: u32) -> Option<&IfaValue> {
        self.constants.get(id as usize)
    }

    /// Load constants from Bytecode into the Sacred Nuts (Ikin)
    /// This converts costly Strings into cheap Arcs for O(1) runtime usage.
    pub fn load_from_bytecode(&mut self, bytecode: &Bytecode) -> IfaResult<()> {
        self.populate_bytecode_mapping(bytecode)
    }

    /// Extract the current bytecode string index mapping, used for VM context switches.
    pub fn take_mapping(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.bytecode_to_ikin)
    }

    /// Restore a previously saved bytecode string index mapping.
    pub fn restore_mapping(&mut self, mapping: Vec<u32>) {
        self.bytecode_to_ikin = mapping;
    }

    /// Rebuild the translation mapping from bytecode strings to loaded constants.
    /// This is used on VM resume to preserve runtime interned strings while mapping bytecode.
    pub fn rebuild_bytecode_mapping(&mut self, bytecode: &Bytecode) -> IfaResult<()> {
        self.populate_bytecode_mapping(bytecode)
    }

    fn populate_bytecode_mapping(&mut self, bytecode: &Bytecode) -> IfaResult<()> {
        if self.strings.len() + bytecode.strings.len() > MAX_INTERNED_STRINGS {
            return Err(IfaError::Runtime(format!(
                "Bytecode string pool exceeds limit ({} + {} > {})",
                self.strings.len(),
                bytecode.strings.len(),
                MAX_INTERNED_STRINGS
            )));
        }

        self.bytecode_to_ikin.clear();
        self.bytecode_to_ikin.reserve(bytecode.strings.len());

        for s in &bytecode.strings {
            let id = if let Some(&existing_id) = self.string_map.get(s.as_str()) {
                existing_id
            } else {
                let arc: Arc<str> = s.as_str().into();
                let new_id = self.strings.len() as u32;
                self.string_map.insert(arc.clone(), new_id);
                self.strings.push(arc.clone());
                let len = ifa_types::value_union::IfaValue::unicode_string_len(s.as_str());
                self.string_lengths.push(len);
                self.ptr_to_len.insert(arc.as_ptr() as usize, len);
                new_id
            };
            self.bytecode_to_ikin.push(id);
        }

        Ok(())
    }
}

impl Default for Ikin {
    fn default() -> Self {
        Self::new()
    }
}

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize)]
struct IkinData {
    strings: Vec<String>,
    constants: Vec<IfaValue>,
}

impl Serialize for Ikin {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = IkinData {
            strings: self.strings.iter().map(|s| s.to_string()).collect(),
            constants: self.constants.clone(),
        };
        data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Ikin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = IkinData::deserialize(deserializer)?;
        let mut ikin = Ikin::new();
        ikin.strings.reserve(data.strings.len());
        ikin.string_lengths.clear();

        for (i, s) in data.strings.into_iter().enumerate() {
            let arc: std::sync::Arc<str> = s.as_str().into();
            ikin.strings.push(arc.clone());
            ikin.string_map.insert(arc, i as u32);
            ikin.string_lengths
                .push(ifa_types::value_union::IfaValue::unicode_string_len(
                    s.as_str(),
                ));
        }
        ikin.constants = data.constants;
        Ok(ikin)
    }
}

#[cfg(test)]
mod tests {
    use super::{Ikin, MAX_INTERNED_STRINGS};
    use crate::bytecode::Bytecode;

    #[test]
    fn load_from_bytecode_populates_string_map_for_runtime_dedup() {
        let mut bytecode = Bytecode::new("ikin_dedup");
        bytecode.strings = vec!["alpha".into(), "beta".into()];

        let mut ikin = Ikin::new();
        ikin.load_from_bytecode(&bytecode)
            .expect("load should succeed");

        let id = ikin
            .intern("beta")
            .expect("intern should reuse existing string");
        assert_eq!(id, 1);

        let gamma = ikin.intern("gamma").expect("new string should append");
        assert_eq!(gamma, 2);
    }

    #[test]
    fn load_from_bytecode_rejects_oversized_string_pool() {
        let mut bytecode = Bytecode::new("ikin_limit");
        bytecode.strings = vec![String::new(); MAX_INTERNED_STRINGS + 1];

        let err = Ikin::new()
            .load_from_bytecode(&bytecode)
            .expect_err("oversized pool should fail");
        assert!(err.to_string().contains("string pool exceeds limit"));
    }

    #[test]
    fn string_lengths_lookup() {
        let mut ikin = Ikin::new();
        let s = "e\u{301}".repeat(16); // 48 bytes, heap allocated
        ikin.intern(&s).unwrap();

        let value = ifa_types::CompactString::new(&s);

        assert_eq!(ikin.string_len(&value), 32);
        assert_eq!(ikin.string_lengths.len(), 1);
    }
}
