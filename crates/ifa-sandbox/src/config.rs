use crate::{Ofun, CapabilitySet};
use std::time::Duration;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityProfile {
    /// Maximum security: No disk/network, 5s timeout, 64MB RAM.
    Untrusted,
    /// Standard security: Read execution dir, no network, 30s timeout.
    Standard,
    /// Development: Full access (warn only), 5min timeout.
    Development,
    /// Custom: User defined
    Custom,
}

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub profile: SecurityProfile,
    pub capabilities: CapabilitySet,
    pub limits: ResourceLimits,
    pub use_os_isolation: bool,
    pub force_wasm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_execution_time: Duration,
    pub max_memory_bytes: usize,
    pub max_stack_depth: usize,
    pub max_file_descriptors: usize,
}

impl SandboxConfig {
    pub fn new(profile: SecurityProfile) -> Self {
        let limits = match profile {
            SecurityProfile::Untrusted => ResourceLimits {
                max_execution_time: Duration::from_secs(5),
                max_memory_bytes: 64 * 1024 * 1024,
                max_stack_depth: 100,
                max_file_descriptors: 3, // stdio only
            },
            SecurityProfile::Standard => ResourceLimits {
                max_execution_time: Duration::from_secs(30),
                max_memory_bytes: 256 * 1024 * 1024,
                max_stack_depth: 500,
                max_file_descriptors: 20,
            },
            SecurityProfile::Development => ResourceLimits {
                max_execution_time: Duration::from_secs(300),
                max_memory_bytes: 2 * 1024 * 1024 * 1024,
                max_stack_depth: 2000,
                max_file_descriptors: 1024,
            },
            SecurityProfile::Custom => ResourceLimits {
                max_execution_time: Duration::from_secs(30),
                max_memory_bytes: 256 * 1024 * 1024,
                max_stack_depth: 500,
                max_file_descriptors: 20,
            },
        };

        SandboxConfig {
            profile,
            capabilities: CapabilitySet::new(),
            limits,
            use_os_isolation: true,
            force_wasm: false,
        }
    }

    pub fn with_capability(mut self, cap: Ofun) -> Self {
        self.capabilities.grant(cap);
        self
    }
    
    pub fn force_wasm(mut self) -> Self {
        self.force_wasm = true;
        self
    }
}
