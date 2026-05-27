//! # ResourceMonitor - Runtime Resource Tracking
//!
//! Monitors memory, CPU, file descriptors, and network usage during execution.

use std::cell::Cell;
use std::time::{Duration, Instant};
use sysinfo::{ProcessRefreshKind, System};

/// Tracks resource usage during sandbox execution
#[derive(Debug)]
pub struct ResourceMonitor {
    start_time: Option<Instant>,
    peak_memory: Cell<usize>,
    current_memory: Cell<usize>,
    cpu_time: Duration,
    file_count: usize,
    bytes_sent: u64,
    bytes_received: u64,
    running: bool,
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new() -> Self {
        ResourceMonitor {
            start_time: None,
            peak_memory: Cell::new(0),
            current_memory: Cell::new(0),
            cpu_time: Duration::ZERO,
            file_count: 0,
            bytes_sent: 0,
            bytes_received: 0,
            running: false,
        }
    }

    /// Start monitoring
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.running = true;

        // Get initial memory snapshot if possible
        let mem = Self::get_process_memory();
        self.current_memory.set(mem);
        self.peak_memory.set(mem);
    }

    /// Stop monitoring
    pub fn stop(&mut self) {
        if let Some(start) = self.start_time {
            self.cpu_time = start.elapsed();
        }
        self.update_peak_memory();
        self.running = false;
    }

    /// Check if monitor is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    // =========================================================================
    // Memory Monitoring
    // =========================================================================

    /// Get current memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        if self.running {
            let current = Self::get_process_memory();
            self.current_memory.set(current);
            if current > self.peak_memory.get() {
                self.peak_memory.set(current);
            }
            current
        } else {
            self.current_memory.get()
        }
    }

    /// Get peak memory usage
    pub fn peak_memory_usage(&self) -> usize {
        let current = self.memory_usage();
        let peak = self.peak_memory.get();
        if current > peak { current } else { peak }
    }

    /// Update peak memory tracking
    pub fn update_peak_memory(&self) {
        let current = Self::get_process_memory();
        self.current_memory.set(current);
        if current > self.peak_memory.get() {
            self.peak_memory.set(current);
        }
    }

    /// Get process memory (platform-specific)
    fn get_process_memory() -> usize {
        if let Ok(pid) = sysinfo::get_current_pid() {
            let mut system = System::new();
            system.refresh_processes_specifics(ProcessRefreshKind::new().with_memory());
            if let Some(process) = system.process(pid) {
                return process.memory() as usize;
            }
        }
        0
    }

    // =========================================================================
    // CPU Monitoring
    // =========================================================================

    /// Get CPU time used
    pub fn cpu_time(&self) -> Duration {
        if self.running {
            self.start_time
                .map(|t| t.elapsed())
                .unwrap_or(Duration::ZERO)
        } else {
            self.cpu_time
        }
    }

    // =========================================================================
    // File Monitoring
    // =========================================================================

    /// Get number of open file descriptors
    pub fn file_count(&self) -> usize {
        #[cfg(target_os = "linux")]
        {
            // Count entries in /proc/self/fd
            if let Ok(entries) = std::fs::read_dir("/proc/self/fd") {
                return entries.count();
            }
            0
        }

        #[cfg(not(target_os = "linux"))]
        {
            self.file_count
        }
    }

    /// Increment file count (for tracking)
    pub fn track_file_open(&mut self) {
        self.file_count += 1;
    }

    /// Decrement file count
    pub fn track_file_close(&mut self) {
        if self.file_count > 0 {
            self.file_count -= 1;
        }
    }

    // =========================================================================
    // Network Monitoring
    // =========================================================================

    /// Get bytes sent
    pub fn bytes_sent(&self) -> u64 {
        self.bytes_sent
    }

    /// Get bytes received
    pub fn bytes_received(&self) -> u64 {
        self.bytes_received
    }

    /// Track bytes sent
    pub fn track_send(&mut self, bytes: u64) {
        self.bytes_sent += bytes;
    }

    /// Track bytes received
    pub fn track_receive(&mut self, bytes: u64) {
        self.bytes_received += bytes;
    }

    // =========================================================================
    // Reporting
    // =========================================================================

    /// Generate a summary report
    pub fn report(&self) -> String {
        format!(
            "ResourceMonitor Report:\n\
             - Memory: {} bytes (peak: {} bytes)\n\
             - CPU Time: {:?}\n\
             - Files: {}\n\
             - Network: {} sent, {} received",
            self.memory_usage(),
            self.peak_memory_usage(),
            self.cpu_time(),
            self.file_count(),
            self.bytes_sent,
            self.bytes_received
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_creation() {
        let monitor = ResourceMonitor::new();
        assert!(!monitor.is_running());
    }

    #[test]
    fn test_monitor_lifecycle() {
        let mut monitor = ResourceMonitor::new();

        monitor.start();
        assert!(monitor.is_running());

        // Do some work
        std::thread::sleep(Duration::from_millis(10));

        let cpu_time = monitor.cpu_time();
        assert!(cpu_time >= Duration::from_millis(10));

        monitor.stop();
        assert!(!monitor.is_running());
    }

    #[test]
    fn test_network_tracking() {
        let mut monitor = ResourceMonitor::new();

        monitor.track_send(100);
        monitor.track_receive(200);

        assert_eq!(monitor.bytes_sent(), 100);
        assert_eq!(monitor.bytes_received(), 200);
    }
}
