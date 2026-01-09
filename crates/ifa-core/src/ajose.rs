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
/// Uses the low-level FFI from ifa-std.
pub struct AjoseBridge {
    /// Python modules (simulated - full impl needs pyo3)
    py_cache: std::collections::HashMap<String, String>,
}

impl AjoseBridge {
    pub fn new() -> Self {
        AjoseBridge {
            py_cache: std::collections::HashMap::new(),
        }
    }

    /// Call a Python function
    /// Ifá syntax: coop.py("math", "sqrt", 16)
    pub fn py(&mut self, module: &str, func: &str, args: &[&str]) -> String {
        // In production, this would use pyo3
        // For now, simulate common math functions
        match (module, func) {
            ("math", "sqrt") => {
                if let Some(arg) = args.first() {
                    if let Ok(n) = arg.parse::<f64>() {
                        return n.sqrt().to_string();
                    }
                }
                "NaN".to_string()
            }
            ("math", "factorial") => {
                if let Some(arg) = args.first() {
                    if let Ok(n) = arg.parse::<u64>() {
                        let result: u64 = (1..=n).product();
                        return result.to_string();
                    }
                }
                "1".to_string()
            }
            ("json", "dumps") => {
                format!("\"{}\"", args.join(", "))
            }
            _ => format!("[py] {}.{}({:?})", module, func, args),
        }
    }

    /// Execute a shell command (sandboxed)
    /// Ifá syntax: coop.sh("echo hello")
    pub fn sh(&self, cmd: &str) -> Result<String, String> {
        // Block dangerous commands
        let blocked = ["rm", "del", "format", "mkfs", "dd", "sudo"];
        for b in blocked {
            if cmd.contains(b) {
                return Err(format!("Blocked command: {}", b));
            }
        }

        // Execute (in production, use proper sandboxing)
        #[cfg(windows)]
        {
            std::process::Command::new("cmd")
                .args(["/C", cmd])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .map_err(|e| e.to_string())
        }
        #[cfg(not(windows))]
        {
            std::process::Command::new("sh")
                .args(["-c", cmd])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .map_err(|e| e.to_string())
        }
    }

    /// Load and call a Rust function via FFI
    /// Ifá syntax: coop.rust("add", [5, 3])
    pub fn rust(&self, _func: &str, _args: &[i64]) -> i64 {
        // Placeholder - would use dlopen/dlsym in production
        0
    }

    /// Import a Python module for caching
    pub fn import_py(&mut self, module: &str) {
        self.py_cache.insert(module.to_string(), module.to_string());
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
