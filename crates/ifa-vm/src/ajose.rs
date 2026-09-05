//! # Àjọṣe - Reactive Relationship Engine (v2 - True Observables)
//!
//! Signal-based reactivity with proper observer pattern.
//! No raw callbacks - actual push-based updates.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock, Weak};

// ============================================================================
// TYPE ALIASES & OBSERVER CONTEXT
// ============================================================================

type SubscriberId = u64;

type ObserverCallback = Arc<dyn Fn() + Send + Sync + 'static>;

pub type CleanupFn = Box<dyn FnOnce() + Send + Sync>;
pub type EpochCleanups = Arc<Mutex<Vec<CleanupFn>>>;
type SubscribersMap<T> = Arc<RwLock<HashMap<SubscriberId, Box<dyn Fn(&T) + Send + Sync>>>>;

thread_local! {
    static ACTIVE_OBSERVER: RefCell<Option<ObserverCallback>> = const { RefCell::new(None) };
    pub static ACTIVE_EPOCH_CLEANUPS: RefCell<Option<EpochCleanups>> = const { RefCell::new(None) };
}

/// Guard representing an epoch cleanup association.
pub struct EpochCleanupGuard {
    prev: Option<EpochCleanups>,
}

impl EpochCleanupGuard {
    pub fn new(cleanups: EpochCleanups) -> Self {
        let prev = ACTIVE_EPOCH_CLEANUPS.with(|cell| cell.replace(Some(cleanups)));
        EpochCleanupGuard { prev }
    }
}

impl Drop for EpochCleanupGuard {
    fn drop(&mut self) {
        ACTIVE_EPOCH_CLEANUPS.with(|cell| {
            *cell.borrow_mut() = self.prev.take();
        });
    }
}

// Helper to register cleanups with the current active epoch if one is active.
fn register_subscription_cleanup(cleanup: Box<dyn FnOnce() + Send + Sync>) {
    ACTIVE_EPOCH_CLEANUPS.with(|cell| {
        if let Some(cleanups) = &*cell.borrow()
            && let Ok(mut c) = cleanups.lock()
        {
            c.push(cleanup);
        }
    });
}

/// Guard representing a subscription. Unsubscribes on drop.
pub struct SubscriptionGuard<T> {
    subscribers: SubscribersMap<T>,
    id: SubscriberId,
}

impl<T> Drop for SubscriptionGuard<T> {
    fn drop(&mut self) {
        if let Ok(mut subs) = self.subscribers.write() {
            subs.remove(&self.id);
        }
    }
}

// ============================================================================
// SIGNALS - Core reactive primitive
// ============================================================================

struct SignalInner<T> {
    value: RwLock<T>,
    subscribers: SubscribersMap<T>,
    version: AtomicU64,
    next_sub_id: AtomicU64,
}

pub struct Signal<T> {
    inner: Arc<SignalInner<T>>,
}

impl<T: Clone + Send + Sync + 'static> Signal<T> {
    pub fn new(initial: T) -> Self {
        Signal {
            inner: Arc::new(SignalInner {
                value: RwLock::new(initial),
                subscribers: Arc::new(RwLock::new(HashMap::new())),
                version: AtomicU64::new(0),
                next_sub_id: AtomicU64::new(0),
            }),
        }
    }

    /// Get current value and dynamically track dependency
    pub fn get(&self) -> T {
        if let Some(obs) = ACTIVE_OBSERVER.with(|cell| cell.borrow().clone()) {
            self.subscribe_internal(obs);
        }
        self.inner
            .value
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    fn subscribe_internal(&self, callback: ObserverCallback) {
        let id = Arc::as_ptr(&callback) as *const () as usize as u64;
        let mut subs = self
            .inner
            .subscribers
            .write()
            .unwrap_or_else(|e| e.into_inner());
        if let std::collections::hash_map::Entry::Vacant(e) = subs.entry(id) {
            let callback_clone = callback.clone();
            e.insert(Box::new(move |_| callback_clone()));

            // Register with Ebo Epoch cleanups to auto-unsubscribe when the epoch scope exits
            let subs_weak = Arc::downgrade(&self.inner.subscribers);
            register_subscription_cleanup(Box::new(move || {
                if let Some(subs_arc) = subs_weak.upgrade()
                    && let Ok(mut s) = subs_arc.write()
                {
                    s.remove(&id);
                }
            }));
        }
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.inner.value.read().unwrap_or_else(|e| e.into_inner()))
    }

    pub fn set(&self, new_value: T) {
        *self.inner.value.write().unwrap_or_else(|e| e.into_inner()) = new_value;
        self.inner.version.fetch_add(1, Ordering::Relaxed);
        self.notify();
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.inner.value.write().unwrap_or_else(|e| e.into_inner()));
        self.inner.version.fetch_add(1, Ordering::Relaxed);
        self.notify();
    }

    pub fn subscribe(&self, callback: impl Fn(&T) + Send + Sync + 'static) -> SubscriptionGuard<T> {
        let id = self.inner.next_sub_id.fetch_add(1, Ordering::Relaxed);
        self.inner
            .subscribers
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, Box::new(callback));
        SubscriptionGuard {
            subscribers: Arc::clone(&self.inner.subscribers),
            id,
        }
    }

    pub fn version(&self) -> u64 {
        self.inner.version.load(Ordering::Relaxed)
    }

    fn notify(&self) {
        let value = self.inner.value.read().unwrap_or_else(|e| e.into_inner());
        if let Ok(subs) = self.inner.subscribers.read() {
            for sub in subs.values() {
                sub(&value);
            }
        }
    }
}

impl<T: Clone + 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: fmt::Debug + Clone + 'static> fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Signal({:?})",
            self.inner.value.read().unwrap_or_else(|e| e.into_inner())
        )
    }
}

// ============================================================================
// COMPUTED - Derived reactive values
// ============================================================================

pub struct Computed<T> {
    value: Arc<RwLock<T>>,
    compute: Arc<dyn Fn() -> T + Send + Sync>,
}

impl<T: Clone + Send + Sync + 'static> Computed<T> {
    pub fn new<F: Fn() -> T + Send + Sync + 'static>(compute: F) -> Self {
        let compute_arc = Arc::new(compute);
        let compute_clone = compute_arc.clone();

        // Initial compute
        let initial_val = compute_clone();
        let value = Arc::new(RwLock::new(initial_val));
        let value_clone = value.clone();

        // Computed is backed by an effect that recalculates the cached value
        // when its dependencies (signals accessed inside compute) change.
        let effect_fn = move || {
            let next_val = compute_clone();
            *value_clone.write().unwrap_or_else(|e| e.into_inner()) = next_val;
        };

        effect(effect_fn);

        Computed {
            value,
            compute: compute_arc,
        }
    }

    pub fn get(&self) -> T {
        // If we are currently inside another reactive context (e.g. nested computed or effect),
        // we re-run compute to register dependencies. Otherwise, read cached value.
        if ACTIVE_OBSERVER.with(|cell| cell.borrow().is_some()) {
            let next_val = (self.compute)();
            *self.value.write().unwrap_or_else(|e| e.into_inner()) = next_val.clone();
            next_val
        } else {
            self.value.read().unwrap_or_else(|e| e.into_inner()).clone()
        }
    }
}

// ============================================================================
// EFFECT - Side effects on signal changes
// ============================================================================

pub fn effect<F: Fn() + Send + Sync + 'static>(f: F) -> EffectGuard {
    let f_arc = Arc::new(f);
    run_effect(f_arc.clone());
    EffectGuard { callback: f_arc }
}

fn run_effect(f: Arc<dyn Fn() + Send + Sync + 'static>) {
    let prev = ACTIVE_OBSERVER.with(|obs| obs.replace(Some(f.clone())));
    f();
    ACTIVE_OBSERVER.with(|obs| {
        *obs.borrow_mut() = prev;
    });
}

pub struct EffectGuard {
    callback: Arc<dyn Fn() + Send + Sync>,
}

impl EffectGuard {
    pub fn run(&self) {
        run_effect(self.callback.clone());
    }
}

// ============================================================================
// RELATIONSHIPS - Type-safe entity bindings
// ============================================================================

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

pub type AjoseRelationship<S, T> = (
    Weak<RwLock<S>>,
    Weak<RwLock<T>>,
    Box<dyn Fn(&S, &mut T) + Send + Sync>,
);

pub struct Ajose<S: 'static, T: 'static> {
    relationships: Vec<AjoseRelationship<S, T>>,
}

impl<S: Send + Sync + 'static, T: Send + Sync + 'static> Ajose<S, T> {
    pub fn new() -> Self {
        Ajose {
            relationships: Vec::new(),
        }
    }

    pub fn bind(
        &mut self,
        source: &Arc<RwLock<S>>,
        target: &Arc<RwLock<T>>,
        transform: impl Fn(&S, &mut T) + Send + Sync + 'static,
    ) {
        let source_weak = Arc::downgrade(source);
        let target_weak = Arc::downgrade(target);

        transform(
            &source.read().unwrap_or_else(|e| e.into_inner()),
            &mut target.write().unwrap_or_else(|e| e.into_inner()),
        );

        self.relationships
            .push((source_weak, target_weak, Box::new(transform)));
    }

    pub fn propagate(&self, source: &Arc<RwLock<S>>) {
        for (src_weak, tgt_weak, transform) in &self.relationships {
            if let Some(src) = src_weak.upgrade()
                && Arc::ptr_eq(&src, source)
                && let Some(tgt) = tgt_weak.upgrade()
            {
                transform(
                    &src.read().unwrap_or_else(|e| e.into_inner()),
                    &mut tgt.write().unwrap_or_else(|e| e.into_inner()),
                );
            }
        }
    }

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

// MACROS
#[macro_export]
macro_rules! bind {
    ($source:expr => $target:expr) => {{
        let source = $source.clone();
        let target = $target.clone();
        source.subscribe(move |val| {
            target.set(val.clone());
        })
    }};
    ($source:expr => $target:expr, |$v:ident| $transform:expr) => {{
        let source = $source.clone();
        let target = $target.clone();
        source.subscribe(move |$v| {
            target.set($transform);
        })
    }};
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

        let _guard = signal.subscribe(move |v| {
            received_clone.store(*v, Ordering::Relaxed);
        });

        signal.set(42);
        assert_eq!(received.load(Ordering::Relaxed), 42);
    }

    #[test]
    fn test_signal_unsubscribe() {
        use std::sync::atomic::{AtomicI32, Ordering};
        let signal = Signal::new(0);
        let c1 = Arc::new(AtomicI32::new(0));
        let c2 = Arc::new(AtomicI32::new(0));
        let c3 = Arc::new(AtomicI32::new(0));

        let c1_clone = c1.clone();
        let _g1 = signal.subscribe(move |v| c1_clone.store(*v, Ordering::Relaxed));
        let c2_clone = c2.clone();
        let g2 = signal.subscribe(move |v| c2_clone.store(*v, Ordering::Relaxed));
        let c3_clone = c3.clone();
        let _g3 = signal.subscribe(move |v| c3_clone.store(*v, Ordering::Relaxed));

        // Drop second guard
        std::mem::drop(g2);

        signal.set(100);
        assert_eq!(c1.load(Ordering::Relaxed), 100);
        assert_eq!(c2.load(Ordering::Relaxed), 0);
        assert_eq!(c3.load(Ordering::Relaxed), 100);
    }

    #[test]
    fn test_signal_subscribe_immediate_drop() {
        use std::sync::atomic::{AtomicI32, Ordering};
        let signal = Signal::new(0);
        let received = Arc::new(AtomicI32::new(0));
        let received_clone = received.clone();

        // Subscribe and immediately drop
        let _ = signal.subscribe(move |v| received_clone.store(*v, Ordering::Relaxed));

        signal.set(42);
        assert_eq!(received.load(Ordering::Relaxed), 0);
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
}
