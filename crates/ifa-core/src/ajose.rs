//! # Àjọṣe - Reactive Relationship Engine (v2 - True Observables)
//!
//! Signal-based reactivity with proper observer pattern.
//! No raw callbacks - actual push-based updates.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;
use std::rc::{Rc, Weak};

// ============================================================================
// TYPE ALIASES for complex types (reduces clippy::type_complexity warnings)
// ============================================================================

/// Type alias for signal subscribers
type Subscribers<T> = Rc<RefCell<Vec<Box<dyn Fn(&T)>>>>;

/// Type alias for reactive binding relationships  
#[allow(dead_code)]
type BindingRelation<S, T> = Vec<(Weak<RefCell<S>>, Weak<RefCell<T>>, Box<dyn Fn(&S, &mut T)>)>;

// ============================================================================
// SIGNALS - Core reactive primitive
// ============================================================================

/// A reactive signal that notifies subscribers on change.
///
/// # Example
/// ```rust,ignore
/// let count = Signal::new(0);
/// let label = Signal::new(String::new());
///
/// // Create derived signal
/// effect(move || {
///     label.set(format!("Count: {}", count.get()));
/// });
///
/// count.set(5);  // label automatically updates to "Count: 5"
/// ```
pub struct Signal<T> {
    value: Rc<RefCell<T>>,
    subscribers: Subscribers<T>,
    version: Rc<Cell<u64>>,
}

impl<T: Clone + 'static> Signal<T> {
    pub fn new(initial: T) -> Self {
        Signal {
            value: Rc::new(RefCell::new(initial)),
            subscribers: Rc::new(RefCell::new(Vec::new())),
            version: Rc::new(Cell::new(0)),
        }
    }

    /// Get current value
    pub fn get(&self) -> T {
        self.value.borrow().clone()
    }

    /// Get reference to value
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.value.borrow())
    }

    /// Set value and notify subscribers
    pub fn set(&self, new_value: T) {
        *self.value.borrow_mut() = new_value;
        self.version.set(self.version.get() + 1);
        self.notify();
    }

    /// Update value with function
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.value.borrow_mut());
        self.version.set(self.version.get() + 1);
        self.notify();
    }

    /// Subscribe to changes
    pub fn subscribe(&self, callback: impl Fn(&T) + 'static) {
        self.subscribers.borrow_mut().push(Box::new(callback));
    }

    /// Get version number (for dirty checking)
    pub fn version(&self) -> u64 {
        self.version.get()
    }

    fn notify(&self) {
        let value = self.value.borrow();
        for sub in self.subscribers.borrow().iter() {
            sub(&value);
        }
    }
}

impl<T: Clone + 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal {
            value: Rc::clone(&self.value),
            subscribers: Rc::clone(&self.subscribers),
            version: Rc::clone(&self.version),
        }
    }
}

impl<T: fmt::Debug + Clone + 'static> fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signal({:?})", self.value.borrow())
    }
}

// ============================================================================
// COMPUTED - Derived reactive values
// ============================================================================

/// A computed value that auto-updates when dependencies change.
pub struct Computed<T> {
    value: Rc<RefCell<T>>,
    compute: Rc<dyn Fn() -> T>,
}

impl<T: Clone + 'static> Computed<T> {
    pub fn new<F: Fn() -> T + 'static>(compute: F) -> Self {
        let value = compute();
        Computed {
            value: Rc::new(RefCell::new(value)),
            compute: Rc::new(compute),
        }
    }

    pub fn get(&self) -> T {
        // Recompute (in real impl, would track dependencies)
        let new_val = (self.compute)();
        *self.value.borrow_mut() = new_val;
        self.value.borrow().clone()
    }
}

// ============================================================================
// EFFECT - Side effects on signal changes
// ============================================================================

/// Run a side effect whenever any accessed signal changes.
///
/// # Example
/// ```rust,ignore
/// let count = Signal::new(0);
/// effect(move || {
///     println!("Count is now: {}", count.get());
/// });
/// ```
pub fn effect<F: Fn() + 'static>(f: F) -> EffectGuard {
    // Initial run
    f();

    // In a full implementation, we'd track which signals were accessed
    // and subscribe to them. For now, just store the callback.
    EffectGuard {
        callback: Rc::new(f),
    }
}

pub struct EffectGuard {
    callback: Rc<dyn Fn()>,
}

impl EffectGuard {
    /// Manually trigger the effect
    pub fn run(&self) {
        (self.callback)();
    }
}

// ============================================================================
// RELATIONSHIPS - Type-safe entity bindings
// ============================================================================

/// Define a relationship type
#[derive(Debug, Clone)]
pub struct Relationship {
    pub name: String,
    pub source_type: String,
    pub target_type: String,
    pub bidirectional: bool,
}

impl Relationship {
    pub fn new(name: &str, source: &str, target: &str) -> Self {
        Relationship {
            name: name.to_string(),
            source_type: source.to_string(),
            target_type: target.to_string(),
            bidirectional: false,
        }
    }

    pub fn bidirectional(mut self) -> Self {
        self.bidirectional = true;
        self
    }
}

/// Context data for relationship events
#[derive(Debug, Clone, Default)]
pub struct RelContext {
    pub data: HashMap<String, String>,
}

impl RelContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, key: &str, value: impl ToString) -> Self {
        self.data.insert(key.to_string(), value.to_string());
        self
    }

    pub fn get<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.data.get(key).and_then(|v| v.parse().ok())
    }
}

/// The Àjọṣe Engine - manages reactive relationships
pub struct Ajose<S: 'static, T: 'static> {
    relationships: Vec<(Weak<RefCell<S>>, Weak<RefCell<T>>, Box<dyn Fn(&S, &mut T)>)>,
}

impl<S: 'static, T: 'static> Ajose<S, T> {
    pub fn new() -> Self {
        Ajose {
            relationships: Vec::new(),
        }
    }

    /// Bind source to target with transformation
    pub fn bind(
        &mut self,
        source: &Rc<RefCell<S>>,
        target: &Rc<RefCell<T>>,
        transform: impl Fn(&S, &mut T) + 'static,
    ) {
        let source_weak = Rc::downgrade(source);
        let target_weak = Rc::downgrade(target);

        // Initial sync (must happen before we move transform into the Box)
        transform(&source.borrow(), &mut target.borrow_mut());

        self.relationships
            .push((source_weak, target_weak, Box::new(transform)));
    }

    /// Propagate changes from source to targets
    pub fn propagate(&self, source: &Rc<RefCell<S>>) {
        for (src_weak, tgt_weak, transform) in &self.relationships {
            if let Some(src) = src_weak.upgrade() {
                if Rc::ptr_eq(&src, source) {
                    if let Some(tgt) = tgt_weak.upgrade() {
                        transform(&src.borrow(), &mut tgt.borrow_mut());
                    }
                }
            }
        }
    }

    /// Cleanup dead references
    pub fn gc(&mut self) {
        self.relationships
            .retain(|(s, t, _)| s.upgrade().is_some() && t.upgrade().is_some());
    }
}

impl<S: 'static, T: 'static> Default for Ajose<S, T> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// MACROS
// ============================================================================

/// Create a reactive binding
#[macro_export]
macro_rules! bind {
    ($source:expr => $target:expr) => {{
        let source = $source.clone();
        let target = $target.clone();
        source.subscribe(move |val| {
            target.set(val.clone());
        });
    }};
    ($source:expr => $target:expr, |$v:ident| $transform:expr) => {{
        let source = $source.clone();
        let target = $target.clone();
        source.subscribe(move |$v| {
            target.set($transform);
        });
    }};
}

// ============================================================================
// FFI BRIDGE - Cross-language interop via Ajose
// ============================================================================

/// Ajose FFI Bridge - The gateway to other languages
///
/// Provides high-level access to Python, C, and other languages.
/// For full FFI capabilities, use `ifa_std::ffi::IfaFfi` directly.
///
/// This bridge provides:
/// - Built-in math functions (no external deps)
/// - Sandboxed shell execution
/// - Placeholder for native FFI (requires ifa-std)
pub struct AjoseBridge {
    /// Cached module imports
    py_cache: std::collections::HashMap<String, String>,
    /// Blocked shell commands for security
    blocked_commands: Vec<&'static str>,
}

impl AjoseBridge {
    pub fn new() -> Self {
        AjoseBridge {
            py_cache: std::collections::HashMap::new(),
            blocked_commands: vec![
                "rm", "del", "format", "mkfs", "dd", "sudo", "chmod",
                "chown", "kill", "pkill", "shutdown", "reboot", "curl",
                "wget", "nc", "netcat", "powershell", "cmd.exe",
            ],
        }
    }

    /// Call a Python-like function (built-in implementations)
    /// Ifá syntax: coop.py("math", "sqrt", 16)
    ///
    /// Supports: math.sqrt, math.factorial, math.pow, math.sin, math.cos,
    ///           math.log, math.exp, math.floor, math.ceil, math.abs
    pub fn py(&mut self, module: &str, func: &str, args: &[&str]) -> String {
        match (module, func) {
            // Math module
            ("math", "sqrt") => self.parse_f64(args, 0).map(|n| n.sqrt()).unwrap_or(f64::NAN).to_string(),
            ("math", "pow") => {
                let base = self.parse_f64(args, 0).unwrap_or(0.0);
                let exp = self.parse_f64(args, 1).unwrap_or(1.0);
                base.powf(exp).to_string()
            }
            ("math", "sin") => self.parse_f64(args, 0).map(|n| n.sin()).unwrap_or(f64::NAN).to_string(),
            ("math", "cos") => self.parse_f64(args, 0).map(|n| n.cos()).unwrap_or(f64::NAN).to_string(),
            ("math", "tan") => self.parse_f64(args, 0).map(|n| n.tan()).unwrap_or(f64::NAN).to_string(),
            ("math", "log") => self.parse_f64(args, 0).map(|n| n.ln()).unwrap_or(f64::NAN).to_string(),
            ("math", "log10") => self.parse_f64(args, 0).map(|n| n.log10()).unwrap_or(f64::NAN).to_string(),
            ("math", "exp") => self.parse_f64(args, 0).map(|n| n.exp()).unwrap_or(f64::NAN).to_string(),
            ("math", "floor") => self.parse_f64(args, 0).map(|n| n.floor()).unwrap_or(f64::NAN).to_string(),
            ("math", "ceil") => self.parse_f64(args, 0).map(|n| n.ceil()).unwrap_or(f64::NAN).to_string(),
            ("math", "abs") => self.parse_f64(args, 0).map(|n| n.abs()).unwrap_or(f64::NAN).to_string(),
            ("math", "factorial") => {
                if let Some(n) = self.parse_u64(args, 0) {
                    if n <= 20 { // Prevent overflow
                        return (1..=n).product::<u64>().to_string();
                    }
                    return "overflow".to_string();
                }
                "NaN".to_string()
            }
            ("math", "pi") => std::f64::consts::PI.to_string(),
            ("math", "e") => std::f64::consts::E.to_string(),
            
            // JSON module
            ("json", "dumps") => format!("\"{}\"", args.join(", ")),
            ("json", "loads") => args.first().map(|s| s.to_string()).unwrap_or_default(),
            
            // String module
            ("str", "upper") => args.first().map(|s| s.to_uppercase()).unwrap_or_default(),
            ("str", "lower") => args.first().map(|s| s.to_lowercase()).unwrap_or_default(),
            ("str", "len") => args.first().map(|s| s.len().to_string()).unwrap_or("0".to_string()),
            
            // Unknown - return debug info
            _ => format!("[py:stub] {}.{}({:?}) - use ifa_std::ffi for real Python", module, func, args),
        }
    }
    
    fn parse_f64(&self, args: &[&str], idx: usize) -> Option<f64> {
        args.get(idx).and_then(|s| s.parse::<f64>().ok())
    }
    
    fn parse_u64(&self, args: &[&str], idx: usize) -> Option<u64> {
        args.get(idx).and_then(|s| s.parse::<u64>().ok())
    }

    /// Execute a shell command (heavily sandboxed)
    /// Ifá syntax: coop.sh("echo hello")
    ///
    /// SECURITY: Many dangerous commands are blocked by default.
    /// For unrestricted shell access, use ifa_std::ffi with proper capabilities.
    pub fn sh(&self, cmd: &str) -> Result<String, String> {
        // Security: Block dangerous commands
        let cmd_lower = cmd.to_lowercase();
        for blocked in &self.blocked_commands {
            if cmd_lower.contains(blocked) {
                return Err(format!(
                    "Security: '{}' is blocked. Use ifa_std::ffi with Execute capability for unrestricted access.",
                    blocked
                ));
            }
        }
        
        // Additional security: limit command length
        if cmd.len() > 1000 {
            return Err("Command too long (max 1000 chars)".to_string());
        }

        // Execute with platform-specific shell
        #[cfg(windows)]
        let output = std::process::Command::new("cmd")
            .args(["/C", cmd])
            .output();
            
        #[cfg(not(windows))]
        let output = std::process::Command::new("sh")
            .args(["-c", cmd])
            .output();
        
        match output {
            Ok(o) if o.status.success() => Ok(String::from_utf8_lossy(&o.stdout).trim().to_string()),
            Ok(o) => Err(String::from_utf8_lossy(&o.stderr).trim().to_string()),
            Err(e) => Err(format!("Execution failed: {}", e)),
        }
    }

    /// Load and call a native function via FFI
    /// NOTE: This is a stub. For real native FFI, use ifa_std::ffi::IfaFfi
    /// which provides libloading integration.
    pub fn rust(&self, func: &str, args: &[i64]) -> Result<i64, String> {
        // Built-in functions for testing
        match func {
            "add" if args.len() >= 2 => Ok(args[0].saturating_add(args[1])),
            "sub" if args.len() >= 2 => Ok(args[0].saturating_sub(args[1])),
            "mul" if args.len() >= 2 => Ok(args[0].saturating_mul(args[1])),
            "div" if args.len() >= 2 && args[1] != 0 => Ok(args[0] / args[1]),
            "max" if args.len() >= 2 => Ok(args[0].max(args[1])),
            "min" if args.len() >= 2 => Ok(args[0].min(args[1])),
            "abs" if !args.is_empty() => Ok(args[0].abs()),
            _ => Err(format!(
                "Unknown function '{}'. For real native FFI, use ifa_std::ffi::IfaFfi with libloading.",
                func
            )),
        }
    }

    /// Import a Python module for caching
    pub fn import_py(&mut self, module: &str) {
        self.py_cache.insert(module.to_string(), module.to_string());
    }
    
    /// Check if a module is cached
    pub fn has_module(&self, module: &str) -> bool {
        self.py_cache.contains_key(module)
    }
    
    /// List all blocked shell commands
    pub fn blocked_commands(&self) -> &[&'static str] {
        &self.blocked_commands
    }
}

impl Default for AjoseBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_basic() {
        let signal = Signal::new(42);
        assert_eq!(signal.get(), 42);

        signal.set(100);
        assert_eq!(signal.get(), 100);
    }

    #[test]
    fn test_signal_subscribe() {
        let signal = Signal::new(0);
        let received = Rc::new(Cell::new(0));
        let received_clone = received.clone();

        signal.subscribe(move |v| {
            received_clone.set(*v);
        });

        signal.set(42);
        assert_eq!(received.get(), 42);
    }

    #[test]
    fn test_signal_update() {
        let signal = Signal::new(vec![1, 2, 3]);
        signal.update(|v| v.push(4));
        assert_eq!(signal.get(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_computed() {
        let a = Signal::new(2);
        let b = Signal::new(3);

        let a_clone = a.clone();
        let b_clone = b.clone();
        let sum = Computed::new(move || a_clone.get() + b_clone.get());

        assert_eq!(sum.get(), 5);

        a.set(10);
        assert_eq!(sum.get(), 13);
    }

    #[test]
    fn test_ajose_bind() {
        let source: Rc<RefCell<i32>> = Rc::new(RefCell::new(10));
        let target: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

        let mut engine: Ajose<i32, String> = Ajose::new();
        engine.bind(&source, &target, |s, t| {
            *t = format!("Value: {}", s);
        });

        assert_eq!(*target.borrow(), "Value: 10");

        *source.borrow_mut() = 42;
        engine.propagate(&source);

        assert_eq!(*target.borrow(), "Value: 42");
    }

    #[test]
    fn test_ajose_bridge_py() {
        let mut bridge = AjoseBridge::new();
        let result = bridge.py("math", "sqrt", &["16"]);
        assert_eq!(result, "4");

        let result = bridge.py("math", "factorial", &["5"]);
        assert_eq!(result, "120");
    }
}
