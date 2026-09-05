use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};

/// Bacon-Rajan Cycle Collection Colors
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    White = 1,
    Gray = 2,
    Purple = 3,
    Red = 4, // Being dismantled
}

impl TryFrom<u8> for Color {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Color::Black),
            1 => Ok(Color::White),
            2 => Ok(Color::Gray),
            3 => Ok(Color::Purple),
            4 => Ok(Color::Red),
            _ => Err(()),
        }
    }
}

pub type TraceCallback<'a> = &'a mut dyn FnMut(NonNull<CycleHeader>);

/// Trait for objects that can be traced by the cycle collector.
pub trait Trace {
    fn trace(&self, cb: TraceCallback);
}

/// The header attached to every cycle-collected allocation
#[repr(C)]
pub struct CycleHeader {
    pub strong: AtomicUsize,
    pub tracing_rc: AtomicUsize,
    pub color: AtomicU8,
    pub buffered: AtomicBool,

    // Virtual method table replacements (Type Erasure)
    pub trace_fn: fn(NonNull<CycleHeader>, TraceCallback),
    pub drop_data_fn: fn(NonNull<CycleHeader>),
    pub dealloc_fn: fn(NonNull<CycleHeader>),
}

/// The node stored on the heap containing the header and the payload
#[repr(C)]
pub struct CycleNode<T> {
    pub header: CycleHeader,
    pub data: ManuallyDrop<T>,
}

thread_local! {
    static SUSPECT_BUFFER: RefCell<Vec<NonNull<CycleHeader>>> = RefCell::new(Vec::with_capacity(256));
}

pub fn add_to_suspect_buffer(ptr: NonNull<CycleHeader>) {
    SUSPECT_BUFFER.with(|buf| {
        buf.borrow_mut().push(ptr);
    });
}

/// Returns the current number of items in the suspect buffer for this thread.
pub fn suspect_count() -> usize {
    SUSPECT_BUFFER.with(|buf| buf.borrow().len())
}

/// A native cycle-collected reference-counted pointer.
/// Replaces `Arc<T>` for heap variants (`List`, `Map`, `Closure`) in the native VM.
///
/// Thread-safety is conditional on T. IfaGc<T> is Send + Sync when T: Send + Sync.
/// The PhantomData<*mut ()> opts out of auto-Send/Sync; the unsafe impls restore it
/// for thread-safe T. For VM-internal heap types (List, Map, Closure) where T is
/// !Send, IfaGc acts as !Send, providing per-actor isolation.
///
/// NOTE: The thread-local SUSPECT_BUFFER means cycle collection cannot reach cycles
/// that span actor thread boundaries. Do not send IfaGc pointers across actors
/// without proving acyclicity first (see: iso — not yet implemented).
pub struct IfaGc<T: Trace> {
    pub(crate) ptr: NonNull<CycleNode<T>>,
    _marker: PhantomData<*mut ()>, // Explicitly !Send and !Sync
}

// Removed unsafe impl Send/Sync for IfaGc
// IfaGc relies on a thread-local suspect buffer for Bacon-Rajan cycle collection.
// It is structurally unsafe to send it across threads. Actor boundaries must pass
// deep-copied values (IfaShared) instead of references.

impl<T: Trace> IfaGc<T> {
    /// Allocate a new object on the cycle-collected heap
    pub fn new(data: T) -> Self {
        let header = CycleHeader {
            strong: AtomicUsize::new(1),
            tracing_rc: AtomicUsize::new(0),
            color: AtomicU8::new(Color::Black as u8),
            buffered: AtomicBool::new(false),
            trace_fn: |ptr, cb| unsafe {
                let node = ptr.cast::<CycleNode<T>>().as_ref();
                node.data.trace(cb);
            },
            drop_data_fn: |ptr| unsafe {
                let mut node = ptr.cast::<CycleNode<T>>();
                ManuallyDrop::drop(&mut node.as_mut().data);
            },
            dealloc_fn: |ptr| unsafe {
                let node_ptr = ptr.cast::<CycleNode<T>>().as_ptr();
                let _ = Box::from_raw(node_ptr);
            },
        };

        let node = Box::new(CycleNode {
            header,
            data: ManuallyDrop::new(data),
        });

        let ptr = NonNull::new(Box::into_raw(node)).expect("Box allocation failed");

        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Access the cycle header
    pub fn header(&self) -> &CycleHeader {
        unsafe { &self.ptr.as_ref().header }
    }

    /// Check if two `IfaGc` point to the same allocation
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        std::ptr::eq(this.ptr.as_ptr(), other.ptr.as_ptr())
    }

    /// Provides a mutable reference to the data, cloning it if shared.
    pub fn make_mut(this: &mut Self) -> &mut T
    where
        T: Clone,
    {
        if this.header().strong.load(Ordering::Relaxed) != 1 {
            *this = IfaGc::new((**this).clone());
        }
        unsafe { &mut (*this.ptr.as_ptr()).data }
    }
}

impl<T: Trace> Clone for IfaGc<T> {
    fn clone(&self) -> Self {
        let header = self.header();
        header.strong.fetch_add(1, Ordering::Relaxed);
        header.color.store(Color::Black as u8, Ordering::Relaxed); // BACON RAJAN CLONE RULE
        Self {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T: Trace> Drop for IfaGc<T> {
    fn drop(&mut self) {
        let h = self.header();

        // If dismantling, do nothing to break the cycle recursively
        if Color::try_from(h.color.load(Ordering::Relaxed)).unwrap_or(Color::Black) == Color::Red {
            return;
        }

        let rc = h.strong.load(Ordering::Relaxed) - 1;
        h.strong.store(rc, Ordering::Relaxed);

        if rc == 0 {
            // FIX: When strong hits 0, the node is unreachable regardless of
            // buffered state. Previously, if buffered==true, we skipped the free
            // and the collector would also skip it (strong==0 → not marked gray,
            // removed from roots, never collected). That was a memory leak on
            // every cloned IfaGc where both refs drop.
            h.color.store(Color::Black as u8, Ordering::Relaxed);
            h.buffered.store(false, Ordering::Relaxed);
            (h.drop_data_fn)(self.ptr.cast());
            (h.dealloc_fn)(self.ptr.cast());
        } else if Color::try_from(h.color.load(Ordering::Relaxed)).unwrap_or(Color::Black)
            == Color::Black
        {
            h.color.store(Color::Purple as u8, Ordering::Relaxed);
            if !h.buffered.load(Ordering::Relaxed) {
                h.buffered.store(true, Ordering::Relaxed);
                add_to_suspect_buffer(self.ptr.cast());
            }
        }
    }
}

impl<T: Trace> Deref for IfaGc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.ptr.as_ref().data }
    }
}

impl<T: Trace> AsRef<T> for IfaGc<T> {
    fn as_ref(&self) -> &T {
        unsafe { &self.ptr.as_ref().data }
    }
}

impl<T: fmt::Debug + Trace> fmt::Debug for IfaGc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: PartialEq + Trace> PartialEq for IfaGc<T> {
    fn eq(&self, other: &Self) -> bool {
        // Fast path: exact same pointer
        if std::ptr::eq(self.ptr.as_ptr(), other.ptr.as_ptr()) {
            return true;
        }
        // Fallback: value equality
        **self == **other
    }
}

impl<T: Eq + Trace> Eq for IfaGc<T> {}

impl<T: Trace> From<T> for IfaGc<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

// =========================================================================
// BACON-RAJAN CYCLE COLLECTOR IMPLEMENTATION
// =========================================================================

/// Free a collected node: drop its data then deallocate the node memory.
/// Separated from collect_cycles so we can release the SUSPECT_BUFFER RefMut first.
unsafe fn free_red_node(s: NonNull<CycleHeader>) {
    let h = unsafe { s.as_ref() };
    (h.drop_data_fn)(s);
    (h.dealloc_fn)(s);
}

pub fn collect_cycles() -> usize {
    // FIX: Collect the list of reds first, then release the RefMut before
    // calling drop_data_fn. Previously, the RefMut was held across phases 4-5,
    // so any IfaGc::drop triggered by drop_data_fn would call
    // add_to_suspect_buffer → SUSPECT_BUFFER.borrow_mut() → panic.
    let reds: Vec<NonNull<CycleHeader>> = SUSPECT_BUFFER.with(|buf| {
        let mut roots = buf.borrow_mut();

        // 1. Mark Roots
        for i in 0..roots.len() {
            let s = roots[i];
            let h = unsafe { s.as_ref() };
            if Color::try_from(h.color.load(Ordering::Relaxed)).unwrap_or(Color::Black)
                == Color::Purple
                && h.strong.load(Ordering::Relaxed) > 0
            {
                mark_gray(s);
            } else {
                h.buffered.store(false, Ordering::Relaxed);
            }
        }
        roots.retain(|s| unsafe { s.as_ref() }.buffered.load(Ordering::Relaxed));

        // 2. Scan Roots
        for &s in roots.iter() {
            scan(s);
        }

        // 3. Collect White
        let mut reds = Vec::new();
        for &s in roots.iter() {
            let h = unsafe { s.as_ref() };
            h.buffered.store(false, Ordering::Relaxed);
            collect_white(s, &mut reds);
        }

        roots.clear();

        reds
    });

    // 4 & 5. Free Red nodes — RefMut is released, so IfaGc::drop can safely
    // call add_to_suspect_buffer without panicking.
    let count = reds.len();
    for &s in &reds {
        unsafe { free_red_node(s); }
    }

    count
}

// =========================================================================
// ITERATIVE MARK/SCAN/COLLECT (no recursion → no stack overflow)
// =========================================================================

fn mark_gray(root: NonNull<CycleHeader>) {
    let mut stack: Vec<NonNull<CycleHeader>> = vec![root];

    while let Some(s) = stack.pop() {
        let h = unsafe { s.as_ref() };
        if Color::try_from(h.color.load(Ordering::Relaxed)).unwrap_or(Color::Black) == Color::Gray {
            continue;
        }
        h.color.store(Color::Gray as u8, Ordering::Relaxed);
        h.tracing_rc
            .store(h.strong.load(Ordering::Relaxed), Ordering::Relaxed);

        // Collect children, decrement their tracing_rc, push onto stack
        let mut children = Vec::new();
        (h.trace_fn)(s, &mut |child| {
            children.push(child);
        });
        for child in children {
            let ch = unsafe { child.as_ref() };
            ch.tracing_rc.store(
                ch.tracing_rc.load(Ordering::Relaxed).saturating_sub(1),
                Ordering::Relaxed,
            );
            // Only push if not already gray (avoid redundant work)
            if Color::try_from(ch.color.load(Ordering::Relaxed)).unwrap_or(Color::Black)
                != Color::Gray
            {
                stack.push(child);
            }
        }
    }
}

fn scan(root: NonNull<CycleHeader>) {
    let mut stack: Vec<NonNull<CycleHeader>> = vec![root];

    while let Some(s) = stack.pop() {
        let h = unsafe { s.as_ref() };
        if Color::try_from(h.color.load(Ordering::Relaxed)).unwrap_or(Color::Black) != Color::Gray {
            continue;
        }
        if h.tracing_rc.load(Ordering::Relaxed) > 0 {
            scan_black(s);
        } else {
            h.color.store(Color::White as u8, Ordering::Relaxed);
            let mut children = Vec::new();
            (h.trace_fn)(s, &mut |child| {
                children.push(child);
            });
            for child in children {
                stack.push(child);
            }
        }
    }
}

fn scan_black(s: NonNull<CycleHeader>) {
    let mut stack: Vec<NonNull<CycleHeader>> = vec![s];

    while let Some(s) = stack.pop() {
        let h = unsafe { s.as_ref() };
        h.color.store(Color::Black as u8, Ordering::Relaxed);
        let mut children = Vec::new();
        (h.trace_fn)(s, &mut |child| {
            children.push(child);
        });
        for child in children {
            let ch = unsafe { child.as_ref() };
            if Color::try_from(ch.color.load(Ordering::Relaxed)).unwrap_or(Color::Black)
                != Color::Black
            {
                stack.push(child);
            }
        }
    }
}

fn collect_white(s: NonNull<CycleHeader>, reds: &mut Vec<NonNull<CycleHeader>>) {
    let mut stack: Vec<NonNull<CycleHeader>> = vec![s];

    while let Some(s) = stack.pop() {
        let h = unsafe { s.as_ref() };
        if Color::try_from(h.color.load(Ordering::Relaxed)).unwrap_or(Color::Black) == Color::White
            && !h.buffered.load(Ordering::Relaxed)
        {
            h.color.store(Color::Red as u8, Ordering::Relaxed);
            reds.push(s);
            let mut children = Vec::new();
            (h.trace_fn)(s, &mut |child| {
                children.push(child);
            });
            for child in children {
                stack.push(child);
            }
        }
    }
}
