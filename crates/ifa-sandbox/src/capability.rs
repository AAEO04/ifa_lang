use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Ọ̀fún Capability Definition
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Ofun {
    /// Read access to filesystem path
    ReadFiles { root: PathBuf },
    /// Write access to filesystem path
    WriteFiles { root: PathBuf },
    /// Network access to specific domains
    Network { domains: Vec<String> },
    /// Execute subprocesses
    Execute { programs: Vec<String> },
    /// Access environment variables
    Environment { keys: Vec<String> },
    /// High-resolution time
    Time,
    /// Random number generation
    Random,
    /// Standard I/O (stdin/stdout/stderr)
    Stdio,
}

/// A set of granted capabilities
#[derive(Debug, Clone, Default)]
pub struct CapabilitySet {
    capabilities: Vec<Ofun>,
    violations: Vec<CapabilityViolation>,
}

#[derive(Debug, Clone)]
pub struct CapabilityViolation {
    pub capability: Ofun,
    pub call_site: String,
    pub timestamp: String,
}

impl CapabilitySet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grant(&mut self, cap: Ofun) {
        self.capabilities.push(cap);
    }

    /// Check if an operation is allowed
    pub fn check(&self, required: &Ofun) -> bool {
        self.capabilities
            .iter()
            .any(|granted| match (granted, required) {
                (Ofun::ReadFiles { root: g }, Ofun::ReadFiles { root: r }) => r.starts_with(g),
                (Ofun::WriteFiles { root: g }, Ofun::WriteFiles { root: r }) => r.starts_with(g),
                (Ofun::Network { domains: g }, Ofun::Network { domains: r }) => {
                    // Simple exact match for now, could add globbing
                    r.iter().all(|d| g.contains(d))
                }
                (Ofun::Environment { keys: g }, Ofun::Environment { keys: r }) => {
                    r.iter().all(|k| g.contains(k))
                }
                (Ofun::Execute { programs: g }, Ofun::Execute { programs: r }) => {
                    r.iter().all(|p| g.contains(p))
                }
                (Ofun::Time, Ofun::Time) => true,
                (Ofun::Random, Ofun::Random) => true,
                (Ofun::Stdio, Ofun::Stdio) => true,
                _ => false,
            })
    }

    /// Get all granted capabilities
    pub fn all(&self) -> &[Ofun] {
        &self.capabilities
    }

    /// Get recorded violations (for audit/debugging)
    pub fn violations(&self) -> &[CapabilityViolation] {
        &self.violations
    }

    /// Record a capability violation
    pub fn record_violation(&mut self, cap: Ofun, call_site: &str) {
        self.violations.push(CapabilityViolation {
            capability: cap,
            call_site: call_site.to_string(),
            timestamp: format!("{:?}", std::time::SystemTime::now()),
        });
    }
}
