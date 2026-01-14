use sysinfo::{System, Disks};
use std::env;

#[derive(Debug, Clone)]
pub struct SystemRequirements {
    pub os: String,
    pub arch: String,
    pub total_memory_gb: u64,
    pub available_disk_gb: u64,
}

pub fn check_system() -> SystemRequirements {
    let mut sys = System::new_all();
    sys.refresh_all(); // Still good for memory

    let total_memory_gb = sys.total_memory() / 1024 / 1024 / 1024;
    
    // Check disk space using new Disks API
    let disks = Disks::new_with_refreshed_list();
    let available_disk_gb = disks.list().first().map(|d| d.available_space() / 1024 / 1024 / 1024).unwrap_or(0);

    SystemRequirements {
        os: env::consts::OS.to_string(),
        arch: env::consts::ARCH.to_string(),
        total_memory_gb,
        available_disk_gb,
    }
}
