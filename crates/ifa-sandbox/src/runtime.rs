use crate::{CapabilitySet, Ofun};
use std::fmt;

/// Error when a capability check fails
#[derive(Debug)]
pub struct CapabilityError {
    pub required: Ofun,
    pub call_site: String,
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Capability denied: {:?} at {}",
            self.required, self.call_site
        )
    }
}

impl std::error::Error for CapabilityError {}

/// Runtime context for native execution
pub struct NativeRuntime {
    capabilities: CapabilitySet,
}

impl NativeRuntime {
    pub fn new(capabilities: CapabilitySet) -> Self {
        NativeRuntime { capabilities }
    }

    /// Check if operation is allowed, return error if not
    pub fn check(&self, required: Ofun, call_site: &str) -> Result<(), CapabilityError> {
        if self.capabilities.check(&required) {
            Ok(())
        } else {
            Err(CapabilityError {
                required,
                call_site: call_site.to_string(),
            })
        }
    }
}
