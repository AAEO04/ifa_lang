//! # Àjọṣe - Reactive Relationship Engine (v2 - True Observables)
//!
//! Signal-based reactivity with proper observer pattern.
//! No raw callbacks - actual push-based updates.

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock, Weak};

// ============================================================================
// TYPE ALIASES for complex types (reduces clippy::type_complexity warnings)
// ============================================================================

/// Type alias for signal subscribers
type Subscribers<T> = Arc<RwLock<Vec<Box<dyn Fn(&T) + Send + Sync>>>>;

/// Type alias for reactive binding relationships  
#[allow(dead_code)]
type BindingRelation<S, T> = Vec<(
    Weak<RwLock<S>>,
    Weak<RwLock<T>>,
    Box<dyn Fn(&S, &mut T) + Send + Sync>,
)>;

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
    value: Arc<RwLock<T>>,
    subscribers: Subscribers<T>,
    version: Arc<AtomicU64>,
}

impl<T: Clone + Send + Sync + 'static> Signal<T> {
    pub fn new(initial: T) -> Self {
        Signal {
            value: Arc::new(RwLock::new(initial)),
            subscribers: Arc::new(RwLock::new(Vec::new())),
            version: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get current value
    pub fn get(&self) -> T {
        self.value.read().unwrap().clone()
    }

    /// Get reference to value
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.value.read().unwrap())
    }

    /// Set value and notify subscribers
    pub fn set(&self, new_value: T) {
        *self.value.write().unwrap() = new_value;
        self.version.fetch_add(1, Ordering::Relaxed);
        self.notify();
    }

    /// Update value with function
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.value.write().unwrap());
        self.version.fetch_add(1, Ordering::Relaxed);
        self.notify();
    }

    /// Subscribe to changes
    pub fn subscribe(&self, callback: impl Fn(&T) + Send + Sync + 'static) {
        self.subscribers.write().unwrap().push(Box::new(callback));
    }

    /// Get version number (for dirty checking)
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Relaxed)
    }

    fn notify(&self) {
        let value = self.value.read().unwrap();
        for sub in self.subscribers.read().unwrap().iter() {
            sub(&value);
        }
    }
}

impl<T: Clone + 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal {
            value: Arc::clone(&self.value),
            subscribers: Arc::clone(&self.subscribers),
            version: Arc::clone(&self.version),
        }
    }
}

impl<T: fmt::Debug + Clone + 'static> fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signal({:?})", self.value.read().unwrap())
    }
}

// ============================================================================
// COMPUTED - Derived reactive values
// ============================================================================

/// A computed value that auto-updates when dependencies change.
pub struct Computed<T> {
    value: Arc<RwLock<T>>,
    compute: Arc<dyn Fn() -> T + Send + Sync>,
}

impl<T: Clone + Send + Sync + 'static> Computed<T> {
    pub fn new<F: Fn() -> T + Send + Sync + 'static>(compute: F) -> Self {
        let value = compute();
        Computed {
            value: Arc::new(RwLock::new(value)),
            compute: Arc::new(compute),
        }
    }

    pub fn get(&self) -> T {
        // Recompute (in real impl, would track dependencies)
        let new_val = (self.compute)();
        *self.value.write().unwrap() = new_val;
        self.value.read().unwrap().clone()
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
pub fn effect<F: Fn() + Send + Sync + 'static>(f: F) -> EffectGuard {
    // Initial run
    f();

    // In a full implementation, we'd track which signals were accessed
    // and subscribe to them. For now, just store the callback.
    EffectGuard {
        callback: Arc::new(f),
    }
}

pub struct EffectGuard {
    callback: Arc<dyn Fn() + Send + Sync>,
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
    relationships: Vec<(
        Weak<RwLock<S>>,
        Weak<RwLock<T>>,
        Box<dyn Fn(&S, &mut T) + Send + Sync>,
    )>,
}

impl<S: Send + Sync + 'static, T: Send + Sync + 'static> Ajose<S, T> {
    pub fn new() -> Self {
        Ajose {
            relationships: Vec::new(),
        }
    }

    /// Bind source to target with transformation
    pub fn bind(
        &mut self,
        source: &Arc<RwLock<S>>,
        target: &Arc<RwLock<T>>,
        transform: impl Fn(&S, &mut T) + Send + Sync + 'static,
    ) {
        let source_weak = Arc::downgrade(source);
        let target_weak = Arc::downgrade(target);

        // Initial sync (must happen before we move transform into the Box)
        transform(&source.read().unwrap(), &mut target.write().unwrap());

        self.relationships
            .push((source_weak, target_weak, Box::new(transform)));
    }

    /// Propagate changes from source to targets
    pub fn propagate(&self, source: &Arc<RwLock<S>>) {
        for (src_weak, tgt_weak, transform) in &self.relationships {
            if let Some(src) = src_weak.upgrade() {
                if Arc::ptr_eq(&src, source) {
                    if let Some(tgt) = tgt_weak.upgrade() {
                        transform(&src.read().unwrap(), &mut tgt.write().unwrap());
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

impl<S: Send + Sync + 'static, T: Send + Sync + 'static> Default for Ajose<S, T> {
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
                "rm",
                "del",
                "format",
                "mkfs",
                "dd",
                "sudo",
                "chmod",
                "chown",
                "kill",
                "pkill",
                "shutdown",
                "reboot",
                "curl",
                "wget",
                "nc",
                "netcat",
                "powershell",
                "cmd.exe",
            ],
        }
    }

    /// Call a built-in Python-compatible math/utility function.
    ///
    /// Ifá syntax: `coop.py("math", "sqrt", 16)`
    ///
    /// Supported modules & functions:
    /// - `math`: sqrt, pow, sin, cos, tan, log, log10, exp, floor, ceil, abs, factorial, pi, e
    /// - `json`: dumps, loads (basic)
    /// - `str`: upper, lower, len
    ///
    /// Returns `Err` for any unknown module/function.
    /// For real Python interop, use `ifa_std::ffi::IfaFfi`.
    pub fn py(&mut self, module: &str, func: &str, args: &[&str]) -> Result<String, String> {
        let result = match (module, func) {
            // Math module
            ("math", "sqrt") => self
                .parse_f64(args, 0)
                .map(|n| n.sqrt())
                .unwrap_or(f64::NAN)
                .to_string(),
            ("math", "pow") => {
                let base = self.parse_f64(args, 0).unwrap_or(0.0);
                let exp = self.parse_f64(args, 1).unwrap_or(1.0);
                base.powf(exp).to_string()
            }
            ("math", "sin") => self
                .parse_f64(args, 0)
                .map(|n| n.sin())
                .unwrap_or(f64::NAN)
                .to_string(),
            ("math", "cos") => self
                .parse_f64(args, 0)
                .map(|n| n.cos())
                .unwrap_or(f64::NAN)
                .to_string(),
            ("math", "tan") => self
                .parse_f64(args, 0)
                .map(|n| n.tan())
                .unwrap_or(f64::NAN)
                .to_string(),
            ("math", "log") => self
                .parse_f64(args, 0)
                .map(|n| n.ln())
                .unwrap_or(f64::NAN)
                .to_string(),
            ("math", "log10") => self
                .parse_f64(args, 0)
                .map(|n| n.log10())
                .unwrap_or(f64::NAN)
                .to_string(),
            ("math", "exp") => self
                .parse_f64(args, 0)
                .map(|n| n.exp())
                .unwrap_or(f64::NAN)
                .to_string(),
            ("math", "floor") => self
                .parse_f64(args, 0)
                .map(|n| n.floor())
                .unwrap_or(f64::NAN)
                .to_string(),
            ("math", "ceil") => self
                .parse_f64(args, 0)
                .map(|n| n.ceil())
                .unwrap_or(f64::NAN)
                .to_string(),
            ("math", "abs") => self
                .parse_f64(args, 0)
                .map(|n| n.abs())
                .unwrap_or(f64::NAN)
                .to_string(),
            ("math", "factorial") => {
                if let Some(n) = self.parse_u64(args, 0) {
                    if n <= 20 {
                        return Ok((1..=n).product::<u64>().to_string());
                    }
                    return Err(
                        "factorial: argument too large (max 20 to avoid overflow)".to_string()
                    );
                }
                return Err("factorial: argument must be a non-negative integer".to_string());
            }
            ("math", "pi") => std::f64::consts::PI.to_string(),
            ("math", "e") => std::f64::consts::E.to_string(),

            // JSON module
            ("json", "dumps") => format!("\"{}\"", args.join(", ")),
            ("json", "loads") => args.first().map(|s| s.to_string()).unwrap_or_default(),

            // String module
            ("str", "upper") => args.first().map(|s| s.to_uppercase()).unwrap_or_default(),
            ("str", "lower") => args.first().map(|s| s.to_lowercase()).unwrap_or_default(),
            ("str", "len") => args
                .first()
                .map(|s| s.len().to_string())
                .unwrap_or("0".to_string()),

            // Unknown module or function — explicit error, never a stub string in user data
            _ => {
                return Err(format!(
                    "Unknown Python module or function: {}.{}. \
                Supported: math.{{sqrt,pow,sin,cos,tan,log,log10,exp,floor,ceil,abs,factorial,pi,e}}, \
                json.{{dumps,loads}}, str.{{upper,lower,len}}. \
                For real Python interop, use ifa_std::ffi::IfaFfi.",
                    module, func
                ));
            }
        };
        Ok(result)
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
        let _ = cmd;
        let _ = &self.blocked_commands;
        Err(
            "Security: coop.sh is disabled. Use an explicit capability-gated FFI path instead."
                .to_string(),
        )
    }

    /// Call a built-in integer math function.
    ///
    /// This does **not** load native shared libraries. It only exposes a small
    /// set of hardcoded arithmetic operations for testing and sandboxed environments.
    ///
    /// Supported: `add`, `sub`, `mul`, `div`, `max`, `min`, `abs`.
    ///
    /// For real dynamic native FFI (loading `.so`/`.dll` symbols at runtime),
    /// use `ifa_std::ffi::IfaFfi` which provides `libloading` integration.
    pub fn builtin_math(&self, func: &str, args: &[i64]) -> Result<i64, String> {
        match func {
            "add" if args.len() >= 2 => Ok(args[0].saturating_add(args[1])),
            "sub" if args.len() >= 2 => Ok(args[0].saturating_sub(args[1])),
            "mul" if args.len() >= 2 => Ok(args[0].saturating_mul(args[1])),
            "div" if args.len() >= 2 && args[1] != 0 => Ok(args[0] / args[1]),
            "div" if args.len() >= 2 => Err("div: division by zero".to_string()),
            "max" if args.len() >= 2 => Ok(args[0].max(args[1])),
            "min" if args.len() >= 2 => Ok(args[0].min(args[1])),
            "abs" if !args.is_empty() => Ok(args[0].abs()),
            _ => Err(format!(
                "builtin_math: unknown function '{}'. \
                Supported: add, sub, mul, div, max, min, abs. \
                For native FFI (loading .so/.dll symbols), use ifa_std::ffi::IfaFfi.",
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
        use std::sync::atomic::{AtomicI32, Ordering};
        let signal = Signal::new(0);
        let received = Arc::new(AtomicI32::new(0));
        let received_clone = received.clone();

        signal.subscribe(move |v| {
            received_clone.store(*v, Ordering::Relaxed);
        });

        signal.set(42);
        assert_eq!(received.load(Ordering::Relaxed), 42);
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
        let source: Arc<RwLock<i32>> = Arc::new(RwLock::new(10));
        let target: Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));

        let mut engine: Ajose<i32, String> = Ajose::new();
        engine.bind(&source, &target, |s, t| {
            *t = format!("Value: {}", s);
        });

        assert_eq!(*target.read().unwrap(), "Value: 10");

        *source.write().unwrap() = 42;
        engine.propagate(&source);

        assert_eq!(*target.read().unwrap(), "Value: 42");
    }

    #[test]
    fn test_ajose_bridge_py() {
        let mut bridge = AjoseBridge::new();
        let result = bridge.py("math", "sqrt", &["16"]).unwrap();
        assert_eq!(result, "4");

        let result = bridge.py("math", "factorial", &["5"]).unwrap();
        assert_eq!(result, "120");

        let err = bridge.py("numpy", "array", &["1", "2"]);
        assert!(err.is_err(), "unknown module must return Err");
    }
}
