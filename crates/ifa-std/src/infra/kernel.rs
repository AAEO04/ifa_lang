//! # Kernel Infrastructure
//!
//! System calls and Hardware information.

/// Hardware Information
pub struct SysInfo;

impl SysInfo {
    /// Get number of logical cores
    pub fn num_cores() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }

    /// Get system uptime (Placeholder)
    pub fn uptime() -> u64 {
        0 // TODO: Platform specific implementation
    }
}
