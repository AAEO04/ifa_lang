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
/// Cached system info with thread-safe mutation
#[cfg(feature = "sysinfo")]
static SYSTEM_INFO: std::sync::OnceLock<std::sync::RwLock<sysinfo::System>> =
    std::sync::OnceLock::new();

#[cfg(feature = "sysinfo")]
fn get_system() -> &'static std::sync::RwLock<sysinfo::System> {
    SYSTEM_INFO.get_or_init(|| {
        use sysinfo::System;
        std::sync::RwLock::new(System::new_all())
    })
}

/// Get system uptime in seconds
#[cfg(feature = "sysinfo")]
pub fn uptime() -> u64 {
    sysinfo::System::uptime()
}

/// Get total system memory in bytes (cached)
#[cfg(feature = "sysinfo")]
pub fn total_memory() -> u64 {
    get_system().read().unwrap().total_memory()
}

/// Get available system memory in bytes (refreshes memory stats only)
#[cfg(feature = "sysinfo")]
pub fn available_memory() -> u64 {
    let sys_lock = get_system();
    // Write lock to refresh
    if let Ok(mut sys) = sys_lock.write() {
        sys.refresh_memory();
        sys.available_memory()
    } else {
        0 // Fallback on lock poisoning
    }
}

/// Refresh and get memory stats
#[cfg(feature = "sysinfo")]
pub fn memory_stats() -> MemoryStats {
    let sys_lock = get_system();
    if let Ok(mut sys) = sys_lock.write() {
        sys.refresh_memory();
        MemoryStats {
            total: sys.total_memory(),
            available: sys.available_memory(),
            used: sys.used_memory(),
        }
    } else {
        MemoryStats {
            total: 0,
            available: 0,
            used: 0,
        }
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
