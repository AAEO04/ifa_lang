//! # Ọ̀yẹ̀kú Domain (0000)
//!
//! The Sleeper - Exit and Sleep Operations
//!
//! RAII-based cleanup with precise sleep using tokio::time.

use crate::impl_odu_domain;
use std::process;
use std::thread;
use std::time::Duration;

use ifa_sandbox::{CapabilitySet, Ofun};

/// Ọ̀yẹ̀kú - The Sleeper (Exit/Sleep)
#[derive(Default)]
pub struct Oyeku {
    capabilities: CapabilitySet,
}

impl_odu_domain!(Oyeku, "Ọ̀yẹ̀kú", "0000", "The Sleeper - Exit/Sleep");

impl Oyeku {
    pub fn new(capabilities: CapabilitySet) -> Self {
        Oyeku { capabilities }
    }

    /// Exit program with code (kú)
    pub fn ku(&self, code: i32) -> ! {
        process::exit(code)
    }

    /// Exit successfully
    pub fn ku_daadaa(&self) -> ! {
        process::exit(0)
    }

    /// Exit with error
    pub fn ku_buruku(&self) -> ! {
        process::exit(1)
    }

    /// Sleep for milliseconds (sùn)
    pub fn sun_ms(&self, ms: u64) {
        if self.capabilities.check(&Ofun::Time) {
            thread::sleep(Duration::from_millis(ms));
        }
    }

    /// Sleep for seconds
    pub fn sun(&self, seconds: f64) {
        if self.capabilities.check(&Ofun::Time) {
            thread::sleep(Duration::from_secs_f64(seconds));
        }
    }

    /// Wait/pause (dúró)
    pub fn duro(&self, ms: u64) {
        self.sun_ms(ms);
    }

    /// Abort immediately (no cleanup)
    pub fn da_duro(&self) -> ! {
        process::abort()
    }
}

/// RAII guard for cleanup on scope exit
pub struct Ebo<F: FnOnce()> {
    cleanup: Option<F>,
}

impl<F: FnOnce()> Ebo<F> {
    /// Create new Ẹbọ (sacrifice) - cleanup guard
    pub fn new(cleanup: F) -> Self {
        Ebo {
            cleanup: Some(cleanup),
        }
    }

    /// Dismiss the cleanup (don't run on drop)
    pub fn dismiss(mut self) {
        self.cleanup = None;
    }
}

impl<F: FnOnce()> Drop for Ebo<F> {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

/// Async sleep (for tokio runtime)
#[cfg(feature = "full")]
pub async fn sun_async(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

/// Precise sleep with spin loop for sub-millisecond accuracy
pub fn sun_precise(nanos: u64) {
    let start = std::time::Instant::now();
    let duration = Duration::from_nanos(nanos);

    // Sleep for most of the time
    if nanos > 1_000_000 {
        thread::sleep(Duration::from_nanos(nanos - 1_000_000));
    }

    // Spin for remaining time (precise)
    while start.elapsed() < duration {
        std::hint::spin_loop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_sleep() {
        // Grant Time capability for sleep
        let mut caps = CapabilitySet::default();
        caps.grant(Ofun::Time);
        let oyeku = Oyeku::new(caps);

        let start = Instant::now();
        oyeku.sun_ms(50);
        let elapsed = start.elapsed();
        // Relaxed timing to account for system variance
        assert!(elapsed.as_millis() >= 40);
    }

    #[test]
    fn test_ebo_cleanup() {
        use std::cell::Cell;
        let cleaned = Cell::new(false);

        {
            let _guard = Ebo::new(|| cleaned.set(true));
        }

        assert!(cleaned.get());
    }

    #[test]
    fn test_ebo_dismiss() {
        use std::cell::Cell;
        let cleaned = Cell::new(false);

        {
            let guard = Ebo::new(|| cleaned.set(true));
            guard.dismiss();
        }

        assert!(!cleaned.get());
    }
}
