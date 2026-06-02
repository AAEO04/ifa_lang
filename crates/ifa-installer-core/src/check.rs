use std::env;
use std::path::Path;
use sysinfo::{Disks, System};

#[derive(Debug, Clone)]
pub struct SystemRequirements {
    pub os: String,
    pub arch: String,
    pub total_memory_gb: u64,
    pub available_disk_gb: u64,
    /// True if `rustc` is on PATH. Required for `ifa build` (transpiler → native).
    /// The VM/interpreter run without Rust installed.
    pub rustc_available: bool,
}

/// Check system requirements for the given install target directory.
/// Uses the disk that actually contains the target path, not necessarily disk #0.
pub fn check_system_for(install_dir: &Path) -> SystemRequirements {
    // Only refresh memory — we don't need CPU/process/network data.
    let mut sys = System::new();
    sys.refresh_memory();

    let total_memory_gb = sys.total_memory() / 1_073_741_824; // bytes → GiB

    // Find the disk whose mount point is the longest prefix of install_dir.
    // This correctly handles multi-disk systems where the install target is not on disk #0.
    let disks = Disks::new_with_refreshed_list();
    let available_disk_gb = disks
        .list()
        .iter()
        .filter(|d| install_dir.starts_with(d.mount_point()))
        .max_by_key(|d| d.mount_point().components().count())
        .or_else(|| disks.list().first())
        .map(|d| d.available_space() / 1_073_741_824)
        .unwrap_or(0);

    // Probe for rustc. Required for `ifa build` (transpiler output must be compiled by rustc).
    // The VM and interpreter do not require Rust on the device.
    let rustc_available = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    SystemRequirements {
        os: env::consts::OS.to_string(),
        arch: env::consts::ARCH.to_string(),
        total_memory_gb,
        available_disk_gb,
        rustc_available,
    }
}

/// Backwards-compatible wrapper — checks system without a specific install target.
pub fn check_system() -> SystemRequirements {
    check_system_for(Path::new("."))
}
