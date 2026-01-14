//! # Ọ̀gbè Domain (1111)
//!
//! The Initiator - System and Lifecycle
//!
//! CLI arguments, environment, and program initialization.

use crate::impl_odu_domain;
use std::env;

use ifa_sandbox::{CapabilitySet, Ofun};

/// Ọ̀gbè - The Initiator (System/Lifecycle)
#[derive(Default)]
pub struct Ogbe {
    capabilities: CapabilitySet,
}

impl_odu_domain!(Ogbe, "Ọ̀gbè", "1111", "The Initiator - System/Lifecycle");

impl Ogbe {
    pub fn new(capabilities: CapabilitySet) -> Self {
        Ogbe { capabilities }
    }

    /// Check env capability
    fn check_env(&self, key: &str) -> bool {
        self.capabilities.check(&Ofun::Environment {
            keys: vec![key.to_string()],
        })
    }

    /// Get CLI arguments (àwọn ohun)
    pub fn awon_ohun(&self) -> Vec<String> {
        env::args().collect()
    }

    /// Get argument at index
    pub fn ohun(&self, index: usize) -> Option<String> {
        env::args().nth(index)
    }

    /// Get argument count
    pub fn iye_ohun(&self) -> usize {
        env::args().count()
    }

    /// Get environment variable (àyíká)
    pub fn ayika(&self, key: &str) -> Option<String> {
        if !self.check_env(key) {
            return None;
        }
        env::var(key).ok()
    }

    /// Get env with default
    pub fn ayika_tabi(&self, key: &str, default: &str) -> String {
        if !self.check_env(key) {
            return default.to_string();
        }
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    /// Set environment variable (fí àyíká)
    /// 
    /// WARNING: This operation is deprecated and does nothing.
    /// Setting environment variables at runtime is thread-unsafe and can cause
    /// data races. Use shell environment or config files instead.
    #[deprecated(since = "1.2.1", note = "Environment variable mutation is thread-unsafe. Use shell environment instead.")]
    pub fn fi_ayika(&self, key: &str, value: &str) {
        if self.check_env(key) {
            eprintln!(
                "[WARN] fi_ayika('{}', '{}') ignored: Setting env vars at runtime is unsafe. \
                 Use shell exports or config files instead.",
                key, value
            );
        }
    }

    /// Get home directory (ilé)
    pub fn ile(&self) -> Option<String> {
        if !self.check_env("HOME") && !self.check_env("USERPROFILE") {
            return None;
        }
        env::var("HOME").or_else(|_| env::var("USERPROFILE")).ok()
    }

    /// Get current working directory (ojú ọ̀nà)
    pub fn oju_ona(&self) -> Option<String> {
        // CWD isn't strictly an ENV var check but good to limit
        env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(String::from))
    }

    /// Get OS name
    pub fn eto(&self) -> &'static str {
        env::consts::OS
    }

    /// Get architecture
    pub fn apẹrẹ(&self) -> &'static str {
        env::consts::ARCH
    }
}

/// Init hook trait for lifecycle management
pub trait InitHook {
    /// Called when program starts
    fn on_init(&self) {}

    /// Called when program exits (if possible)
    fn on_exit(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ogbe() {
        let ogbe = Ogbe::default();
        assert!(!ogbe.eto().is_empty());
        assert!(!ogbe.apẹrẹ().is_empty());
    }
}
