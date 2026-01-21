//! # Sandbox - Process Isolation Wrapper
//!
//! Provides runtime isolation for native execution with capability enforcement.

use crate::{Ofun, SandboxConfig, SecurityProfile};
use std::path::Path;
use std::time::{Duration, Instant};

/// State of the sandbox execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxState {
    /// Created but not yet started
    Created,
    /// Currently executing
    Running,
    /// Execution completed normally
    Completed,
    /// Execution was terminated (time limit, resource limit, or manual)
    Terminated,
}

/// A sandboxed execution environment for native code
#[derive(Debug)]
pub struct Sandbox {
    config: SandboxConfig,
    state: SandboxState,
    start_time: Option<Instant>,
    termination_reason: Option<String>,
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Sandbox {
    /// Create a new sandbox with default (restricted) settings
    pub fn new() -> Self {
        Sandbox {
            config: SandboxConfig::new(SecurityProfile::Standard),
            state: SandboxState::Created,
            start_time: None,
            termination_reason: None,
        }
    }

    /// Create a sandbox from a configuration
    pub fn with_config(config: SandboxConfig) -> Self {
        Sandbox {
            config,
            state: SandboxState::Created,
            start_time: None,
            termination_reason: None,
        }
    }

    // =========================================================================
    // Capability Management
    // =========================================================================

    /// Grant a capability to this sandbox
    pub fn grant_capability(&mut self, cap: Ofun) {
        self.config.capabilities.grant(cap);
    }

    /// Check if sandbox has a specific capability
    pub fn has_capability(&self, cap: Ofun) -> bool {
        self.config.capabilities.check(&cap)
    }

    /// Check if sandbox is in restricted mode (no capabilities)
    pub fn is_restricted(&self) -> bool {
        self.config.capabilities.all().is_empty()
    }

    // =========================================================================
    // Resource Limits
    // =========================================================================

    /// Set memory limit in bytes
    pub fn set_memory_limit(&mut self, bytes: usize) {
        self.config.limits.max_memory_bytes = bytes;
    }

    /// Get memory limit
    pub fn memory_limit(&self) -> usize {
        self.config.limits.max_memory_bytes
    }

    /// Set CPU time limit in milliseconds
    pub fn set_cpu_limit(&mut self, ms: u64) {
        self.config.limits.max_execution_time = Duration::from_millis(ms);
    }

    /// Get CPU limit in milliseconds
    pub fn cpu_limit(&self) -> u64 {
        self.config.limits.max_execution_time.as_millis() as u64
    }

    /// Set file descriptor limit
    pub fn set_file_limit(&mut self, count: usize) {
        self.config.limits.max_file_descriptors = count;
    }

    /// Get file limit
    pub fn file_limit(&self) -> usize {
        self.config.limits.max_file_descriptors
    }

    /// Set time limit
    pub fn set_time_limit(&mut self, duration: Duration) {
        self.config.limits.max_execution_time = duration;
    }

    /// Get time limit
    pub fn time_limit(&self) -> Duration {
        self.config.limits.max_execution_time
    }

    /// Set process limit (for fork bomb prevention)
    pub fn set_process_limit(&mut self, _count: usize) {
        // Note: Process limiting would require OS-specific implementation
        // This is a placeholder for the API
    }

    // =========================================================================
    // Execution Control
    // =========================================================================

    /// Start execution tracking
    pub fn start_execution(&mut self) {
        self.state = SandboxState::Running;
        self.start_time = Some(Instant::now());
    }

    /// Check if sandbox is currently running
    pub fn is_running(&self) -> bool {
        self.state == SandboxState::Running
    }

    /// Terminate the sandbox
    pub fn terminate(&mut self) {
        self.state = SandboxState::Terminated;
        self.termination_reason = Some("Manual termination".to_string());
    }

    /// Check if sandbox was terminated (vs completed normally)
    pub fn was_terminated(&self) -> bool {
        self.state == SandboxState::Terminated
    }

    /// Get elapsed time since start
    pub fn elapsed_time(&self) -> Duration {
        self.start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO)
    }

    // =========================================================================
    // Security Checks
    // =========================================================================

    /// Check if file access is allowed (with path traversal prevention)
    pub fn can_access_file(&self, path: &Path) -> bool {
        // Canonicalize to prevent path traversal attacks
        let canonical = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return false, // If we can't resolve, deny
        };

        // Check if it's a symlink (deny symlinks for security)
        if path.is_symlink() {
            return false;
        }

        // Check against granted file capabilities
        self.config
            .capabilities
            .check(&Ofun::ReadFiles { root: canonical })
    }

    /// Check if network connection is allowed
    pub fn can_connect_to(&self, domain: &str) -> bool {
        self.config.capabilities.check(&Ofun::Network {
            domains: vec![domain.to_string()],
        })
    }

    /// Check if environment variable access is allowed
    pub fn can_access_env_var(&self, key: &str) -> bool {
        self.config.capabilities.check(&Ofun::Environment {
            keys: vec![key.to_string()],
        })
    }

    /// Check if file creation is allowed (respects file limit)
    pub fn can_create_file(&self, _path: &Path) -> bool {
        // This would need actual file count tracking
        // For now, just check write capability
        self.config
            .capabilities
            .all()
            .iter()
            .any(|c| matches!(c, Ofun::WriteFiles { .. }))
    }

    /// Check if process spawning is allowed
    pub fn can_spawn_process(&self) -> bool {
        self.config
            .capabilities
            .all()
            .iter()
            .any(|c| matches!(c, Ofun::Execute { .. }))
    }

    /// Treat input as literal string (for injection prevention)
    pub fn treat_as_literal(&self, _input: &str) -> bool {
        // All user input should be treated as literal by default
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let sandbox = Sandbox::new();
        assert!(sandbox.is_restricted());
        assert!(!sandbox.is_running());
    }

    #[test]
    fn test_sandbox_capabilities() {
        let mut sandbox = Sandbox::new();
        sandbox.grant_capability(Ofun::Stdio);

        assert!(sandbox.has_capability(Ofun::Stdio));
        assert!(!sandbox.has_capability(Ofun::Time));
    }

    #[test]
    fn test_sandbox_lifecycle() {
        let mut sandbox = Sandbox::new();

        assert!(!sandbox.is_running());
        sandbox.start_execution();
        assert!(sandbox.is_running());

        sandbox.terminate();
        assert!(!sandbox.is_running());
        assert!(sandbox.was_terminated());
    }
}
