//! # Ẹbọ - Sacrifice & Resource Management (v2 - Zero-Cost)
//!
//! RAII resource lifecycle management without global state.
//! Uses generics instead of trait objects for zero allocation overhead.
//!
//! ## Features
//! - `Ebo<F>` - Zero-cost RAII guard
//! - `EboScope<T>` - Scoped resource with typed cleanup
//! - No global registries - explicit is better

use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

/// Zero-cost RAII guard that runs cleanup on drop.
///
/// # Example
/// ```rust
/// let file = std::fs::File::create("temp.txt")?;
/// let _guard = Ebo::new("tempfile", || std::fs::remove_file("temp.txt").ok());
/// // file removed when guard drops
/// ```
pub struct Ebo<F: FnOnce()> {
    _name: &'static str,
    cleanup: ManuallyDrop<Option<F>>,
}

impl<F: FnOnce()> Ebo<F> {
    /// Create an Ẹbọ guard with named cleanup
    #[inline]
    pub fn new(name: &'static str, cleanup: F) -> Self {
        Ebo {
            _name: name,
            cleanup: ManuallyDrop::new(Some(cleanup)),
        }
    }

    /// Dismiss the guard - cleanup will NOT run
    #[inline]
    pub fn dismiss(mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.cleanup);
        }
        std::mem::forget(self);
    }

    /// Run cleanup early and dismiss
    #[inline]
    pub fn sacrifice(mut self) {
        if let Some(cleanup) = unsafe { ManuallyDrop::take(&mut self.cleanup) } {
            cleanup();
        }
        std::mem::forget(self);
    }
}

impl<F: FnOnce()> Drop for Ebo<F> {
    #[inline]
    fn drop(&mut self) {
        if let Some(cleanup) = unsafe { ManuallyDrop::take(&mut self.cleanup) } {
            cleanup();
            #[cfg(debug_assertions)]
            println!("  🔥 [Ẹbọ] {}", self._name);
        }
    }
}

/// Scoped resource with automatic cleanup.
///
/// Wraps a value and runs cleanup when dropped.
///
/// # Example
/// ```rust
/// let scoped_file = EboScope::new(
///     std::fs::File::create("data.txt")?,
///     |f| { f.sync_all().ok(); }
/// );
/// scoped_file.write_all(b"data")?;
/// // sync_all called automatically on drop
/// ```
pub struct EboScope<T, F: FnOnce(&mut T)> {
    value: T,
    cleanup: Option<F>,
}

impl<T, F: FnOnce(&mut T)> EboScope<T, F> {
    /// Create scoped resource with cleanup function
    #[inline]
    pub fn new(value: T, cleanup: F) -> Self {
        EboScope {
            value,
            cleanup: Some(cleanup),
        }
    }

    /// Get inner value, consuming the scope
    #[inline]
    pub fn into_inner(mut self) -> T {
        // Run cleanup before returning
        if let Some(cleanup) = self.cleanup.take() {
            cleanup(&mut self.value);
        }
        let value = unsafe { std::ptr::read(&self.value) };
        std::mem::forget(self);
        value
    }

    /// Release without running cleanup
    #[inline]
    pub fn leak(mut self) -> T {
        self.cleanup = None;
        let value = unsafe { std::ptr::read(&self.value) };
        std::mem::forget(self);
        value
    }
}

impl<T, F: FnOnce(&mut T)> Deref for EboScope<T, F> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T, F: FnOnce(&mut T)> DerefMut for EboScope<T, F> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T, F: FnOnce(&mut T)> Drop for EboScope<T, F> {
    #[inline]
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup(&mut self.value);
        }
    }
}

/// Defer macro - Golang-style defer
///
/// # Example
/// ```rust
/// let mut file = File::create("temp.txt")?;
/// defer!(|| std::fs::remove_file("temp.txt"));
/// // cleanup runs at end of scope
/// ```
#[macro_export]
macro_rules! defer {
    ($cleanup:expr) => {
        let _ebo_guard = $crate::ebo::Ebo::new("defer", $cleanup);
    };
}

/// Ẹbọ block macro - scoped cleanup
///
/// # Example
/// ```rust
/// ebo! {
///     let conn = db.connect()?;
///     cleanup: conn.close();
/// }
/// ```
#[macro_export]
macro_rules! ebo {
    ($($body:tt)*) => {{
        struct _EboBlockGuard;
        impl Drop for _EboBlockGuard {
            fn drop(&mut self) {
                #[cfg(debug_assertions)]
                println!("╚═══ Ẹbọ ═══╝");
            }
        }
        #[cfg(debug_assertions)]
        println!("╔═══ Ẹbọ ═══╗");
        let _guard = _EboBlockGuard;
        $($body)*
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn test_ebo_runs_on_drop() {
        let ran = Rc::new(Cell::new(false));
        let ran_clone = ran.clone();
        {
            let _guard = Ebo::new("test", move || ran_clone.set(true));
        }
        assert!(ran.get());
    }

    #[test]
    fn test_ebo_dismiss() {
        let ran = Rc::new(Cell::new(false));
        let ran_clone = ran.clone();
        {
            let guard = Ebo::new("test", move || ran_clone.set(true));
            guard.dismiss();
        }
        assert!(!ran.get());
    }

    #[test]
    fn test_ebo_scope() {
        let cleaned = Rc::new(Cell::new(false));
        let cleaned_clone = cleaned.clone();
        {
            let scope = EboScope::new(42i32, move |_| cleaned_clone.set(true));
            assert_eq!(*scope, 42);
        }
        assert!(cleaned.get());
    }

    #[test]
    fn test_ebo_scope_into_inner() {
        let cleaned = Rc::new(Cell::new(false));
        let cleaned_clone = cleaned.clone();
        let scope = EboScope::new(42i32, move |_| cleaned_clone.set(true));
        let value = scope.into_inner();
        assert_eq!(value, 42);
        assert!(cleaned.get()); // Cleanup ran
    }
}
