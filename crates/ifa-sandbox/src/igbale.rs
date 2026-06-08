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
        let mut child = Command::new("unshare")
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
            .spawn()
            .wrap_err("Failed to spawn sandbox")?;

        let pid = child.id();

        // Polling loop for memory usage
        let (tx, rx) = std::sync::mpsc::channel();
        let memory_monitor = std::thread::spawn(move || {
            let mut max_mem = 0;
            while rx.try_recv().is_err() {
                let mem = get_process_memory(pid);
                if mem > max_mem {
                    max_mem = mem;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            max_mem
        });

        let output = child
            .wait_with_output()
            .wrap_err("Failed to wait on sandbox")?;
        let _ = tx.send(());
        let memory_used = memory_monitor.join().unwrap_or(0);

        let execution_time = start.elapsed();
        let timed_out = output.status.code() == Some(124); // timeout exit code

        Ok(SandboxResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time,
            memory_used,
            timed_out,
        })
    }

    /// Run code in sandbox (Windows)
    #[cfg(target_os = "windows")]
    pub fn run(&self, code_path: &Path) -> eyre::Result<SandboxResult> {
        use std::os::windows::io::AsRawHandle;
        use std::os::windows::process::CommandExt;
        use std::process::Stdio;
        use std::time::Instant;
        use tokio::time::timeout;

        use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
        use windows_sys::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
        };
        use windows_sys::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_JOB_MEMORY,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_UILIMIT_DESKTOP,
            JOB_OBJECT_UILIMIT_DISPLAYSETTINGS, JOB_OBJECT_UILIMIT_EXITWINDOWS,
            JOB_OBJECT_UILIMIT_GLOBALATOMS, JOB_OBJECT_UILIMIT_HANDLES,
            JOB_OBJECT_UILIMIT_READCLIPBOARD, JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS,
            JOB_OBJECT_UILIMIT_WRITECLIPBOARD, JOBOBJECT_BASIC_UI_RESTRICTIONS,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectBasicUIRestrictions,
            JobObjectExtendedLimitInformation, SetInformationJobObject,
        };
        use windows_sys::Win32::System::Threading::{
            CREATE_SUSPENDED, OpenThread, ResumeThread, THREAD_SUSPEND_RESUME,
        };

        struct AutoHandle(HANDLE);
        impl Drop for AutoHandle {
            fn drop(&mut self) {
                unsafe {
                    if self.0 != 0 && self.0 != -1 {
                        CloseHandle(self.0);
                    }
                }
            }
        }

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let start = Instant::now();

        let result = rt.block_on(async {
            // Create Job Object with RAII guard
            let raw_job_handle =
                unsafe { CreateJobObjectW(std::ptr::null_mut(), std::ptr::null()) };
            if raw_job_handle == 0 || raw_job_handle as isize == -1 {
                return Err(eyre::eyre!("Failed to create Job Object"));
            }
            let job_handle = AutoHandle(raw_job_handle);

            // Map limits to Job Object Extended Limit Information
            let mut limit_info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION =
                unsafe { std::mem::zeroed() };
            limit_info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if self.config.limits.max_memory_bytes > 0 {
                limit_info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_JOB_MEMORY;
                limit_info.JobMemoryLimit = self.config.limits.max_memory_bytes;
            }
            limit_info.BasicLimitInformation.ActiveProcessLimit = 1;

            let set_limit = unsafe {
                SetInformationJobObject(
                    job_handle.0,
                    JobObjectExtendedLimitInformation,
                    &limit_info as *const _ as *const _,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            };

            if set_limit == 0 {
                return Err(eyre::eyre!("Failed to set job limits"));
            }

            // Set UI Restrictions
            let mut ui_restrictions: JOBOBJECT_BASIC_UI_RESTRICTIONS =
                unsafe { std::mem::zeroed() };
            ui_restrictions.UIRestrictionsClass = JOB_OBJECT_UILIMIT_DESKTOP
                | JOB_OBJECT_UILIMIT_DISPLAYSETTINGS
                | JOB_OBJECT_UILIMIT_EXITWINDOWS
                | JOB_OBJECT_UILIMIT_GLOBALATOMS
                | JOB_OBJECT_UILIMIT_HANDLES
                | JOB_OBJECT_UILIMIT_READCLIPBOARD
                | JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS
                | JOB_OBJECT_UILIMIT_WRITECLIPBOARD;

            let set_ui = unsafe {
                SetInformationJobObject(
                    job_handle.0,
                    JobObjectBasicUIRestrictions,
                    &ui_restrictions as *const _ as *const _,
                    std::mem::size_of::<JOBOBJECT_BASIC_UI_RESTRICTIONS>() as u32,
                )
            };

            if set_ui == 0 {
                return Err(eyre::eyre!("Failed to set job UI restrictions"));
            }

            // Spawn the process in a suspended state using std::process::Command
            let mut std_child = std::process::Command::new("ifa")
                .args(["run"])
                .arg(code_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .creation_flags(CREATE_SUSPENDED)
                .spawn()?;

            let child_id = std_child.id();

            // Assign child process to job immediately
            let process_handle = std_child.as_raw_handle() as HANDLE;
            let assign_result = unsafe { AssignProcessToJobObject(job_handle.0, process_handle) };
            if assign_result == 0 {
                let _ = std_child.kill();
                return Err(eyre::eyre!(
                    "Failed to assign suspended process to Job Object"
                ));
            }

            // We don't convert to tokio::process::Child since from_std isn't available on Windows tokio.
            // We just wait on the child in spawn_blocking.

            // Resume the main thread of the suspended process
            unsafe {
                let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
                if snapshot as isize != -1 {
                    let snapshot_handle = AutoHandle(snapshot);
                    let mut te32: THREADENTRY32 = std::mem::zeroed();
                    te32.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;

                    if Thread32First(snapshot_handle.0, &mut te32) != 0 {
                        loop {
                            if te32.th32OwnerProcessID == child_id {
                                let thread_handle =
                                    OpenThread(THREAD_SUSPEND_RESUME, 0, te32.th32ThreadID);
                                if thread_handle != 0 && thread_handle as isize != -1 {
                                    ResumeThread(thread_handle);
                                    CloseHandle(thread_handle);
                                }
                                break;
                            }
                            if Thread32Next(snapshot_handle.0, &mut te32) == 0 {
                                break;
                            }
                        }
                    }
                }
            }

            match timeout(
                self.config.limits.max_execution_time,
                tokio::task::spawn_blocking(move || std_child.wait_with_output()),
            )
            .await
            {
                Ok(Ok(Ok(output))) => {
                    let execution_time = start.elapsed();
                    Ok(SandboxResult {
                        success: output.status.success(),
                        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                        execution_time,
                        memory_used: get_process_memory(child_id),
                        timed_out: false,
                    })
                }
                Ok(Ok(Err(e))) => Err(eyre::eyre!("Execution failed: {}", e)),
                Ok(Err(e)) => Err(eyre::eyre!("Execution join failed: {}", e)),
                Err(_) => {
                    // child.kill() is optional since AutoHandle dropping will kill the job on close,
                    // but it's good practice to terminate early.
                    // Wait, we moved std_child into the closure above, so we can't kill it here.
                    // But that's fine, the job object kill-on-close will kill the process automatically
                    // when we drop `job_handle`.
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
    // Demo implementation removed
}
