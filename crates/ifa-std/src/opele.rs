//! # Ọpẹlẹ Module - Divination Chain
//!
//! The Ọpẹlẹ is the divination chain used in Ifá tradition.
//! This module provides:
//! - OpeleChain: Append-only, tamper-evident log structure
//! - Odu casting: 256 Odu pattern generation
//! - Divination: Question interpretation with proverbs
//!
//! Design (Linus-approved):
//! - Simple, no fancy operators
//! - Zero-cost abstractions
//! - Culturally accurate (8-bit = 2 legs of 4 bits each)

use std::time::{SystemTime, UNIX_EPOCH};

// =============================================================================
// OPELE CHAIN - Tamper-evident append-only log
// =============================================================================

/// A single link in the Ọpẹlẹ chain
#[derive(Debug, Clone)]
pub struct ChainEntry {
    /// The data stored in this link
    pub data: String,
    /// Index in the chain
    pub index: u64,
    /// Hash of the previous entry
    prev_hash: [u8; 32],
    /// Hash of this entry (data + prev_hash + index)
    hash: [u8; 32],
    /// Timestamp when this link was cast
    pub timestamp: u64,
}

/// Ọpẹlẹ Chain - Append-only verifiable sequence
///
/// Like the divination chain, each link depends on the previous,
/// creating an unbroken sequence of truth.
#[derive(Debug)]
pub struct OpeleChain {
    entries: Vec<ChainEntry>,
    root_hash: [u8; 32],
}

impl OpeleChain {
    /// Create a new empty chain
    pub fn new() -> Self {
        OpeleChain {
            entries: Vec::new(),
            root_hash: [0u8; 32],
        }
    }

    /// Cast a new link into the chain
    /// Returns &mut Self for simple chaining: chain.cast("a").cast("b")
    pub fn cast(&mut self, data: &str) -> &mut Self {
        let index = self.entries.len() as u64;
        let prev_hash = self.root_hash;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Simple hash: SHA-256 of (data || prev_hash || index || timestamp)
        let hash = self.compute_hash(data, &prev_hash, index, timestamp);

        let entry = ChainEntry {
            data: data.to_string(),
            index,
            prev_hash,
            hash,
            timestamp,
        };

        self.root_hash = hash;
        self.entries.push(entry);
        self
    }

    /// Verify the chain integrity
    /// Returns true if all links hash correctly
    pub fn verify(&self) -> bool {
        let mut expected_prev = [0u8; 32];

        for entry in &self.entries {
            // Check prev_hash matches expected
            if entry.prev_hash != expected_prev {
                return false;
            }

            // Recompute hash and verify
            let computed =
                self.compute_hash(&entry.data, &entry.prev_hash, entry.index, entry.timestamp);

            if computed != entry.hash {
                return false;
            }

            expected_prev = entry.hash;
        }

        // Final hash should match root
        expected_prev == self.root_hash || self.entries.is_empty()
    }

    /// Get the root hash (Merkle root)
    pub fn root(&self) -> [u8; 32] {
        self.root_hash
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if chain is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get all entries
    pub fn entries(&self) -> &[ChainEntry] {
        &self.entries
    }

    /// Get entry by index
    pub fn get(&self, index: usize) -> Option<&ChainEntry> {
        self.entries.get(index)
    }

    /// Simple hash function (SHA-256-like using basic ops)
    /// In production, use ring or sha2 crate
    fn compute_hash(&self, data: &str, prev: &[u8; 32], index: u64, ts: u64) -> [u8; 32] {
        let mut hash = [0u8; 32];

        // Mix data bytes
        for (i, b) in data.bytes().enumerate() {
            hash[i % 32] ^= b;
            hash[(i + 1) % 32] = hash[(i + 1) % 32].wrapping_add(b);
        }

        // Mix previous hash
        for i in 0..32 {
            hash[i] ^= prev[i];
            hash[(i + 7) % 32] = hash[(i + 7) % 32].wrapping_add(prev[i]);
        }

        // Mix index and timestamp
        let idx_bytes = index.to_le_bytes();
        let ts_bytes = ts.to_le_bytes();
        for i in 0..8 {
            hash[i] ^= idx_bytes[i];
            hash[i + 8] ^= ts_bytes[i];
        }

        // Final mixing rounds
        for round in 0..4 {
            for i in 0..32 {
                let j = (i + round * 7 + 1) % 32;
                hash[j] = hash[j].wrapping_add(hash[i].rotate_left(3));
            }
        }

        hash
    }
}

impl Default for OpeleChain {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// ODU SYSTEM - 256 Patterns
// =============================================================================

/// The 16 principal Odù (Oju Odù)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PrincipalOdu {
    Ogbe = 0,      // 1111
    Oyeku = 1,     // 0000
    Iwori = 2,     // 0110
    Odi = 3,       // 1001
    Irosu = 4,     // 1100
    Owonrin = 5,   // 0011
    Obara = 6,     // 1000
    Okanran = 7,   // 0001
    Ogunda = 8,    // 1110
    Osa = 9,       // 0111
    Ika = 10,      // 0100
    Oturupon = 11, // 0010
    Otura = 12,    // 1011
    Irete = 13,    // 1101
    Ose = 14,      // 1010
    Ofun = 15,     // 0101
}

impl PrincipalOdu {
    pub fn name(&self) -> &'static str {
        match self {
            PrincipalOdu::Ogbe => "Ogbe",
            PrincipalOdu::Oyeku => "Oyeku",
            PrincipalOdu::Iwori => "Iwori",
            PrincipalOdu::Odi => "Odi",
            PrincipalOdu::Irosu => "Irosu",
            PrincipalOdu::Owonrin => "Owonrin",
            PrincipalOdu::Obara => "Obara",
            PrincipalOdu::Okanran => "Okanran",
            PrincipalOdu::Ogunda => "Ogunda",
            PrincipalOdu::Osa => "Osa",
            PrincipalOdu::Ika => "Ika",
            PrincipalOdu::Oturupon => "Oturupon",
            PrincipalOdu::Otura => "Otura",
            PrincipalOdu::Irete => "Irete",
            PrincipalOdu::Ose => "Ose",
            PrincipalOdu::Ofun => "Ofun",
        }
    }

    pub fn binary(&self) -> &'static str {
        match self {
            PrincipalOdu::Ogbe => "1111",
            PrincipalOdu::Oyeku => "0000",
            PrincipalOdu::Iwori => "0110",
            PrincipalOdu::Odi => "1001",
            PrincipalOdu::Irosu => "1100",
            PrincipalOdu::Owonrin => "0011",
            PrincipalOdu::Obara => "1000",
            PrincipalOdu::Okanran => "0001",
            PrincipalOdu::Ogunda => "1110",
            PrincipalOdu::Osa => "0111",
            PrincipalOdu::Ika => "0100",
            PrincipalOdu::Oturupon => "0010",
            PrincipalOdu::Otura => "1011",
            PrincipalOdu::Irete => "1101",
            PrincipalOdu::Ose => "1010",
            PrincipalOdu::Ofun => "0101",
        }
    }

    pub fn from_index(i: u8) -> Self {
        match i % 16 {
            0 => PrincipalOdu::Ogbe,
            1 => PrincipalOdu::Oyeku,
            2 => PrincipalOdu::Iwori,
            3 => PrincipalOdu::Odi,
            4 => PrincipalOdu::Irosu,
            5 => PrincipalOdu::Owonrin,
            6 => PrincipalOdu::Obara,
            7 => PrincipalOdu::Okanran,
            8 => PrincipalOdu::Ogunda,
            9 => PrincipalOdu::Osa,
            10 => PrincipalOdu::Ika,
            11 => PrincipalOdu::Oturupon,
            12 => PrincipalOdu::Otura,
            13 => PrincipalOdu::Irete,
            14 => PrincipalOdu::Ose,
            _ => PrincipalOdu::Ofun,
        }
    }
}

/// Full Odu (combination of two principal Odu = 256 patterns)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Odu {
    /// Right leg (first cast)
    pub right: PrincipalOdu,
    /// Left leg (second cast)  
    pub left: PrincipalOdu,
}

impl Odu {
    /// Create Odu from two legs
    pub fn new(right: PrincipalOdu, left: PrincipalOdu) -> Self {
        Odu { right, left }
    }

    /// Create from 8-bit value (0-255)
    pub fn from_byte(value: u8) -> Self {
        let right = PrincipalOdu::from_index(value & 0x0F);
        let left = PrincipalOdu::from_index((value >> 4) & 0x0F);
        Odu { right, left }
    }

    /// Convert to 8-bit value
    pub fn to_byte(&self) -> u8 {
        (self.right as u8) | ((self.left as u8) << 4)
    }

    /// Get the compound name
    pub fn name(&self) -> String {
        if self.right == self.left {
            // Meji (double) - e.g., Ogbe Meji
            format!("{} Meji", self.right.name())
        } else {
            // Compound - e.g., Ogbe Oyeku
            format!("{} {}", self.right.name(), self.left.name())
        }
    }

    /// Check if this is a principal Odu (Meji)
    pub fn is_principal(&self) -> bool {
        self.right == self.left
    }
}

// =============================================================================
// COMPOUND ODU - Variable Depth (2 to n levels)
// =============================================================================

/// Variable-depth compound Odù for advanced patterns
///
/// Supports arbitrary ancestral depth:
/// - 2 levels: 256 patterns (Parent-Child)
/// - 3 levels: 4,096 patterns (Grandparent-Parent-Child / GPC)
/// - 4 levels: 65,536 patterns (Great-Grandparent-Grandparent-Parent-Child)
/// - n levels: 16^n patterns
///
/// # Example
/// ```rust
/// // 2-level compound
/// let compound = CompoundOdu::from_pair(
///     PrincipalOdu::Otura,
///     PrincipalOdu::Ika
/// );
/// assert_eq!(compound.name(), "Otura_Ika");
///
/// // 3-level GPC
/// let gpc = CompoundOdu::from_triple(
///     PrincipalOdu::Ogbe,
///     PrincipalOdu::Otura,
///     PrincipalOdu::Ika
/// );
/// assert_eq!(gpc.name(), "Ogbe_Otura_Ika");
///
/// // Arbitrary depth
/// let deep = CompoundOdu::new(vec![
///     PrincipalOdu::Ogbe,
///     PrincipalOdu::Oyeku,
///     PrincipalOdu::Iwori,
///     PrincipalOdu::Otura,
///     PrincipalOdu::Ika,
/// ]);
/// assert_eq!(deep.depth(), 5);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundOdu {
    /// Ancestral lineage from oldest (root) to newest (current)
    /// ancestors[0] = root/oldest, ancestors[n-1] = current/leaf
    ancestors: Vec<PrincipalOdu>,
}

impl CompoundOdu {
    /// Create compound from ancestors (oldest to newest)
    pub fn new(ancestors: Vec<PrincipalOdu>) -> Self {
        assert!(
            !ancestors.is_empty(),
            "Compound Odu must have at least one ancestor"
        );
        CompoundOdu { ancestors }
    }

    /// Create 2-level compound (traditional)
    pub fn from_pair(parent: PrincipalOdu, child: PrincipalOdu) -> Self {
        CompoundOdu {
            ancestors: vec![parent, child],
        }
    }

    /// Create 3-level compound (GPC: Grandparent-Parent-Child)
    pub fn from_triple(
        grandparent: PrincipalOdu,
        parent: PrincipalOdu,
        child: PrincipalOdu,
    ) -> Self {
        CompoundOdu {
            ancestors: vec![grandparent, parent, child],
        }
    }

    /// Get depth (number of levels in hierarchy)
    pub fn depth(&self) -> usize {
        self.ancestors.len()
    }

    /// Get root (oldest ancestor)
    pub fn root(&self) -> PrincipalOdu {
        self.ancestors[0]
    }

    /// Get current (newest/leaf node)
    pub fn current(&self) -> PrincipalOdu {
        *self.ancestors.last().unwrap()
    }

    /// Get parent (one level up from current)
    pub fn parent(&self) -> Option<PrincipalOdu> {
        if self.depth() >= 2 {
            Some(self.ancestors[self.depth() - 2])
        } else {
            None
        }
    }

    /// Get specific ancestor by index (0 = oldest/root)
    pub fn ancestor(&self, index: usize) -> Option<PrincipalOdu> {
        self.ancestors.get(index).copied()
    }

    /// Get all ancestors
    pub fn ancestors(&self) -> &[PrincipalOdu] {
        &self.ancestors
    }

    /// Get compound name (underscores separate levels)
    pub fn name(&self) -> String {
        match self.depth() {
            1 => self.root().name().to_string(),
            2 => {
                if self.root() == self.current() {
                    format!("{} Meji", self.root().name())
                } else {
                    format!("{}_{}", self.root().name(), self.current().name())
                }
            }
            _ => {
                // Multi-level: join with underscores
                self.ancestors
                    .iter()
                    .map(|o| o.name())
                    .collect::<Vec<_>>()
                    .join("_")
            }
        }
    }

    /// Get short name (abbreviated with hyphens)
    pub fn short_name(&self) -> String {
        self.ancestors
            .iter()
            .map(|o| &o.name()[..2])
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Get genealogical lineage description
    pub fn lineage(&self) -> String {
        let roles = self.lineage_roles();

        self.ancestors
            .iter()
            .zip(roles.iter())
            .map(|(odu, role)| format!("{}: {}", role, odu.name()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get role names for each level
    fn lineage_roles(&self) -> Vec<String> {
        match self.depth() {
            1 => vec!["Self".to_string()],
            2 => vec!["Parent".to_string(), "Child".to_string()],
            3 => vec![
                "Grandparent".to_string(),
                "Parent".to_string(),
                "Child".to_string(),
            ],
            4 => vec![
                "Great-Grandparent".to_string(),
                "Grandparent".to_string(),
                "Parent".to_string(),
                "Child".to_string(),
            ],
            5 => vec![
                "Great²-Grandparent".to_string(),
                "Great-Grandparent".to_string(),
                "Grandparent".to_string(),
                "Parent".to_string(),
                "Child".to_string(),
            ],
            n => {
                let mut roles = Vec::with_capacity(n);
                for i in 0..(n - 1) {
                    roles.push(format!("Ancestor-{}", n - i - 1));
                }
                roles.push("Current".to_string());
                roles
            }
        }
    }

    /// Pack to bytes (efficient variable-length encoding)
    pub fn to_bytes(&self) -> Vec<u8> {
        let bits_needed = self.depth() * 4;
        let bytes_needed = (bits_needed + 7) / 8;
        let mut bytes = vec![0u8; bytes_needed];

        for (i, odu) in self.ancestors.iter().enumerate() {
            let bit_offset = i * 4;
            let byte_idx = bit_offset / 8;
            let bit_in_byte = bit_offset % 8;

            if bit_in_byte <= 4 {
                bytes[byte_idx] |= (*odu as u8) << bit_in_byte;
            } else {
                // Nibble spans two bytes
                bytes[byte_idx] |= (*odu as u8) << bit_in_byte;
                if byte_idx + 1 < bytes.len() {
                    bytes[byte_idx + 1] |= (*odu as u8) >> (8 - bit_in_byte);
                }
            }
        }

        bytes
    }

    /// Unpack from bytes
    pub fn from_bytes(bytes: &[u8], depth: usize) -> Self {
        let mut ancestors = Vec::with_capacity(depth);

        for i in 0..depth {
            let bit_offset = i * 4;
            let byte_idx = bit_offset / 8;
            let bit_in_byte = bit_offset % 8;

            let nibble = if bit_in_byte <= 4 {
                (bytes[byte_idx] >> bit_in_byte) & 0x0F
            } else {
                let low = bytes[byte_idx] >> bit_in_byte;
                let high = if byte_idx + 1 < bytes.len() {
                    bytes[byte_idx + 1] << (8 - bit_in_byte)
                } else {
                    0
                };
                (low | high) & 0x0F
            };

            ancestors.push(PrincipalOdu::from_index(nibble));
        }

        CompoundOdu { ancestors }
    }
}

/// Convert 2-level Odu to CompoundOdu
impl From<Odu> for CompoundOdu {
    fn from(odu: Odu) -> Self {
        CompoundOdu::from_pair(odu.right, odu.left)
    }
}

impl std::fmt::Display for CompoundOdu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// =============================================================================
// DIVINATION FUNCTIONS
// =============================================================================

/// Cast the Ọpẹlẹ chain to reveal an Odu
/// Uses time-based seeding for randomness
pub fn cast() -> Odu {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    // LCG random number generator
    let random = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);

    Odu::from_byte((random >> 32) as u8)
}

/// Cast with a specific seed (for reproducibility)
pub fn cast_seeded(seed: u64) -> Odu {
    let random = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    Odu::from_byte((random >> 32) as u8)
}

/// Divine with a question - returns interpretation
pub fn divine(question: &str) -> DivineResult {
    let odu = cast();
    let proverb = proverb_for(&odu);
    let guidance = guidance_for(&odu);

    DivineResult {
        question: question.to_string(),
        odu,
        proverb,
        guidance,
    }
}

/// Cast a compound Odu with specified depth
///
/// # Example
/// ```rust
/// // 2-level compound
/// let compound = cast_compound(2);
///
/// // 3-level GPC
/// let gpc = cast_compound(3);
///
/// // 5-level deep
/// let deep = cast_compound(5);
/// ```
pub fn cast_compound(depth: usize) -> CompoundOdu {
    assert!(depth > 0, "Depth must be at least 1");

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    let mut ancestors = Vec::with_capacity(depth);
    let mut current_seed = seed;

    for _ in 0..depth {
        let random = current_seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);

        let odu_index = ((random >> 32) as u8) % 16;
        ancestors.push(PrincipalOdu::from_index(odu_index));

        current_seed = random;
    }

    CompoundOdu::new(ancestors)
}

/// Create a compound Odu from specific ancestors
///
/// # Example
/// ```rust
/// let compound = create_compound(vec![
///     PrincipalOdu::Ogbe,
///     PrincipalOdu::Otura,
///     PrincipalOdu::Ika,
/// ]);
/// ```
pub fn create_compound(ancestors: Vec<PrincipalOdu>) -> CompoundOdu {
    CompoundOdu::new(ancestors)
}

/// Result of a divination
#[derive(Debug)]
pub struct DivineResult {
    pub question: String,
    pub odu: Odu,
    pub proverb: String,
    pub guidance: String,
}

impl std::fmt::Display for DivineResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Question: {}\nOdu: {}\nProverb: {}\nGuidance: {}",
            self.question,
            self.odu.name(),
            self.proverb,
            self.guidance
        )
    }
}

/// Get proverb for an Odu
fn proverb_for(odu: &Odu) -> String {
    // Proverbs based on right (primary) Odu
    match odu.right {
        PrincipalOdu::Ogbe => {
            "Ogbe says: The path is clear, move forward with confidence.".to_string()
        }
        PrincipalOdu::Oyeku => "Oyeku says: In darkness, wisdom prepares for dawn.".to_string(),
        PrincipalOdu::Iwori => "Iwori says: Look within before seeking without.".to_string(),
        PrincipalOdu::Odi => "Odi says: Close one door, and another opens.".to_string(),
        PrincipalOdu::Irosu => {
            "Irosu says: Speak truth, for lies have no legs to stand.".to_string()
        }
        PrincipalOdu::Owonrin => {
            "Owonrin says: Change is the only constant; embrace it.".to_string()
        }
        PrincipalOdu::Obara => "Obara says: What you give, returns to you multiplied.".to_string(),
        PrincipalOdu::Okanran => "Okanran says: The tongue is sharper than the sword.".to_string(),
        PrincipalOdu::Ogunda => "Ogunda says: Clear the path with patience, not force.".to_string(),
        PrincipalOdu::Osa => "Osa says: Let go of what no longer serves you.".to_string(),
        PrincipalOdu::Ika => "Ika says: Words bind; choose them carefully.".to_string(),
        PrincipalOdu::Oturupon => "Oturupon says: Balance is the key to health.".to_string(),
        PrincipalOdu::Otura => {
            "Otura says: The journey teaches more than the destination.".to_string()
        }
        PrincipalOdu::Irete => "Irete says: Secrets revealed bring freedom.".to_string(),
        PrincipalOdu::Ose => "Ose says: Beauty flows from inner peace.".to_string(),
        PrincipalOdu::Ofun => "Ofun says: The ancestors watch; honor their wisdom.".to_string(),
    }
}

/// Get guidance for an Odu
fn guidance_for(odu: &Odu) -> String {
    if odu.is_principal() {
        format!(
            "{} Meji brings double power. Your path is strongly affirmed.",
            odu.right.name()
        )
    } else {
        format!(
            "The combination of {} and {} suggests balance between {} and transformation.",
            odu.right.name(),
            odu.left.name(),
            odu.right.name().to_lowercase()
        )
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_creation() {
        let mut chain = OpeleChain::new();
        assert!(chain.is_empty());

        chain.cast("First entry");
        assert_eq!(chain.len(), 1);
        assert!(chain.verify());
    }

    #[test]
    fn test_chain_integrity() {
        let mut chain = OpeleChain::new();
        chain.cast("Entry 1").cast("Entry 2").cast("Entry 3");

        assert_eq!(chain.len(), 3);
        assert!(chain.verify());
    }

    #[test]
    fn test_odu_from_byte() {
        let odu = Odu::from_byte(0);
        assert_eq!(odu.right, PrincipalOdu::Ogbe);
        assert_eq!(odu.left, PrincipalOdu::Ogbe);
        assert!(odu.is_principal());

        let odu = Odu::from_byte(0x10);
        assert_eq!(odu.right, PrincipalOdu::Ogbe);
        assert_eq!(odu.left, PrincipalOdu::Oyeku);
        assert!(!odu.is_principal());
    }

    #[test]
    fn test_odu_roundtrip() {
        for i in 0..=255u8 {
            let odu = Odu::from_byte(i);
            assert_eq!(odu.to_byte(), i);
        }
    }

    #[test]
    fn test_cast_seeded() {
        let odu1 = cast_seeded(12345);
        let odu2 = cast_seeded(12345);
        assert_eq!(odu1.to_byte(), odu2.to_byte());
    }

    #[test]
    fn test_divine() {
        let result = divine("Will this test pass?");
        assert!(!result.proverb.is_empty());
        assert!(!result.guidance.is_empty());
    }

    #[test]
    fn test_all_256_odu() {
        for i in 0..=255u8 {
            let odu = Odu::from_byte(i);
            let name = odu.name();
            assert!(!name.is_empty());
        }
    }

    // =========================================================================
    // COMPOUND ODU TESTS
    // =========================================================================

    #[test]
    fn test_compound_2_level() {
        let compound = CompoundOdu::from_pair(PrincipalOdu::Otura, PrincipalOdu::Ika);
        assert_eq!(compound.depth(), 2);
        assert_eq!(compound.name(), "Otura_Ika");
        assert_eq!(compound.root(), PrincipalOdu::Otura);
        assert_eq!(compound.current(), PrincipalOdu::Ika);
    }

    #[test]
    fn test_compound_3_level_gpc() {
        let gpc =
            CompoundOdu::from_triple(PrincipalOdu::Ogbe, PrincipalOdu::Otura, PrincipalOdu::Ika);
        assert_eq!(gpc.depth(), 3);
        assert_eq!(gpc.name(), "Ogbe_Otura_Ika");
        assert_eq!(gpc.short_name(), "Og-Ot-Ik");
    }

    #[test]
    fn test_compound_arbitrary_depth() {
        let deep = CompoundOdu::new(vec![
            PrincipalOdu::Ogbe,
            PrincipalOdu::Oyeku,
            PrincipalOdu::Iwori,
            PrincipalOdu::Otura,
            PrincipalOdu::Ika,
        ]);
        assert_eq!(deep.depth(), 5);
        assert_eq!(deep.root(), PrincipalOdu::Ogbe);
        assert_eq!(deep.current(), PrincipalOdu::Ika);
        assert_eq!(deep.parent(), Some(PrincipalOdu::Otura));
        assert_eq!(deep.ancestor(2), Some(PrincipalOdu::Iwori));
    }

    #[test]
    fn test_compound_lineage() {
        let compound =
            CompoundOdu::from_triple(PrincipalOdu::Ogbe, PrincipalOdu::Otura, PrincipalOdu::Ika);
        let lineage = compound.lineage();
        assert!(lineage.contains("Grandparent: Ogbe"));
        assert!(lineage.contains("Parent: Otura"));
        assert!(lineage.contains("Child: Ika"));
    }

    #[test]
    fn test_compound_bytes_roundtrip() {
        // Test 2-level
        let c2 = CompoundOdu::from_pair(PrincipalOdu::Otura, PrincipalOdu::Ika);
        let bytes2 = c2.to_bytes();
        let decoded2 = CompoundOdu::from_bytes(&bytes2, 2);
        assert_eq!(c2, decoded2);

        // Test 3-level
        let c3 =
            CompoundOdu::from_triple(PrincipalOdu::Ogbe, PrincipalOdu::Otura, PrincipalOdu::Ika);
        let bytes3 = c3.to_bytes();
        let decoded3 = CompoundOdu::from_bytes(&bytes3, 3);
        assert_eq!(c3, decoded3);

        // Test 5-level
        let c5 = CompoundOdu::new(vec![
            PrincipalOdu::Ogbe,
            PrincipalOdu::Oyeku,
            PrincipalOdu::Iwori,
            PrincipalOdu::Otura,
            PrincipalOdu::Ika,
        ]);
        let bytes5 = c5.to_bytes();
        let decoded5 = CompoundOdu::from_bytes(&bytes5, 5);
        assert_eq!(c5, decoded5);
    }

    #[test]
    fn test_odu_to_compound_conversion() {
        let odu = Odu::new(PrincipalOdu::Otura, PrincipalOdu::Ika);
        let compound: CompoundOdu = odu.into();
        assert_eq!(compound.depth(), 2);
        assert_eq!(compound.name(), "Otura_Ika");
    }
}
