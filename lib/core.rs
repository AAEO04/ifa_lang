// ═══════════════════════════════════════════════════════════════════════════
// IFÁ-LANG RUST RUNTIME - CORE TYPES
// src/lib/std/core.rs
// ═══════════════════════════════════════════════════════════════════════════
//
// This single file turns Rust into a dynamic runtime. It handles the IfaValue 
// enum and all the operator overloading required so that:
//   IfaValue::Int(5) + IfaValue::Int(5) == IfaValue::Int(10)
//
// ═══════════════════════════════════════════════════════════════════════════

use std::fmt;
use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div, Not, Neg};
use std::collections::HashMap;
use std::io::{self, Write, BufRead};
use std::panic;
use std::cell::RefCell;
use std::sync::Once;
use std::rc::Rc;  // For shared ownership of Objects

// Thread-local storage for Opon (needed for panic handler access)
thread_local! {
    pub static OPON: RefCell<Option<*mut Opon>> = RefCell::new(None);
}

// Ensure panic handler is only installed once
static INIT_PANIC_HANDLER: Once = Once::new();

// =============================================================================
// THE UNIVERSAL CONTAINER (ODÙ WRAPPER)
// This allows variables to change type at runtime (Dynamic Typing).
// =============================================================================

// Function signature for lambdas: takes arguments, returns a value
pub type IfaFn = Rc<dyn Fn(Vec<IfaValue>) -> IfaValue>;

#[derive(Clone)]
pub enum IfaValue {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    List(Vec<IfaValue>),
    Map(HashMap<String, IfaValue>),
    // OOP: Objects are heap-allocated, reference-counted hashmaps
    Object(Rc<RefCell<HashMap<String, IfaValue>>>),
    // Lambdas: First-class functions
    Fn(IfaFn),
    Null,
}

// Manual Debug impl since IfaFn doesn't implement Debug
impl fmt::Debug for IfaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IfaValue::Int(v) => write!(f, "Int({})", v),
            IfaValue::Float(v) => write!(f, "Float({})", v),
            IfaValue::Str(v) => write!(f, "Str({:?})", v),
            IfaValue::Bool(v) => write!(f, "Bool({})", v),
            IfaValue::List(v) => write!(f, "List({:?})", v),
            IfaValue::Map(v) => write!(f, "Map({:?})", v),
            IfaValue::Object(v) => write!(f, "Object({:?})", v.borrow()),
            IfaValue::Fn(_) => write!(f, "Fn(<lambda>)"),
            IfaValue::Null => write!(f, "Null"),
        }
    }
}

// =============================================================================
// 1. DISPLAY TRAIT (How Ìrosù speaks)
// =============================================================================

impl fmt::Display for IfaValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IfaValue::Int(v) => write!(f, "{}", v),
            IfaValue::Float(v) => write!(f, "{}", v),
            IfaValue::Str(v) => write!(f, "{}", v), // No quotes for printing
            IfaValue::Bool(v) => write!(f, "{}", if *v { "òtítọ́" } else { "èké" }),
            IfaValue::List(v) => {
                let elems: Vec<String> = v.iter().map(|e| e.to_string()).collect();
                write!(f, "[{}]", elems.join(", "))
            },
            IfaValue::Map(m) => {
                let items: Vec<String> = m.iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v))
                    .collect();
                write!(f, "{{{}}}", items.join(", "))
            },
            IfaValue::Object(obj_ref) => {
                let obj = obj_ref.borrow();
                write!(f, "<Object with {} fields>", obj.len())
            },
            IfaValue::Fn(_) => write!(f, "<function>"),
            IfaValue::Null => write!(f, "àìsí"),
        }
    }
}

// =============================================================================
// 2. MATH OPERATIONS (Ọ̀bàrà & Òtúúrúpọ̀n)
// =============================================================================

// ADDITION (+)
impl Add for IfaValue {
    type Output = IfaValue;

    fn add(self, other: IfaValue) -> IfaValue {
        match (self, other) {
            // Integer Math
            (IfaValue::Int(a), IfaValue::Int(b)) => IfaValue::Int(a + b),
            // Float Math (Promote Int to Float)
            (IfaValue::Float(a), IfaValue::Float(b)) => IfaValue::Float(a + b),
            (IfaValue::Int(a), IfaValue::Float(b)) => IfaValue::Float(a as f64 + b),
            (IfaValue::Float(a), IfaValue::Int(b)) => IfaValue::Float(a + b as f64),
            // String Concatenation (Ìká.sọpọ̀)
            (IfaValue::Str(a), IfaValue::Str(b)) => IfaValue::Str(format!("{}{}", a, b)),
            (IfaValue::Str(a), b) => IfaValue::Str(format!("{}{}", a, b)),
            (a, IfaValue::Str(b)) => IfaValue::Str(format!("{}{}", a, b)),
            // List Concatenation
            (IfaValue::List(mut a), IfaValue::List(b)) => {
                a.extend(b);
                IfaValue::List(a)
            },
            _ => panic!("[Ọ̀kànràn] Runtime Error: Cannot ADD these types."),
        }
    }
}

// SUBTRACTION (-)
impl Sub for IfaValue {
    type Output = IfaValue;

    fn sub(self, other: IfaValue) -> IfaValue {
        match (self, other) {
            (IfaValue::Int(a), IfaValue::Int(b)) => IfaValue::Int(a - b),
            (IfaValue::Float(a), IfaValue::Float(b)) => IfaValue::Float(a - b),
            (IfaValue::Int(a), IfaValue::Float(b)) => IfaValue::Float(a as f64 - b),
            (IfaValue::Float(a), IfaValue::Int(b)) => IfaValue::Float(a - b as f64),
            _ => panic!("[Ọ̀kànràn] Runtime Error: Cannot SUBTRACT these types."),
        }
    }
}

// MULTIPLICATION (*)
impl Mul for IfaValue {
    type Output = IfaValue;

    fn mul(self, other: IfaValue) -> IfaValue {
        match (self, other) {
            (IfaValue::Int(a), IfaValue::Int(b)) => IfaValue::Int(a * b),
            (IfaValue::Float(a), IfaValue::Float(b)) => IfaValue::Float(a * b),
            (IfaValue::Int(a), IfaValue::Float(b)) => IfaValue::Float(a as f64 * b),
            (IfaValue::Float(a), IfaValue::Int(b)) => IfaValue::Float(a * b as f64),
            // Python-style String Repetition ("Na" * 3 = "NaNaNa")
            (IfaValue::Str(s), IfaValue::Int(n)) => IfaValue::Str(s.repeat(n as usize)),
            (IfaValue::Int(n), IfaValue::Str(s)) => IfaValue::Str(s.repeat(n as usize)),
            _ => panic!("[Ọ̀kànràn] Runtime Error: Cannot MULTIPLY these types."),
        }
    }
}

// DIVISION (/)
impl Div for IfaValue {
    type Output = IfaValue;

    fn div(self, other: IfaValue) -> IfaValue {
        match (self, other) {
            (IfaValue::Int(a), IfaValue::Int(b)) => {
                if b == 0 { panic!("[Ọ̀kànràn] Math Error: Division by Zero (Void)"); }
                // Auto-float for cleaner division
                IfaValue::Float(a as f64 / b as f64)
            },
            (IfaValue::Float(a), IfaValue::Float(b)) => {
                if b == 0.0 { panic!("[Ọ̀kànràn] Math Error: Division by Zero (Void)"); }
                IfaValue::Float(a / b)
            },
            (IfaValue::Int(a), IfaValue::Float(b)) => {
                if b == 0.0 { panic!("[Ọ̀kànràn] Math Error: Division by Zero (Void)"); }
                IfaValue::Float(a as f64 / b)
            },
            (IfaValue::Float(a), IfaValue::Int(b)) => {
                if b == 0 { panic!("[Ọ̀kànràn] Math Error: Division by Zero (Void)"); }
                IfaValue::Float(a / b as f64)
            },
            _ => panic!("[Ọ̀kànràn] Runtime Error: Cannot DIVIDE these types."),
        }
    }
}

// NEGATION (-x)
impl Neg for IfaValue {
    type Output = IfaValue;
    fn neg(self) -> IfaValue {
        match self {
            IfaValue::Int(a) => IfaValue::Int(-a),
            IfaValue::Float(a) => IfaValue::Float(-a),
            _ => panic!("[Ọ̀kànràn] Runtime Error: Cannot negate non-number."),
        }
    }
}

// LOGICAL NOT (!x)
impl Not for IfaValue {
    type Output = IfaValue;
    fn not(self) -> IfaValue {
        match self {
            IfaValue::Bool(b) => IfaValue::Bool(!b),
            IfaValue::Int(0) => IfaValue::Bool(true),
            IfaValue::Int(_) => IfaValue::Bool(false),
            IfaValue::Null => IfaValue::Bool(true),
            IfaValue::Str(ref s) if s.is_empty() => IfaValue::Bool(true),
            _ => IfaValue::Bool(false),
        }
    }
}

// =============================================================================
// 3. LOGIC OPERATIONS (Dídá / Comparison)
// =============================================================================

// EQUALITY (==)
impl PartialEq for IfaValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (IfaValue::Int(a), IfaValue::Int(b)) => a == b,
            (IfaValue::Float(a), IfaValue::Float(b)) => a == b,
            (IfaValue::Int(a), IfaValue::Float(b)) => (*a as f64) == *b,
            (IfaValue::Float(a), IfaValue::Int(b)) => *a == (*b as f64),
            (IfaValue::Str(a), IfaValue::Str(b)) => a == b,
            (IfaValue::Bool(a), IfaValue::Bool(b)) => a == b,
            (IfaValue::Null, IfaValue::Null) => true,
            _ => false, // Different types are never equal
        }
    }
}

// ORDERING (<, >, <=, >=)
impl PartialOrd for IfaValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (IfaValue::Int(a), IfaValue::Int(b)) => a.partial_cmp(b),
            (IfaValue::Float(a), IfaValue::Float(b)) => a.partial_cmp(b),
            (IfaValue::Int(a), IfaValue::Float(b)) => (*a as f64).partial_cmp(b),
            (IfaValue::Float(a), IfaValue::Int(b)) => a.partial_cmp(&(*b as f64)),
            (IfaValue::Str(a), IfaValue::Str(b)) => a.partial_cmp(b),
            _ => None, // Cannot compare String < Int
        }
    }
}

// =============================================================================
// 4. HELPER METHODS
// =============================================================================

impl IfaValue {
    /// Check if value is truthy (for control flow)
    pub fn is_truthy(&self) -> bool {
        match self {
            IfaValue::Bool(b) => *b,
            IfaValue::Int(n) => *n != 0,
            IfaValue::Float(f) => *f != 0.0,
            IfaValue::Str(s) => !s.is_empty(),
            IfaValue::List(l) => !l.is_empty(),
            IfaValue::Map(m) => !m.is_empty(),
            IfaValue::Null => false,
        }
    }
    
    /// Get length (for strings, lists, maps)
    pub fn len(&self) -> IfaValue {
        match self {
            IfaValue::Str(s) => IfaValue::Int(s.len() as i64),
            IfaValue::List(l) => IfaValue::Int(l.len() as i64),
            IfaValue::Map(m) => IfaValue::Int(m.len() as i64),
            _ => IfaValue::Int(0),
        }
    }
    
    /// Index access for lists and maps
    pub fn get(&self, index: &IfaValue) -> IfaValue {
        match (self, index) {
            (IfaValue::List(l), IfaValue::Int(i)) => {
                l.get(*i as usize).cloned().unwrap_or(IfaValue::Null)
            },
            (IfaValue::Map(m), IfaValue::Str(k)) => {
                m.get(k).cloned().unwrap_or(IfaValue::Null)
            },
            (IfaValue::Str(s), IfaValue::Int(i)) => {
                s.chars().nth(*i as usize)
                    .map(|c| IfaValue::Str(c.to_string()))
                    .unwrap_or(IfaValue::Null)
            },
            _ => IfaValue::Null,
        }
    }
    
    /// Index set for lists and maps
    pub fn set(&mut self, index: &IfaValue, value: IfaValue) {
        match (self, index) {
            (IfaValue::List(ref mut l), IfaValue::Int(i)) => {
                let idx = *i as usize;
                if idx < l.len() {
                    l[idx] = value;
                }
            },
            (IfaValue::Map(ref mut m), IfaValue::Str(k)) => {
                m.insert(k.clone(), value);
            },
            _ => {},
        }
    }
    
    /// Push to list (Ògúndá.fi)
    pub fn push(&mut self, value: IfaValue) {
        if let IfaValue::List(ref mut l) = self {
            l.push(value);
        }
    }
    
    /// Pop from list (Ògúndá.mu)
    pub fn pop(&mut self) -> IfaValue {
        if let IfaValue::List(ref mut l) = self {
            l.pop().unwrap_or(IfaValue::Null)
        } else {
            IfaValue::Null
        }
    }
}

// =============================================================================
// 5. THE OPON (Virtual Machine State) with Ring Buffer Debugger
// =============================================================================

/// A single recorded event in the flight recorder
#[derive(Debug, Clone)]
pub struct OponEvent {
    pub spirit: String,     // Which Odù (e.g., "Ìrosù")
    pub action: String,     // What action (e.g., "fọ̀ (spoke)")
    pub value: String,      // What value was involved
}

/// The Opon - Central State Machine with Flight Recorder
pub struct Opon {
    // Memory (256 slots)
    pub memory: Vec<IfaValue>,
    
    // === CIRCULAR BUFFER (Flight Recorder) ===
    // Records the last 256 events (16 Odù × 16 = sacred number)
    history: Vec<OponEvent>,
    cursor: usize,
    capacity: usize,
}

impl Opon {
    /// Create a new Opon WITHOUT panic handler (for testing)
    pub fn new() -> Self {
        Opon {
            memory: vec![IfaValue::Null; 256],
            history: Vec::with_capacity(256),
            cursor: 0,
            capacity: 256,  // 16 × 16 = sacred
        }
    }
    
    /// Create a new Opon WITH panic handler that dumps flight recorder on crash
    pub fn new_with_panic_handler() -> Self {
        // Install panic handler once
        INIT_PANIC_HANDLER.call_once(|| {
            let default_hook = panic::take_hook();
            
            panic::set_hook(Box::new(move |panic_info| {
                // Print the divination-style crash header
                eprintln!("\n");
                eprintln!("╔══════════════════════════════════════════════════════════════╗");
                eprintln!("║          ⚡ ỌYẸ̀KÚ'S INTERRUPTION (CRASH DETECTED) ⚡         ║");
                eprintln!("╠══════════════════════════════════════════════════════════════╣");
                eprintln!("║  \"The divination board trembles...                           ║");
                eprintln!("║   What followed the cowries reveals the path to failure.\"   ║");
                eprintln!("╚══════════════════════════════════════════════════════════════╝");
                eprintln!();
                
                // Print panic info
                if let Some(location) = panic_info.location() {
                    eprintln!("📍 Location: {}:{}:{}", 
                        location.file(), 
                        location.line(), 
                        location.column()
                    );
                }
                
                if let Some(msg) = panic_info.payload().downcast_ref::<&str>() {
                    eprintln!("💀 Cause: {}", msg);
                } else if let Some(msg) = panic_info.payload().downcast_ref::<String>() {
                    eprintln!("💀 Cause: {}", msg);
                }
                
                eprintln!();
                
                // Try to dump flight recorder from thread-local storage
                OPON.with(|opon_cell| {
                    if let Some(opon_ptr) = *opon_cell.borrow() {
                        // Safety: We're in single-threaded context during panic
                        unsafe {
                            let opon = &*opon_ptr;
                            eprintln!("┌─────────────────────────────────────────────────────────────┐");
                            eprintln!("│          📜 ÌWÒRÌ'S DYING REPORT (Last Events)              │");
                            eprintln!("└─────────────────────────────────────────────────────────────┘");
                            opon.dump_history();
                        }
                    } else {
                        eprintln!("⚠️  No flight recorder data available.");
                    }
                });
                
                eprintln!();
                eprintln!("════════════════════════════════════════════════════════════════");
                
                // Call the default panic hook
                default_hook(panic_info);
            }));
        });
        
        let mut opon = Self::new();
        
        // Register this Opon instance for the panic handler
        OPON.with(|opon_cell| {
            *opon_cell.borrow_mut() = Some(&mut opon as *mut Opon);
        });
        
        opon
    }
    
    // ═══════════════════════════════════════════════════════════════════
    // FLIGHT RECORDER METHODS
    // ═══════════════════════════════════════════════════════════════════
    
    /// Record an event to the circular buffer.
    /// When full, overwrites oldest entry (the snake bites its tail).
    pub fn record(&mut self, spirit: &str, action: &str, val: &IfaValue) {
        let event = OponEvent {
            spirit: spirit.to_string(),
            action: action.to_string(),
            value: val.to_string(),
        };
        
        if self.history.len() < self.capacity {
            self.history.push(event);
        } else {
            self.history[self.cursor] = event;
            self.cursor = (self.cursor + 1) % self.capacity;
        }
    }
    
    /// Record a simple string message
    pub fn record_msg(&mut self, spirit: &str, action: &str, msg: &str) {
        let event = OponEvent {
            spirit: spirit.to_string(),
            action: action.to_string(),
            value: msg.to_string(),
        };
        
        if self.history.len() < self.capacity {
            self.history.push(event);
        } else {
            self.history[self.cursor] = event;
            self.cursor = (self.cursor + 1) % self.capacity;
        }
    }
    
    /// Dump the history buffer (Time Travel / Post-Mortem)
    pub fn dump_history(&self) {
        println!("\n╔══════════════════════════════════════════════════════════════╗");
        println!("║            📜 ÌWÒRÌ'S REPORT (Flight Recorder)               ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        
        let count = self.history.len();
        if count == 0 {
            println!("  (The Opon is empty - no events recorded)");
            return;
        }
        
        // Start reading from the oldest entry (cursor) forward
        let start = if count < self.capacity { 0 } else { self.cursor };
        
        println!();
        for i in 0..count {
            let idx = (start + i) % count.min(self.capacity);
            let event = &self.history[idx];
            let steps_ago = count - i;
            println!("  Step -{:<3} │ [{}] {} → {}", 
                     steps_ago, event.spirit, event.action, event.value);
        }
        
        println!();
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║  Total events: {} / {} capacity                              ║", count, self.capacity);
        println!("╚══════════════════════════════════════════════════════════════╝\n");
    }
    
    /// Get history as vector (for programmatic access)
    pub fn get_history(&self) -> Vec<OponEvent> {
        let count = self.history.len();
        if count == 0 {
            return Vec::new();
        }
        
        let start = if count < self.capacity { 0 } else { self.cursor };
        let mut result = Vec::with_capacity(count);
        
        for i in 0..count {
            let idx = (start + i) % count.min(self.capacity);
            result.push(self.history[idx].clone());
        }
        
        result
    }
    
    /// Clear history buffer
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.cursor = 0;
    }
}

// =============================================================================
// 6. EXPORTED SPIRITS (Standard Library Wrappers) - With Flight Recording
// =============================================================================

pub mod opon {
    use super::*;

    // === ÌROSÙ: Print ===
    pub fn irosu_fo(opon: &mut Opon, val: &IfaValue) {
        opon.record("Ìrosù", "fọ̀ (spoke)", val);
        println!("{}", val);
    }
    
    pub fn irosu_so(opon: &mut Opon, label: &str, val: &IfaValue) {
        opon.record_msg("Ìrosù", "sọ (labeled)", &format!("[{}] {}", label, val));
        println!("[{}] {}", label, val);
    }

    // === OGBÈ: Initialize / Input ===
    pub fn ogbe_bi(opon: &mut Opon, val: &IfaValue) -> IfaValue {
        opon.record("Ogbè", "bí (birthed)", val);
        val.clone()
    }
    
    pub fn ogbe_gba(opon: &mut Opon) -> IfaValue {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().lock().read_line(&mut input).expect("Failed to read");
        let result = IfaValue::Str(input.trim().to_string());
        opon.record("Ogbè", "gbà (received)", &result);
        result
    }

    // === Ọ̀YẸKÚ: Exit ===
    pub fn oyeku_duro(opon: &mut Opon) {
        opon.record_msg("Ọ̀yẹ̀kú", "dúró (halted)", "Process Complete");
        println!("\n[Ọ̀yẹ̀kú] Àṣẹ! (Process Complete)");
    }
    
    pub fn oyeku_ku(opon: &mut Opon, code: i32) {
        opon.record_msg("Ọ̀yẹ̀kú", "kú (died)", &format!("Exit code: {}", code));
        std::process::exit(code);
    }

    // === ỌBÀRÀ: Add ===
    pub fn obara_fikun(opon: &mut Opon, a: &IfaValue, b: &IfaValue) -> IfaValue {
        let result = a.clone() + b.clone();
        opon.record("Ọ̀bàrà", "fikun (added)", &result);
        result
    }

    // === ÒTÚÚRÚPỌ̀N: Subtract ===
    pub fn oturupon_din(opon: &mut Opon, a: &IfaValue, b: &IfaValue) -> IfaValue {
        let result = a.clone() - b.clone();
        opon.record("Òtúúrúpọ̀n", "din (subtracted)", &result);
        result
    }

    // === ÒTÚRÁ: Network (Stubs) ===
    pub fn otura_ran(opon: &mut Opon, val: &IfaValue) {
        opon.record("Òtúrá", "rán (sent)", val);
        println!("[Òtúrá] Sending: {}", val);
    }
    
    pub fn otura_de(opon: &mut Opon, port: &IfaValue) {
        opon.record("Òtúrá", "dè (bound)", port);
        println!("[Òtúrá] Binding to port: {}", port);
    }
    
    pub fn otura_gba(opon: &mut Opon) -> IfaValue {
        let result = IfaValue::Str("[Simulated Response]".to_string());
        opon.record("Òtúrá", "gbà (received)", &result);
        result
    }

    // === ÒDÍ: Memory/Files ===
    pub fn odi_fi(opon: &mut Opon, addr: usize, val: &IfaValue) {
        opon.record_msg("Òdí", "fi (stored)", &format!("addr[{}] = {}", addr, val));
        if addr < opon.memory.len() {
            opon.memory[addr] = val.clone();
        }
    }
    
    pub fn odi_gba(opon: &mut Opon, addr: usize) -> IfaValue {
        if addr < opon.memory.len() {
            let result = opon.memory[addr].clone();
            opon.record_msg("Òdí", "gbà (loaded)", &format!("addr[{}] = {}", addr, result));
            result
        } else {
            IfaValue::Null
        }
    }

    // === ÒGÚNDÁ: List Operations ===
    pub fn ogunda_ge(opon: &mut Opon, a: &IfaValue, b: &IfaValue) -> IfaValue {
        let result = a.clone() / b.clone();
        opon.record("Ògúndá", "gé (divided)", &result);
        result
    }
    
    pub fn ogunda_fi(list: &mut IfaValue, val: IfaValue) {
        list.push(val);
    }
    
    // ═══════════════════════════════════════════════════════════════════
    // PHASE 2 MODULES: Regex, Dates, Env
    // ═══════════════════════════════════════════════════════════════════

    // ─────────────────────────────────────────────────────────────────
    // ÌKÁ (Text) - REGEX EXTENSIONS
    // ─────────────────────────────────────────────────────────────────
    
    /// Ìká.wá (Find/Search) - Regex search
    #[cfg(feature = "full")]
    pub fn ika_wa(opon: &mut Opon, pattern: &IfaValue, text: &IfaValue) -> IfaValue {
        use regex::Regex; // Needs 'regex' crate in Cargo.toml
        
        let pat_str = pattern.to_string();
        let txt_str = text.to_string();
        
        opon.record_msg("Ìká", "wá (regex)", &pat_str);
        
        match Regex::new(&pat_str) {
            Ok(re) => IfaValue::Bool(re.is_match(&txt_str)),
            Err(e) => {
                eprintln!("[Ìká] Regex Error: {}", e);
                IfaValue::Bool(false)
            }
        }
    }

    /// Ìká.pọ̀ (Match/Extract) - Regex capture
    #[cfg(feature = "full")]
    pub fn ika_po(opon: &mut Opon, pattern: &IfaValue, text: &IfaValue) -> IfaValue {
        use regex::Regex;
        
        let pat_str = pattern.to_string();
        let txt_str = text.to_string();
        
        opon.record_msg("Ìká", "pọ̀ (extract)", &pat_str);
        
        match Regex::new(&pat_str) {
            Ok(re) => {
                if let Some(caps) = re.captures(&txt_str) {
                    // Try to get first capture group, else whole match
                    let m = caps.get(1).or_else(|| caps.get(0)).map_or("", |m| m.as_str());
                    IfaValue::Str(m.to_string())
                } else {
                    IfaValue::Null
                }
            },
            Err(_) => IfaValue::Null
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // ÌWÒRÌ (Time) - DATE FORMATTING
    // ─────────────────────────────────────────────────────────────────

    /// Ìwòrì.ọjọ́ (Date) - Format current date
    #[cfg(feature = "full")]
    pub fn iwori_ojo(opon: &mut Opon, format: &IfaValue) -> IfaValue {
        use chrono::Local; // Needs 'chrono' crate in Cargo.toml
        
        let fmt_str = format.to_string();
        let now = Local::now();
        let result = now.format(&fmt_str).to_string();
        
        opon.record_msg("Ìwòrì", "ọjọ́ (date)", &result);
        IfaValue::Str(result)
    }

    // ─────────────────────────────────────────────────────────────────
    // OGBÈ (System) - ENV VARS & SECRETS
    // ─────────────────────────────────────────────────────────────────

    /// Ogbè.àyíká (Environment) - Get env var
    #[cfg(feature = "full")]
    pub fn ogbe_ayika(opon: &mut Opon, key: &IfaValue) -> IfaValue {
        use std::env;
        
        let key_str = key.to_string();
        opon.record_msg("Ogbè", "àyíká (env)", &key_str);
        
        match env::var(&key_str) {
            Ok(val) => IfaValue::Str(val),
            Err(_) => IfaValue::Null
        }
    }
    
    /// Ogbè.awo (Secret) - Secure env get (redacted in logs)
    #[cfg(feature = "full")]
    pub fn ogbe_awo(opon: &mut Opon, key: &IfaValue) -> IfaValue {
        use std::env;
        
        let key_str = key.to_string();
        opon.record_msg("Ogbè", "awo (secret)", "***"); // redact!
        
        match env::var(&key_str) {
            Ok(val) => IfaValue::Str(val),
            Err(_) => IfaValue::Null
        }
    }
    
    pub fn ogunda_mu(list: &mut IfaValue) -> IfaValue {
        list.pop()
    }
    
    pub fn ogunda_ka(list: &IfaValue) -> IfaValue {
        list.len()
    }

    // === ÌKÁ: String Operations ===
    pub fn ika_sopo(opon: &mut Opon, a: &IfaValue, b: &IfaValue) -> IfaValue {
        let result = a.clone() + b.clone();
        opon.record("Ìká", "sọpọ̀ (joined)", &result);
        result
    }
    
    pub fn ika_ka(_opon: &mut Opon, s: &IfaValue) -> IfaValue {
        s.len()
    }

    // === ỌWỌNRÍN: Random ===
    pub fn owonrin_bo(opon: &mut Opon, max: &IfaValue) -> IfaValue {
        match max {
            IfaValue::Int(n) => {
                // Simple pseudo-random using timestamp
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as i64;
                let result = IfaValue::Int(now % n);
                opon.record("Ọ̀wọ́nrín", "bọ̀ (rolled)", &result);
                result
            },
            _ => IfaValue::Int(0),
        }
    }

    // === ÌWÒRÍ: Time & Debugger ===
    pub fn iwori_ago(opon: &mut Opon) -> IfaValue {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let result = IfaValue::Int(now as i64);
        opon.record("Ìwòrì", "ago (timestamp)", &result);
        result
    }
    
    pub fn iwori_duro(opon: &mut Opon, ms: &IfaValue) {
        opon.record("Ìwòrì", "dúró (sleeping)", ms);
        if let IfaValue::Int(n) = ms {
            std::thread::sleep(std::time::Duration::from_millis(*n as u64));
        }
    }
    
    /// THE DEBUGGER - Ìwòrì.ròyìn (Report)
    /// Dumps the entire flight recorder history
    pub fn iwori_royin(opon: &mut Opon) {
        // Don't record the reporting itself to keep log clean
        opon.dump_history();
    }
    
    /// Clear the flight recorder
    pub fn iwori_nu(opon: &mut Opon) {
        opon.clear_history();
        println!("[Ìwòrì] Flight recorder cleared.");
    }
    
    // ═══════════════════════════════════════════════════════════════════
    // NEW ODÙ: Ọ̀SÁ / Async / Proc / Thread - Concurrency
    // ═══════════════════════════════════════════════════════════════════
    
    /// Ọ̀sá.sá (Run/Spawn) - Spawn async task (stub)
    pub fn osa_sa(opon: &mut Opon, task_name: &IfaValue) {
        opon.record("Ọ̀sá", "sá (spawned)", task_name);
        println!("[Ọ̀sá] Spawning task: {}", task_name);
    }
    
    /// Ọ̀sá.dúró (Wait/Await) - Wait for completion
    pub fn osa_duro(opon: &mut Opon, ms: &IfaValue) {
        opon.record("Ọ̀sá", "dúró (waiting)", ms);
        if let IfaValue::Int(n) = ms {
            std::thread::sleep(std::time::Duration::from_millis(*n as u64));
        }
    }
    
    /// Ọ̀sá.ago (Time) - Get current timestamp
    pub fn osa_ago(opon: &mut Opon) -> IfaValue {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let result = IfaValue::Int(now);
        opon.record("Ọ̀sá", "ago (time)", &result);
        result
    }
    
    // ═══════════════════════════════════════════════════════════════════
    // NEW ODÙ: Ọ̀KÀNRÀN / Error / Except / Test - Error Handling
    // ═══════════════════════════════════════════════════════════════════
    
    /// Ọ̀kànràn.bínú (Throw/Angry) - Throw exception
    pub fn okanran_binu(opon: &mut Opon, msg: &IfaValue) {
        opon.record("Ọ̀kànràn", "bínú (threw)", msg);
        eprintln!("[Ọ̀kànràn] Error thrown: {}", msg);
        panic!("Ifá Exception: {}", msg);
    }
    
    /// Ọ̀kànràn.jẹ́ (Assert/It is so) - Assert condition
    pub fn okanran_je(opon: &mut Opon, condition: &IfaValue, msg: &IfaValue) {
        let is_true = match condition {
            IfaValue::Bool(b) => *b,
            IfaValue::Int(n) => *n != 0,
            _ => false,
        };
        
        if !is_true {
            opon.record("Ọ̀kànràn", "jẹ́ (assertion failed)", msg);
            panic!("Assertion failed: {}", msg);
        } else {
            opon.record_msg("Ọ̀kànràn", "jẹ́ (asserted)", "passed");
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════
    // NEW ODÙ: ÌRẸTẸ̀ / Crypto / Hash / Zip - Compression & Hashing
    // ═══════════════════════════════════════════════════════════════════
    
    /// Ìrẹtẹ̀.dì (Hash) - Simple hash function
    pub fn irete_di(opon: &mut Opon, val: &IfaValue) -> IfaValue {
        // Simple djb2 hash for demo
        let s = val.to_string();
        let mut hash: u64 = 5381;
        for c in s.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(c as u64);
        }
        let result = IfaValue::Int(hash as i64);
        opon.record("Ìrẹtẹ̀", "dì (hashed)", &result);
        result
    }
    
    /// Ìrẹtẹ̀.fún (Compress) - Stub for compression
    pub fn irete_fun(_opon: &mut Opon, _val: &IfaValue) -> IfaValue {
        println!("[Ìrẹtẹ̀] Compression not yet implemented");
        IfaValue::Null
    }
    
    /// Ìrẹtẹ̀.tú (Decompress) - Stub for decompression
    pub fn irete_tu(_opon: &mut Opon, _val: &IfaValue) -> IfaValue {
        println!("[Ìrẹtẹ̀] Decompression not yet implemented");
        IfaValue::Null
    }
    
    // ═══════════════════════════════════════════════════════════════════
    // NEW ODÙ: ÒFÚN / Meta / Reflect / Root - Permissions & Reflection
    // ═══════════════════════════════════════════════════════════════════
    
    /// Òfún.àṣẹ (Sudo/Elevate) - Request elevated permissions
    pub fn ofun_ase(opon: &mut Opon) {
        opon.record_msg("Òfún", "àṣẹ (elevated)", "Requesting permissions...");
        println!("[Òfún] Elevated permissions requested");
    }
    
    /// Òfún.fún (Grant) - Grant permission
    pub fn ofun_fun(opon: &mut Opon, permission: &IfaValue) {
        opon.record("Òfún", "fún (granted)", permission);
        println!("[Òfún] Permission granted: {}", permission);
    }
    
    /// Òfún.kà (Read manifest/config) - Read configuration
    pub fn ofun_ka(opon: &mut Opon, key: &IfaValue) -> IfaValue {
        opon.record("Òfún", "kà (read config)", key);
        // Stub: return null for now
        IfaValue::Null
    }
    
    /// Òfún.iru (Type of) - Reflection: get type name
    pub fn ofun_iru(_opon: &mut Opon, val: &IfaValue) -> IfaValue {
        let type_name = match val {
            IfaValue::Int(_) => "Int",
            IfaValue::Float(_) => "Float",
            IfaValue::Str(_) => "Str",
            IfaValue::Bool(_) => "Bool",
            IfaValue::List(_) => "List",
            IfaValue::Map(_) => "Map",
            IfaValue::Null => "Null",
        };
        IfaValue::Str(type_name.to_string())
    }
    
    // ═══════════════════════════════════════════════════════════════════
    // PRODUCTION MODULES: JSON, HTTP, SQLite, WebSocket
    // These require the dependencies in Cargo.toml to be compiled
    // ═══════════════════════════════════════════════════════════════════
    
    // ─────────────────────────────────────────────────────────────────
    // ÌKÁ (JSON) - String Controller with JSON parsing
    // ─────────────────────────────────────────────────────────────────
    
    /// Ìká.tú (Parse JSON) - Untie/decode JSON string to IfaValue
    #[cfg(feature = "full")]
    pub fn ika_tu(opon: &mut Opon, json_str: &IfaValue) -> IfaValue {
        use serde_json::Value;
        
        let s = json_str.to_string();
        opon.record_msg("Ìká", "tú (parsing JSON)", &s);
        
        match serde_json::from_str::<Value>(&s) {
            Ok(v) => convert_json_to_ifa(v),
            Err(e) => {
                eprintln!("[Ìká] JSON parse error: {}", e);
                IfaValue::Null
            }
        }
    }
    
    /// Ìká.dì (Stringify JSON) - Tie/encode IfaValue to JSON string
    #[cfg(feature = "full")]
    pub fn ika_di(opon: &mut Opon, val: &IfaValue) -> IfaValue {
        opon.record("Ìká", "dì (stringify JSON)", val);
        
        let json = convert_ifa_to_json(val);
        IfaValue::Str(json.to_string())
    }
    
    /// Ìká.gba_inu (Get nested value) - Access JSON object property
    #[cfg(feature = "full")]
    pub fn ika_gba_inu(opon: &mut Opon, obj: &IfaValue, key: &IfaValue) -> IfaValue {
        opon.record_msg("Ìká", "gbà inú (get nested)", &key.to_string());
        
        if let IfaValue::Map(map) = obj {
            let key_str = key.to_string();
            map.get(&key_str).cloned().unwrap_or(IfaValue::Null)
        } else {
            IfaValue::Null
        }
    }
    
    // ─────────────────────────────────────────────────────────────────
    // ÒTÚRÁ (HTTP) - The Messenger with real HTTP client
    // ─────────────────────────────────────────────────────────────────
    
    /// Òtúrá.gbà (HTTP GET) - Receive/fetch data from URL
    #[cfg(feature = "full")]
    pub fn otura_gba_http(opon: &mut Opon, url: &IfaValue) -> IfaValue {
        use reqwest::blocking::Client;
        
        let url_str = url.to_string();
        opon.record_msg("Òtúrá", "gbà (HTTP GET)", &url_str);
        
        let client = Client::new();
        match client.get(&url_str).send() {
            Ok(response) => {
                let text = response.text().unwrap_or_default();
                opon.record_msg("Òtúrá", "gbà (received)", 
                    &format!("{} bytes", text.len()));
                IfaValue::Str(text)
            },
            Err(e) => {
                eprintln!("[Òtúrá] HTTP GET error: {}", e);
                IfaValue::Null
            }
        }
    }
    
    /// Òtúrá.rán (HTTP POST) - Send data to URL
    #[cfg(feature = "full")]
    pub fn otura_ran_http(opon: &mut Opon, url: &IfaValue, body: &IfaValue) -> IfaValue {
        use reqwest::blocking::Client;
        
        let url_str = url.to_string();
        let body_str = body.to_string();
        opon.record_msg("Òtúrá", "rán (HTTP POST)", &url_str);
        
        let client = Client::new();
        match client.post(&url_str)
            .header("Content-Type", "application/json")
            .body(body_str)
            .send() 
        {
            Ok(response) => {
                let status = response.status().as_u16();
                opon.record_msg("Òtúrá", "rán (response)", 
                    &format!("status {}", status));
                IfaValue::Int(status as i64)
            },
            Err(e) => {
                eprintln!("[Òtúrá] HTTP POST error: {}", e);
                IfaValue::Int(-1)
            }
        }
    }
    
    /// Òtúrá.fi (HTTP PUT) - Update data
    #[cfg(feature = "full")]
    pub fn otura_fi_http(opon: &mut Opon, url: &IfaValue, body: &IfaValue) -> IfaValue {
        use reqwest::blocking::Client;
        
        let url_str = url.to_string();
        opon.record_msg("Òtúrá", "fi (HTTP PUT)", &url_str);
        
        let client = Client::new();
        match client.put(&url_str)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send() 
        {
            Ok(response) => IfaValue::Int(response.status().as_u16() as i64),
            Err(_) => IfaValue::Int(-1)
        }
    }
    
    /// Òtúrá.pa (HTTP DELETE) - Delete resource
    #[cfg(feature = "full")]
    pub fn otura_pa_http(opon: &mut Opon, url: &IfaValue) -> IfaValue {
        use reqwest::blocking::Client;
        
        let url_str = url.to_string();
        opon.record_msg("Òtúrá", "pa (HTTP DELETE)", &url_str);
        
        let client = Client::new();
        match client.delete(&url_str).send() {
            Ok(response) => IfaValue::Int(response.status().as_u16() as i64),
            Err(_) => IfaValue::Int(-1)
        }
    }
    
    // ─────────────────────────────────────────────────────────────────
    // ÒDÍ (SQLite) - The Sealed Container / Database
    // ─────────────────────────────────────────────────────────────────
    
    // Note: SQLite connections need to be stored in a resource table.
    // For now, we use a thread-local static to hold the connection.
    
    #[cfg(feature = "full")]
    thread_local! {
        static DB_CONN: std::cell::RefCell<Option<rusqlite::Connection>> = 
            std::cell::RefCell::new(None);
    }
    
    /// Òdí.sí (Open Database) - Open/create SQLite database
    #[cfg(feature = "full")]
    pub fn odi_si(opon: &mut Opon, path: &IfaValue) -> IfaValue {
        use rusqlite::Connection;
        
        let path_str = path.to_string();
        opon.record_msg("Òdí", "sí (opening DB)", &path_str);
        
        match Connection::open(&path_str) {
            Ok(conn) => {
                DB_CONN.with(|db| {
                    *db.borrow_mut() = Some(conn);
                });
                println!("[Òdí] Database opened: {}", path_str);
                IfaValue::Bool(true)
            },
            Err(e) => {
                eprintln!("[Òdí] Failed to open DB: {}", e);
                IfaValue::Bool(false)
            }
        }
    }
    
    /// Òdí.pa_ase (Execute SQL) - Run write query (CREATE, INSERT, UPDATE)
    #[cfg(feature = "full")]
    pub fn odi_pa_ase(opon: &mut Opon, sql: &IfaValue) -> IfaValue {
        let sql_str = sql.to_string();
        opon.record_msg("Òdí", "pa àṣẹ (execute)", &sql_str);
        
        DB_CONN.with(|db| {
            if let Some(conn) = db.borrow().as_ref() {
                match conn.execute(&sql_str, []) {
                    Ok(rows) => {
                        println!("[Òdí] Executed: {} rows affected", rows);
                        IfaValue::Int(rows as i64)
                    },
                    Err(e) => {
                        eprintln!("[Òdí] SQL error: {}", e);
                        IfaValue::Int(-1)
                    }
                }
            } else {
                eprintln!("[Òdí] No database connection. Call Òdí.sí() first.");
                IfaValue::Int(-1)
            }
        })
    }
    
    /// Òdí.kà_inú (Select Query) - Read data from database
    #[cfg(feature = "full")]
    pub fn odi_ka_inu(opon: &mut Opon, sql: &IfaValue) -> IfaValue {
        use rusqlite::params;
        
        let sql_str = sql.to_string();
        opon.record_msg("Òdí", "kà inú (select)", &sql_str);
        
        DB_CONN.with(|db| {
            if let Some(conn) = db.borrow().as_ref() {
                let mut stmt = match conn.prepare(&sql_str) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[Òdí] Prepare error: {}", e);
                        return IfaValue::List(vec![]);
                    }
                };
                
                let rows: Vec<IfaValue> = stmt.query_map([], |row| {
                    // Get first column as string (simplified)
                    let val: String = row.get(0).unwrap_or_default();
                    Ok(IfaValue::Str(val))
                }).unwrap_or_else(|_| vec![].into_iter())
                    .filter_map(|r| r.ok())
                    .collect();
                
                IfaValue::List(rows)
            } else {
                eprintln!("[Òdí] No database connection.");
                IfaValue::List(vec![])
            }
        })
    }
    
    /// Òdí.tì (Close Database) - Close connection
    #[cfg(feature = "full")]
    pub fn odi_ti(opon: &mut Opon) -> IfaValue {
        opon.record_msg("Òdí", "tì (closing DB)", "connection");
        
        DB_CONN.with(|db| {
            *db.borrow_mut() = None;
        });
        println!("[Òdí] Database closed.");
        IfaValue::Bool(true)
    }
}

// ═══════════════════════════════════════════════════════════════════
// JSON CONVERSION HELPERS
// ═══════════════════════════════════════════════════════════════════

#[cfg(feature = "full")]
fn convert_json_to_ifa(v: serde_json::Value) -> IfaValue {
    use serde_json::Value;
    
    match v {
        Value::String(s) => IfaValue::Str(s),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                IfaValue::Int(i)
            } else if let Some(f) = n.as_f64() {
                IfaValue::Float(f)
            } else {
                IfaValue::Null
            }
        },
        Value::Bool(b) => IfaValue::Bool(b),
        Value::Array(arr) => {
            let list: Vec<IfaValue> = arr.into_iter()
                .map(convert_json_to_ifa)
                .collect();
            IfaValue::List(list)
        },
        Value::Object(map) => {
            let mut hm = HashMap::new();
            for (k, v) in map {
                hm.insert(k, convert_json_to_ifa(v));
            }
            IfaValue::Map(hm)
        },
        Value::Null => IfaValue::Null,
    }
}

#[cfg(feature = "full")]
fn convert_ifa_to_json(val: &IfaValue) -> serde_json::Value {
    use serde_json::{Value, json};
    
    match val {
        IfaValue::Int(n) => json!(n),
        IfaValue::Float(f) => json!(f),
        IfaValue::Str(s) => json!(s),
        IfaValue::Bool(b) => json!(b),
        IfaValue::List(arr) => {
            let v: Vec<Value> = arr.iter()
                .map(convert_ifa_to_json)
                .collect();
            json!(v)
        },
        IfaValue::Map(map) => {
            let m: serde_json::Map<String, Value> = map.iter()
                .map(|(k, v)| (k.clone(), convert_ifa_to_json(v)))
                .collect();
            Value::Object(m)
        },
        IfaValue::Null => Value::Null,
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_add() {
        let result = IfaValue::Int(5) + IfaValue::Int(5);
        assert_eq!(result, IfaValue::Int(10));
    }

    #[test]
    fn test_int_float_add() {
        let result = IfaValue::Int(10) + IfaValue::Float(5.5);
        assert_eq!(result, IfaValue::Float(15.5));
    }

    #[test]
    fn test_string_concat() {
        let result = IfaValue::Str("Hello ".to_string()) + IfaValue::Str("World".to_string());
        assert_eq!(result, IfaValue::Str("Hello World".to_string()));
    }

    #[test]
    fn test_string_repeat() {
        let result = IfaValue::Str("Na".to_string()) * IfaValue::Int(3);
        assert_eq!(result, IfaValue::Str("NaNaNa".to_string()));
    }

    #[test]
    fn test_comparison() {
        assert!(IfaValue::Int(10) > IfaValue::Int(5));
        assert!(IfaValue::Float(3.14) < IfaValue::Float(3.15));
    }

    #[test]
    fn test_truthy() {
        assert!(IfaValue::Int(1).is_truthy());
        assert!(!IfaValue::Int(0).is_truthy());
        assert!(IfaValue::Str("hello".to_string()).is_truthy());
        assert!(!IfaValue::Null.is_truthy());
    }

    #[test]
    fn test_list_index() {
        let list = IfaValue::List(vec![IfaValue::Int(1), IfaValue::Int(2), IfaValue::Int(3)]);
        assert_eq!(list.get(&IfaValue::Int(0)), IfaValue::Int(1));
        assert_eq!(list.get(&IfaValue::Int(2)), IfaValue::Int(3));
    }
}

// =============================================================================
// OOP HELPERS (dá - Object Creation)
// =============================================================================

/// Get a field from an Object (dá instance)
pub fn get_field(obj: &IfaValue, field: &str) -> IfaValue {
    if let IfaValue::Object(map_ref) = obj {
        let map = map_ref.borrow();
        if let Some(val) = map.get(field) {
            return val.clone();
        }
    }
    IfaValue::Null
}

/// Set a field on an Object (dá instance)
pub fn set_field(obj: &IfaValue, field: &str, val: IfaValue) {
    if let IfaValue::Object(map_ref) = obj {
        let mut map = map_ref.borrow_mut();
        map.insert(field.to_string(), val);
    }
}

/// Create a new empty Object (used by transpiler for dá constructors)
pub fn new_object() -> IfaValue {
    IfaValue::Object(Rc::new(RefCell::new(HashMap::new())))
}

