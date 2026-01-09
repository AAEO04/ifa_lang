//! # Opon - Memory Management (Calabash)
//!
//! The Opon is the sacred calabash - the memory container for the Ifá-Lang VM.
//! It provides:
//! - Memory slots for variables
//! - Flight recorder for debugging
//! - Configurable sizing for embedded vs full systems
//! - **Memory limit enforcement** - prevents exceeding configured limits

use crate::value::IfaValue;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

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
#[derive(Debug, Clone)]
pub struct OponEvent {
    /// Which Odù domain (e.g., "Ìrosù")
    pub spirit: String,
    /// What action (e.g., "fọ̀ (spoke)")
    pub action: String,
    /// What value was involved
    pub value: String,
}

/// The Opon - Central State Machine with Flight Recorder
pub struct Opon {
    /// Memory slots
    memory: Vec<IfaValue>,
    /// Maximum slots (0 = unlimited)
    max_slots: usize,

    /// Circular buffer for flight recorder
    history: Vec<OponEvent>,
    /// Current write position in history
    cursor: usize,
    /// History capacity (256 = 16 × 16, sacred number)
    history_capacity: usize,
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
            memory: vec![IfaValue::Null; slots],
            max_slots: size.slot_count(),
            history: Vec::with_capacity(256),
            cursor: 0,
            history_capacity: 256,
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
                // Unlimited mode - grow freely
                self.memory.resize(addr + 1, IfaValue::Null);
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
                self.memory.resize(addr + 1, IfaValue::Null);
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
                requested: addr,
                current_usage: self.memory_used(),
            });
        }

        self.memory.push(value);
        Ok(addr)
    }

    /// Check if we have room for N more slots
    pub fn can_allocate(&self, count: usize) -> bool {
        if self.max_slots == usize::MAX {
            true
        } else {
            self.memory.len() + count <= self.max_slots
        }
    }

    /// Get remaining capacity (usize::MAX for unlimited)
    pub fn remaining_capacity(&self) -> usize {
        if self.max_slots == usize::MAX {
            usize::MAX
        } else {
            self.max_slots.saturating_sub(self.memory.len())
        }
    }

    /// Get current memory usage
    pub fn memory_used(&self) -> usize {
        self.memory
            .iter()
            .filter(|v| !matches!(v, IfaValue::Null))
            .count()
    }

    /// Get total memory capacity
    pub fn memory_capacity(&self) -> usize {
        self.memory.len()
    }

    /// Get maximum allowed slots
    pub fn max_capacity(&self) -> usize {
        self.max_slots
    }

    // =========================================================================
    // FLIGHT RECORDER
    // =========================================================================

    /// Record an event to the circular buffer
    pub fn record(&mut self, spirit: &str, action: &str, val: &IfaValue) {
        let event = OponEvent {
            spirit: spirit.to_string(),
            action: action.to_string(),
            value: val.to_string(),
        };
        self.record_event(event);
    }

    /// Record a message event
    pub fn record_msg(&mut self, spirit: &str, action: &str, msg: &str) {
        let event = OponEvent {
            spirit: spirit.to_string(),
            action: action.to_string(),
            value: msg.to_string(),
        };
        self.record_event(event);
    }

    fn record_event(&mut self, event: OponEvent) {
        if self.history.len() < self.history_capacity {
            self.history.push(event);
        } else {
            self.history[self.cursor] = event;
            self.cursor = (self.cursor + 1) % self.history_capacity;
        }
    }

    /// Get history as vector (oldest first)
    pub fn get_history(&self) -> Vec<&OponEvent> {
        let count = self.history.len();
        if count == 0 {
            return Vec::new();
        }

        let start = if count < self.history_capacity {
            0
        } else {
            self.cursor
        };
        let mut result = Vec::with_capacity(count);

        for i in 0..count {
            let idx = (start + i) % count.min(self.history_capacity);
            result.push(&self.history[idx]);
        }

        result
    }

    /// Dump history to stderr (for debugging/crash reports)
    pub fn dump_history(&self) {
        eprintln!("\n=== IWORI'S REPORT (Flight Recorder) ===");

        let history = self.get_history();
        if history.is_empty() {
            eprintln!("  (The Opon is empty - no events recorded)");
            return;
        }

        eprintln!();
        for (i, event) in history.iter().enumerate() {
            let steps_ago = history.len() - i;
            eprintln!(
                "  Step -{:<3} | [{}] {} -> {}",
                steps_ago, event.spirit, event.action, event.value
            );
        }

        eprintln!();
        eprintln!(
            "  Total events: {} / {} capacity",
            history.len(),
            self.history_capacity
        );
        eprintln!("=========================================\n");
    }

    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.cursor = 0;
    }
}

// Thread-local Opon for panic handler access
thread_local! {
    pub static CURRENT_OPON: RefCell<Option<Rc<RefCell<Opon>>>> = const { RefCell::new(None) };
}

/// Create new Opon and register for panic handling
pub fn create_opon_with_panic_handler(size: OponSize) -> Rc<RefCell<Opon>> {
    use std::panic;
    use std::sync::Once;

    static INIT_PANIC_HANDLER: Once = Once::new();

    INIT_PANIC_HANDLER.call_once(|| {
        let default_hook = panic::take_hook();

        panic::set_hook(Box::new(move |panic_info| {
            eprintln!("\n");
            eprintln!("=== OYEKU'S INTERRUPTION (CRASH DETECTED) ===");
            eprintln!("\"The divination board trembles...");
            eprintln!(" What followed the cowries reveals the path to failure.\"");
            eprintln!();

            if let Some(location) = panic_info.location() {
                eprintln!(
                    "Location: {}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                );
            }

            if let Some(msg) = panic_info.payload().downcast_ref::<&str>() {
                eprintln!("Cause: {}", msg);
            } else if let Some(msg) = panic_info.payload().downcast_ref::<String>() {
                eprintln!("Cause: {}", msg);
            }

            eprintln!();

            CURRENT_OPON.with(|opon_cell| {
                if let Some(ref opon_rc) = *opon_cell.borrow() {
                    if let Ok(opon) = opon_rc.try_borrow() {
                        opon.dump_history();
                    }
                }
            });

            default_hook(panic_info);
        }));
    });

    let opon = Rc::new(RefCell::new(Opon::new(size)));

    CURRENT_OPON.with(|opon_cell| {
        *opon_cell.borrow_mut() = Some(Rc::clone(&opon));
    });

    opon
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_operations() {
        let mut opon = Opon::default();

        assert!(opon.set(0, IfaValue::Int(42)));
        assert_eq!(opon.get(0), Some(&IfaValue::Int(42)));
    }

    #[test]
    fn test_flight_recorder() {
        let mut opon = Opon::default();

        opon.record("Ìrosù", "fọ̀", &IfaValue::Str("Hello".to_string()));
        opon.record("Ọ̀bàrà", "fikun", &IfaValue::Int(42));

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
}
