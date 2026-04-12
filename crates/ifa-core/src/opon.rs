//! # Opon - Memory Management (Calabash)
//!
//! The Opon is the sacred calabash - the memory container for the Ifá-Lang VM.
//! It provides:
//! - Memory slots for variables
//! - Flight recorder for debugging
//! - Configurable sizing for embedded vs full systems
//! - **Memory limit enforcement** - prevents exceeding configured limits

use crate::value::IfaValue;
use serde::{Deserialize, Serialize};
use std::fmt;

const AILOPIN_HARD_LIMIT: usize = 1 << 20;

/// Error when memory limit is exceeded
#[derive(Debug, Clone)]
pub struct OponError {
    pub kind: OponErrorKind,
    pub limit: usize,
    pub requested: usize,
    pub current_usage: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OponErrorKind {
    /// Tried to allocate beyond max slots
    MemoryLimitExceeded,
    /// Tried to access invalid address
    InvalidAddress,
}

impl fmt::Display for OponError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            OponErrorKind::MemoryLimitExceeded => {
                write!(
                    f,
                    "Opon memory limit exceeded: requested slot {} but limit is {} ({} slots in use). Hint: Use '#opon nla' or '#opon ailopin' for more capacity.",
                    self.requested, self.limit, self.current_usage
                )
            }
            OponErrorKind::InvalidAddress => {
                write!(f, "Invalid memory address: {}", self.requested)
            }
        }
    }
}

impl std::error::Error for OponError {}

/// Result type for Opon operations
pub type OponResult<T> = Result<T, OponError>;

/// Size presets for Opon (the sacred calabash / memory container)
///
/// Each size has both Yoruba and English names:
/// - Kekere / Small / Tiny - For embedded systems
/// - Arinrin / Medium / Standard - Default size
/// - Nla / Large / Big - For large workloads  
/// - Ailopin / Unlimited / Dynamic - Grows as needed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OponSize {
    /// 4KB - For constrained embedded (256 slots)
    /// Aliases: kekere, small, tiny, embedded
    Kekere,
    /// 64KB - Standard (4096 slots)
    /// Aliases: arinrin, medium, standard, default
    Arinrin,
    /// 1MB - Large workloads (65536 slots)
    /// Aliases: nla, large, big, mega
    Nla,
    /// Unlimited - Uses Vec with dynamic growth
    /// Aliases: ailopin, unlimited, dynamic, infinite
    Ailopin,
}

impl OponSize {
    /// Get the number of memory slots for this size
    pub fn slot_count(&self) -> usize {
        match self {
            OponSize::Kekere => 256,
            OponSize::Arinrin => 4096,
            OponSize::Nla => 65536,
            OponSize::Ailopin => usize::MAX,
        }
    }

    /// Parse from string (supports both Yoruba and English names)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            // Yoruba names
            "kekere" | "kẹ́kẹ́rẹ́" => Some(OponSize::Kekere),
            "arinrin" | "àrínrin" => Some(OponSize::Arinrin),
            "nla" | "nlá" => Some(OponSize::Nla),
            "ailopin" | "àìlópin" => Some(OponSize::Ailopin),
            // English aliases
            "small" | "tiny" | "embedded" | "micro" => Some(OponSize::Kekere),
            "medium" | "standard" | "default" | "normal" => Some(OponSize::Arinrin),
            "large" | "big" | "mega" | "xl" => Some(OponSize::Nla),
            "unlimited" | "dynamic" | "infinite" | "max" => Some(OponSize::Ailopin),
            _ => None,
        }
    }

    /// Get human-readable name (bilingual)
    pub fn display_name(&self) -> &'static str {
        match self {
            OponSize::Kekere => "Kekere (Small)",
            OponSize::Arinrin => "Arinrin (Standard)",
            OponSize::Nla => "Nla (Large)",
            OponSize::Ailopin => "Ailopin (Unlimited)",
        }
    }

    /// Get approximate memory usage
    pub fn approx_memory(&self) -> &'static str {
        match self {
            OponSize::Kekere => "~4KB",
            OponSize::Arinrin => "~64KB",
            OponSize::Nla => "~1MB",
            OponSize::Ailopin => "Dynamic",
        }
    }
}

/// A recorded event in the flight recorder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OponEvent {
    /// Which Odù domain (e.g., "Ìrosù")
    pub spirit: String,
    /// What action (e.g., "fọ̀ (spoke)")
    pub action: String,
    /// What value was involved
    pub value: String,
}

/// An Ẹbọ Epoch - a scoped allocation region
/// All allocations within an epoch are released together when the epoch ends.
/// This provides deterministic memory management without garbage collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EboEpoch {
    /// Epoch ID (monotonically increasing)
    pub id: usize,
    /// Name of the epoch (e.g., "request_handler", "frame", "transaction")
    pub name: String,
    /// First address allocated in this epoch
    pub start_addr: usize,
    /// Number of allocations in this epoch
    pub alloc_count: usize,
    /// Whether this epoch is still active
    pub active: bool,
}

impl EboEpoch {
    pub fn new(id: usize, name: &str, start_addr: usize) -> Self {
        EboEpoch {
            id,
            name: name.to_string(),
            start_addr,
            alloc_count: 0,
            active: true,
        }
    }
}

/// The Opon - Central State Machine with Flight Recorder
/// Extended with Ẹbọ Epochs for scoped allocation
#[derive(Serialize, Deserialize)]
pub struct Opon {
    /// Memory slots
    memory: Vec<IfaValue>,
    /// Maximum slots (0 = unlimited)
    max_slots: usize,

    /// Circular buffer for flight recorder
    #[serde(skip, default)]
    history: Vec<OponEvent>,
    /// Current write position in history
    #[serde(skip, default)]
    cursor: usize,
    /// History capacity (256 = 16 × 16, sacred number)
    #[serde(skip, default = "default_history_capacity")]
    history_capacity: usize,

    // ═══════════════════════════════════════════════════════════════════
    // Ẹbọ Epoch System
    // ═══════════════════════════════════════════════════════════════════
    /// Stack of active epochs (innermost is last)
    epochs: Vec<EboEpoch>,
    /// Next epoch ID
    #[allow(dead_code)]
    next_epoch_id: usize,
    /// High water mark (highest address ever allocated)
    high_water: usize,
}

fn default_history_capacity() -> usize {
    256
}

impl Default for Opon {
    fn default() -> Self {
        Self::new(OponSize::Arinrin)
    }
}

impl Opon {
    /// Create new Opon with specified size
    pub fn new(size: OponSize) -> Self {
        let slots = match size {
            OponSize::Ailopin => 1024, // Start with 1024, grow as needed
            s => s.slot_count(),
        };

        Opon {
            memory: Vec::with_capacity(slots),
            max_slots: size.slot_count(),
            history: Vec::with_capacity(256),
            cursor: 0,
            history_capacity: 256,
            epochs: Vec::new(),
            next_epoch_id: 0,
            high_water: 0,
        }
    }

    /// Create with default size (Arinrin/64KB)
    pub fn create_default() -> Self {
        Self::new(OponSize::Arinrin)
    }

    /// Create for embedded use (Kekere/4KB)
    pub fn embedded() -> Self {
        Self::new(OponSize::Kekere)
    }

    // =========================================================================
    // MEMORY OPERATIONS
    // =========================================================================

    /// Get value at memory address
    pub fn get(&self, addr: usize) -> Option<&IfaValue> {
        self.memory.get(addr)
    }

    /// Set value at memory address (returns bool for backward compatibility)
    pub fn set(&mut self, addr: usize, value: IfaValue) -> bool {
        self.try_set(addr, value).is_ok()
    }

    /// Set value at memory address with detailed error
    pub fn try_set(&mut self, addr: usize, value: IfaValue) -> OponResult<()> {
        // Grow if needed (for unlimited mode)
        if addr >= self.memory.len() {
            if self.max_slots == usize::MAX {
                if addr >= AILOPIN_HARD_LIMIT {
                    return Err(OponError {
                        kind: OponErrorKind::MemoryLimitExceeded,
                        limit: AILOPIN_HARD_LIMIT,
                        requested: addr,
                        current_usage: self.memory_used(),
                    });
                }
                // Unlimited mode - grow up to a hard host-safety ceiling
                self.memory.resize(addr + 1, IfaValue::null());
            } else if addr >= self.max_slots {
                // LIMIT EXCEEDED - return error with helpful message
                return Err(OponError {
                    kind: OponErrorKind::MemoryLimitExceeded,
                    limit: self.max_slots,
                    requested: addr,
                    current_usage: self.memory_used(),
                });
            } else {
                // Within limits - grow
                self.memory.resize(addr + 1, IfaValue::null());
            }
        }

        self.memory[addr] = value;
        Ok(())
    }

    /// Allocate next available slot (returns address or error)
    pub fn allocate(&mut self, value: IfaValue) -> OponResult<usize> {
        let addr = self.memory.len();

        // Check if we'd exceed the limit
        if self.max_slots != usize::MAX && addr >= self.max_slots {
            return Err(OponError {
                kind: OponErrorKind::MemoryLimitExceeded,
                limit: self.max_slots,
                requested: addr + 1,
                current_usage: self.memory_used(),
            });
        }

        self.memory.push(value);

        // Update high water mark
        if self.memory.len() > self.high_water {
            self.high_water = self.memory.len();
        }

        // Track allocation in current epoch
        if let Some(epoch) = self.epochs.last_mut() {
            epoch.alloc_count += 1;
        }

        Ok(addr)
    }

    // =========================================================================
    // Ẹbọ EPOCHS (SCOPED ALLOCATION)
    // =========================================================================

    /// Begin a new scoped allocation epoch.
    ///
    /// All allocations performed while the epoch is active are released together
    /// when the epoch ends.
    pub fn begin_epoch(&mut self, name: &str) {
        let id = self.next_epoch_id;
        self.next_epoch_id = self.next_epoch_id.saturating_add(1);
        let start_addr = self.memory.len();
        self.epochs.push(EboEpoch::new(id, name, start_addr));
    }

    /// Return the current epoch (innermost), if any.
    pub fn current_epoch(&self) -> Option<&EboEpoch> {
        self.epochs.last()
    }

    /// End the current epoch and release all allocations made within it.
    pub fn end_epoch(&mut self) -> OponResult<()> {
        let mut epoch = self.epochs.pop().ok_or(OponError {
            kind: OponErrorKind::InvalidAddress,
            limit: self.max_slots,
            requested: 0,
            current_usage: self.memory_used(),
        })?;

        epoch.active = false;
        self.memory.truncate(epoch.start_addr);
        Ok(())
    }

    /// Get current memory usage
    pub fn memory_used(&self) -> usize {
        self.memory
            .iter()
            .filter(|v| !matches!(v, IfaValue::Null))
            .count()
    }

    // ... (rest of implementation) ...

    /// Record an event in the flight recorder
    pub fn record(&mut self, spirit: &str, action: &str, value: &IfaValue) {
        let event = OponEvent {
            spirit: spirit.to_string(),
            action: action.to_string(),
            value: value.to_string(),
        };

        if self.history.len() < self.history_capacity {
            self.history.push(event);
        } else {
            self.history[self.cursor] = event;
            self.cursor = (self.cursor + 1) % self.history_capacity;
        }
    }

    /// Get flight recorder history
    pub fn get_history(&self) -> Vec<OponEvent> {
        let mut events = Vec::new();
        // If wrapped, start from cursor
        if self.history.len() == self.history_capacity {
            for i in 0..self.history_capacity {
                let idx = (self.cursor + i) % self.history_capacity;
                events.push(self.history[idx].clone());
            }
        } else {
            events = self.history.clone();
        }
        events
    }

    /// Helper for recording simple messages (for tests/legacy)
    pub fn record_msg(&mut self, spirit: &str, action: &str, msg: &str) {
        self.record(spirit, action, &IfaValue::str(msg));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_operations() {
        let mut opon = Opon::default();

        assert!(opon.set(0, IfaValue::int(42)));
        // Check value using kind()
        if let Some(val) = opon.get(0) {
            match val {
                IfaValue::Int(v) => assert_eq!(*v, 42),
                _ => panic!("Expected Int"),
            }
        } else {
            panic!("Value not set");
        }
    }

    #[test]
    fn test_flight_recorder() {
        let mut opon = Opon::default();

        opon.record("Ìrosù", "fọ̀", &IfaValue::str("Hello"));
        opon.record("Ọ̀bàrà", "fikun", &IfaValue::int(42));

        let history = opon.get_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].spirit, "Ìrosù");
        assert_eq!(history[1].spirit, "Ọ̀bàrà");
    }

    #[test]
    fn test_circular_buffer() {
        let mut opon = Opon::embedded(); // Small history

        // Fill beyond capacity
        for i in 0..300 {
            opon.record_msg("Test", "event", &format!("{}", i));
        }

        let history = opon.get_history();
        assert_eq!(history.len(), 256); // Capped at capacity
    }

    #[test]
    fn test_ailopin_has_host_safety_ceiling() {
        let mut opon = Opon::new(OponSize::Ailopin);
        let err = opon
            .try_set(AILOPIN_HARD_LIMIT, IfaValue::int(1))
            .expect_err("expected hard limit error");
        assert_eq!(err.kind, OponErrorKind::MemoryLimitExceeded);
        assert_eq!(err.limit, AILOPIN_HARD_LIMIT);
    }

    #[test]
    fn test_ebo_epochs() {
        let mut opon = Opon::default();

        // Start an epoch
        opon.begin_epoch("request");
        let start_used = opon.memory_used();

        // Allocate within epoch
        opon.allocate(IfaValue::int(1)).unwrap();
        opon.allocate(IfaValue::int(2)).unwrap();

        // Check usage increased
        assert_eq!(opon.memory_used(), start_used + 2);

        // Check epoch stats
        let epoch = opon.current_epoch().unwrap();
        assert_eq!(epoch.name, "request");
        assert_eq!(epoch.alloc_count, 2);

        // End epoch
        opon.end_epoch().unwrap();

        // Check memory released
        assert_eq!(opon.memory_used(), start_used);
    }

    #[test]
    fn test_nested_epochs() {
        let mut opon = Opon::default();

        opon.begin_epoch("outer");
        opon.allocate(IfaValue::int(1)).unwrap();

        opon.begin_epoch("inner");
        opon.allocate(IfaValue::int(2)).unwrap();

        // Check total usage
        assert_eq!(opon.memory.len(), 2);

        // End inner epoch
        opon.end_epoch().unwrap();
        assert_eq!(opon.memory.len(), 1); // Inner released, outer remains

        // End outer epoch
        opon.end_epoch().unwrap();
        assert_eq!(opon.memory.len(), 0); // All released
    }
}
