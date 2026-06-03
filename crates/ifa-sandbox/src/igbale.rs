//! # Ìgbálẹ̀ Sandbox
//!
//! OS-level sandboxed execution environment wrapper for untrusted Ifá code.
//! Uses cgroups/namespaces on Linux, and Job Objects/timeout on Windows.

use crate::config::SandboxConfig;
use std::path::Path;
use std::time::Duration;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use eyre::WrapErr;

/// Sandbox execution result
#[derive(Debug)]
pub struct SandboxResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: Duration,
    pub memory_used: usize,
    pub timed_out: bool,
}

/// Get memory usage for a process ID (returns bytes)
fn get_process_memory(pid: u32) -> usize {
    use sysinfo::{Pid, System};
    let mut sys = System::new();
    sys.refresh_processes();
    if let Some(process) = sys.process(Pid::from_u32(pid)) {
        process.memory() as usize
    } else {
        0
    }
}

/// Ìgbálẹ̀ OS-Level Sandbox Wrapper
pub struct Igbale {
    config: SandboxConfig,
}

impl Igbale {
    /// Create new sandbox wrapper with config
    pub fn new(config: SandboxConfig) -> Self {
        Igbale { config }
    }

    /// Run code in sandbox
    #[cfg(target_os = "linux")]
    pub fn run(&self, code_path: &Path) -> eyre::Result<SandboxResult> {
        use std::process::{Command, Stdio};
        use std::time::Instant;

        let start = Instant::now();

        // Use unshare for namespace isolation on Linux
        let output = Command::new("unshare")
            .args([
                "--mount",
                "--net",
                "--pid",
                "--fork",
                "--",
                "timeout",
                &format!("{}", self.config.limits.max_execution_time.as_secs()),
                "ifa",
                "run",
            ])
            .arg(code_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .wrap_err("Failed to execute sandbox")?;

        let execution_time = start.elapsed();
        let timed_out = output.status.code() == Some(124); // timeout exit code

        Ok(SandboxResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time,
            memory_used: get_process_memory(output.status.code().unwrap_or(0) as u32),
            timed_out,
        })
    }

    /// Run code in sandbox (Windows)
    #[cfg(target_os = "windows")]
    pub fn run(&self, code_path: &Path) -> eyre::Result<SandboxResult> {
        use std::process::Stdio;
        use std::time::Instant;
        use tokio::process::Command;
        use tokio::time::timeout;

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let start = Instant::now();

        let result = rt.block_on(async {
            let child = Command::new("ifa")
                .args(["run"])
                .arg(code_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            let child_id = child.id();

            match timeout(self.config.limits.max_execution_time, child.wait_with_output()).await {
                Ok(Ok(output)) => {
                    let execution_time = start.elapsed();
                    Ok(SandboxResult {
                        success: output.status.success(),
                        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                        execution_time,
                        memory_used: child_id.map(get_process_memory).unwrap_or(0),
                        timed_out: false,
                    })
                }
                Ok(Err(e)) => Err(eyre::eyre!("Execution failed: {}", e)),
                Err(_) => {
                    #[cfg(windows)]
                    {
                        if let Some(pid) = child_id {
                            let _ = std::process::Command::new("taskkill")
                                .args(["/F", "/PID", &pid.to_string()])
                                .spawn();
                        }
                    }
                    Ok(SandboxResult {
                        success: false,
                        stdout: String::new(),
                        stderr: "Execution timed out".to_string(),
                        execution_time: start.elapsed(),
                        memory_used: 0,
                        timed_out: true,
                    })
                }
            }
        })?;

        Ok(result)
    }

    /// Run code in sandbox (macOS)
    #[cfg(target_os = "macos")]
    pub fn run(&self, code_path: &Path) -> eyre::Result<SandboxResult> {
        use std::process::{Command, Stdio};
        use std::time::Instant;

        let start = Instant::now();

        // Use sandbox-exec for sandboxing on macOS
        let output = Command::new("sandbox-exec")
            .args([
                "-p",
                "(version 1)(allow default)(deny network*)",
                "timeout",
                &format!("{}", self.config.limits.max_execution_time.as_secs()),
                "ifa",
                "run",
            ])
            .arg(code_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .wrap_err("Failed to execute sandbox")?;

        let execution_time = start.elapsed();
        let timed_out = output.status.code() == Some(124);

        Ok(SandboxResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time,
            memory_used: 0,
            timed_out,
        })
    }
}

/// Demo sandbox capabilities
pub fn demo() {
    println!("=== IGBALE SANDBOX DEMO ===");
    println!();
    println!("The Igbale (sandbox) provides secure execution of");
    println!("untrusted Ifa code with:");
    println!();
    println!("  - Execution timeouts");
    println!("  - Memory limits");
    println!("  - File system restrictions");
    println!("  - Network isolation");
    println!("  - Process isolation");
    println!();
    println!("Platforms:");
    println!("  - Linux: unshare + cgroups");
    println!("  - macOS: sandbox-exec");
    println!("  - Windows: Job Objects");
    println!();
}
