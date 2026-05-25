//! # Èṣù Middleware
//!
//! Capability enforcement layer for storage and networking domains.
//!
//! Òfún still defines the capability vocabulary; Èṣù owns the live world state
//! and enforces access at the crossroads before I/O proceeds.

use crate::sandbox_shim::{CapabilitySet, Ofun};
use ifa_vm::error::{IfaError, IfaResult};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct Esu {
    world_state: Arc<Mutex<CapabilitySet>>,
}

impl Esu {
    pub fn new(capabilities: CapabilitySet) -> Self {
        Self {
            world_state: Arc::new(Mutex::new(capabilities)),
        }
    }

    pub fn from_world(capabilities: CapabilitySet) -> Self {
        Self::new(capabilities)
    }

    pub fn world_state(&self) -> CapabilitySet {
        match self.world_state.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    pub fn enforce_crossroads(&self, requested_cap: &Ofun, call_site: &str) -> IfaResult<()> {
        let guard = match self.world_state.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        
        if guard.check(requested_cap) {
            Ok(())
        } else {
            Err(IfaError::PermissionDenied(format!(
                "Èṣù blocked '{}' at the crossroads",
                call_site
            )))
        }
    }

    pub fn ju(&self, cap: Ofun) {
        let mut guard = match self.world_state.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.revoke(&cap);
    }

    pub fn grant(&self, cap: Ofun) {
        let mut guard = match self.world_state.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.grant(cap);
    }

    pub fn allows(&self, requested_cap: &Ofun) -> bool {
        let guard = match self.world_state.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.check(requested_cap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox_shim::CapabilitySet;

    #[test]
    fn test_enforce_crossroads_blocks_missing_capability() {
        let esu = Esu::new(CapabilitySet::default());

        let err = esu
            .enforce_crossroads(
                &Ofun::ReadFiles {
                    root: std::path::PathBuf::from("/tmp"),
                },
                "test",
            )
            .expect_err("missing capability should be blocked");

        assert!(matches!(err, ifa_vm::error::IfaError::PermissionDenied(_)));
    }

    #[test]
    fn test_revocation_updates_shared_world_state() {
        let mut caps = CapabilitySet::default();
        caps.grant(Ofun::WriteFiles {
            root: std::path::PathBuf::from("/tmp"),
        });

        let esu = Esu::new(caps);
        assert!(esu.allows(&Ofun::WriteFiles {
            root: std::path::PathBuf::from("/tmp"),
        }));

        esu.ju(Ofun::WriteFiles {
            root: std::path::PathBuf::from("/tmp"),
        });

        assert!(!esu.allows(&Ofun::WriteFiles {
            root: std::path::PathBuf::from("/tmp"),
        }));
    }
}
