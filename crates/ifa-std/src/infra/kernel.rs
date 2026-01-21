//! # Kernel Infrastructure
//!
//! System calls and hardware information.

/// Get number of logical CPU cores
pub fn num_cores() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

/// Cached system info for expensive sysinfo calls
#[cfg(feature = "sysinfo")]
static SYSTEM_INFO: std::sync::OnceLock<sysinfo::System> = std::sync::OnceLock::new();

#[cfg(feature = "sysinfo")]
fn get_system() -> &'static sysinfo::System {
    SYSTEM_INFO.get_or_init(|| {
        use sysinfo::System;
        System::new_all()
    })
}

/// Get system uptime in seconds (cross-platform via sysinfo crate)
#[cfg(feature = "sysinfo")]
pub fn uptime() -> u64 {
    sysinfo::System::uptime()
}

/// Get total system memory in bytes (cached)
#[cfg(feature = "sysinfo")]
pub fn total_memory() -> u64 {
    get_system().total_memory()
}

/// Get available system memory in bytes (refreshes on each call)
#[cfg(feature = "sysinfo")]
pub fn available_memory() -> u64 {
    // Note: For truly fresh data, we refresh. For the cached static,
    // we'd need RefCell/Mutex. For now, create a fresh System for this call.
    use sysinfo::System;
    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.available_memory()
}

/// Refresh and get memory stats (for when you need fresh values)
#[cfg(feature = "sysinfo")]
pub fn memory_stats() -> MemoryStats {
    use sysinfo::System;
    let mut sys = System::new_all();
    sys.refresh_memory();
    MemoryStats {
        total: sys.total_memory(),
        available: sys.available_memory(),
        used: sys.used_memory(),
    }
}

/// Memory statistics snapshot
#[cfg(feature = "sysinfo")]
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub total: u64,
    pub available: u64,
    pub used: u64,
}
