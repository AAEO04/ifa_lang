//! # Ikin - The Sacred Nuts (Constant Pool)
//!
//! Ikin implements the interned constant pool for the Ifá-Lang VM.
//! It replaces the previous "InstructionCache" (HashMap) with a flat, indexed storage.
//!
//! "The Ikin are immutable seeds of truth." - Cultural Metaphor for Constants.

use crate::value::IfaValue;
use std::collections::HashMap;
use std::sync::Arc;

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

    /// Cached Constants (Numbers, etc) - Reserved for future heavy constants
    #[allow(dead_code)]
    constants: Vec<IfaValue>,
}

impl Ikin {
    /// Create new empty Ikin
    pub fn new() -> Self {
        Ikin {
            strings: Vec::with_capacity(256),
            string_map: HashMap::with_capacity(256),
            constants: Vec::with_capacity(64),
        }
    }

    /// Intern a string (Turn it into a sacred nut)
    /// Returns the Unique ID (u32) for the string.
    pub fn intern(&mut self, s: &str) -> u32 {
        // 1. Check if already interned
        // We temporarily create Arc<str> to check map?
        // No, HashMap<Arc<str>> can query with &str if we use hash_brown/raw entry api,
        // but standard HashMap requires &Q where K: Borrow<Q>. Arc<str> borrows as &str!
        // So yes, we can query with &str.

        // However, the keys are `Arc<str>`.
        // Let's iterate? No, O(N).
        // Let's use the map.

        // Note: standard HashMap get with &str on Arc<str> keys works naturally.
        if let Some(&id) = self.string_map.get(s) {
            return id;
        }

        // 2. Intern new string
        let arc: Arc<str> = s.into();
        let id = self.strings.len() as u32;

        self.strings.push(arc.clone());
        self.string_map.insert(arc, id);

        id
    }

    /// Consult the nuts (Get constant by ID)
    #[inline(always)]
    pub fn consult_string(&self, idx: usize) -> Option<&Arc<str>> {
        self.strings.get(idx)
    }

    /// Load constants from Bytecode into the Sacred Nuts (Ikin)
    /// This converts costly Strings into cheap Arcs for O(1) runtime usage.
    pub fn load_from_bytecode(&mut self, bytecode: &crate::bytecode::Bytecode) {
        // Clear existing? Or append? For now, we assume a fresh load or append is fine.
        // We map the Bytecode string index to the Ikin ID.
        // Since bytecode uses index-based access, we must ensure order is preserved
        // OR we need a mapping table if we deduplicate.

        // Strategy:
        // Bytecode relies on index X being "String X".
        // If we deduplicate, "String X" and "String Y" might both point to Ikin ID Z.
        // But the VM instruction is `PushStr(Index)`.
        // So we need a mapping: BytecodeIndex -> IkinIndex.
        // Or, simpler for V1: Just mirror the vector (no dedup on load, just caching).
        // But Ikin is about dedup.

        // Actually, Bytecode `strings` vector IS the index.
        // OpCode says: PushStr(idx).
        // So `ikin.consult_string(idx)` must return the string at `bytecode.strings[idx]`.
        // So we MUST preserve 1:1 mapping with bytecode.strings indices.

        self.strings.clear();
        self.string_map.clear();

        self.strings.reserve(bytecode.strings.len());

        for s in &bytecode.strings {
            let arc: Arc<str> = s.as_str().into();
            self.strings.push(arc.clone());
            // Map is intentionally NOT populated for bulk loads to save time/memory.
            // The VM only accesses strings by index via `consult_string`.
            // The map is only needed if `intern` is called for new runtime strings.
            // self.string_map.insert(arc, (self.strings.len() - 1) as u32);
        }
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

        for (i, s) in data.strings.into_iter().enumerate() {
            let arc: std::sync::Arc<str> = s.into();
            ikin.strings.push(arc.clone());
            ikin.string_map.insert(arc, i as u32);
        }
        ikin.constants = data.constants;
        Ok(ikin)
    }
}
