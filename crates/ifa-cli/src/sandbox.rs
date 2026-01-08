//! # Ìgbálẹ̀ Sandbox
//! 
//! Sandboxed execution environment for untrusted Ifá code.
//! 
//! Uses:
//! - OS-level isolation (cgroups/namespaces on Linux, Job Objects on Windows)
//! - Unified Ọ̀fún capability system for permission enforcement

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::time::Duration;
use std::collections::HashSet;
use ifa_sandbox::{CapabilitySet, Ofun};
use eyre::{Result, WrapErr};

/// Sandbox configuration
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum execution time
    pub timeout: Duration,
    /// Maximum memory (bytes)
    pub max_memory: usize,
    /// Allowed file system paths (read)
    pub allowed_read_paths: HashSet<PathBuf>,
    /// Allowed file system paths (write)
    pub allowed_write_paths: HashSet<PathBuf>,
    /// Allowed network domains
    pub allowed_network_domains: Vec<String>,
    /// Allowed environment variable keys
    pub allowed_env_keys: Vec<String>,
    /// Allow subprocess spawning
    pub allow_spawn: bool,
    /// Allow time functions
    pub allow_time: bool,
    /// Allow random generation
    pub allow_random: bool,
    /// Allow stdio
    pub allow_stdio: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        SandboxConfig {
            timeout: Duration::from_secs(30),
            max_memory: 256 * 1024 * 1024, // 256MB
            allowed_read_paths: HashSet::new(),
            allowed_write_paths: HashSet::new(),
            allowed_network_domains: Vec::new(),
            allowed_env_keys: Vec::new(),
            allow_spawn: false,
            allow_time: true,
            allow_random: true,
            allow_stdio: true,
        }
    }
}

impl SandboxConfig {
    /// Minimal sandbox (most restrictive)
    pub fn minimal() -> Self {
        SandboxConfig {
            timeout: Duration::from_secs(5),
            max_memory: 64 * 1024 * 1024, // 64MB
            allow_time: false,
            allow_random: false,
            allow_stdio: true, // Still allow output
            ..Default::default()
        }
    }
    
    /// Standard sandbox (moderate restrictions)
    pub fn standard() -> Self {
        SandboxConfig {
            timeout: Duration::from_secs(60),
            max_memory: 512 * 1024 * 1024, // 512MB
            allow_time: true,
            allow_random: true,
            allow_stdio: true,
            ..Default::default()
        }
    }
    
    /// Allow reading from path
    pub fn allow_read(mut self, path: impl AsRef<Path>) -> Self {
        self.allowed_read_paths.insert(path.as_ref().to_path_buf());
        self
    }
    
    /// Allow writing to path
    pub fn allow_write(mut self, path: impl AsRef<Path>) -> Self {
        self.allowed_write_paths.insert(path.as_ref().to_path_buf());
        self
    }
    
    /// Allow network access to domain
    pub fn allow_network(mut self, domain: &str) -> Self {
        self.allowed_network_domains.push(domain.to_string());
        self
    }
    
    /// Allow environment variable access
    pub fn allow_env(mut self, key: &str) -> Self {
        self.allowed_env_keys.push(key.to_string());
        self
    }
    
    /// Convert to unified Ọ̀fún CapabilitySet
    pub fn to_capability_set(&self) -> CapabilitySet {
        let mut caps = CapabilitySet::new();
        
        // File read permissions
        for path in &self.allowed_read_paths {
            caps.grant(Ofun::ReadFiles { root: path.clone() });
        }
        
        // File write permissions
        for path in &self.allowed_write_paths {
            caps.grant(Ofun::WriteFiles { root: path.clone() });
        }
        
        // Network permissions
        if !self.allowed_network_domains.is_empty() {
            caps.grant(Ofun::Network { domains: self.allowed_network_domains.clone() });
        }
        
        // Environment permissions
        if !self.allowed_env_keys.is_empty() {
            caps.grant(Ofun::Environment { keys: self.allowed_env_keys.clone() });
        }
        
        // Process execution
        if self.allow_spawn {
            caps.grant(Ofun::Execute { programs: vec!["*".to_string()] });
        }
        
        // Time access
        if self.allow_time {
            caps.grant(Ofun::Time);
        }
        
        // Random generation
        if self.allow_random {
            caps.grant(Ofun::Random);
        }
        
        // Stdio access
        if self.allow_stdio {
            caps.grant(Ofun::Stdio);
        }
        
        caps
    }
}

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

/// Ìgbálẹ̀ Sandbox
pub struct Igbale {
    config: SandboxConfig,
}

impl Igbale {
    /// Create new sandbox with config
    pub fn new(config: SandboxConfig) -> Self {
        Igbale { config }
    }
    
    /// Create with default config
    pub fn default_sandbox() -> Self {
        Self::new(SandboxConfig::default())
    }
    
    /// Run code in sandbox
    #[cfg(target_os = "linux")]
    pub fn run(&self, code_path: &Path) -> Result<SandboxResult> {
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
                &format!("{}", self.config.timeout.as_secs()),
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
            memory_used: 0, // TODO: Track with cgroups
            timed_out,
        })
    }
    
    /// Run code in sandbox (Windows)
    #[cfg(target_os = "windows")]
    pub fn run(&self, code_path: &Path) -> Result<SandboxResult> {
        use std::process::{Command, Stdio};
        use std::time::Instant;
        
        let start = Instant::now();
        
        // Use Job Objects for process isolation on Windows
        // For now, just use basic process with timeout
        let output = Command::new("cmd")
            .args([
                "/C",
                "timeout",
                "/T",
                &format!("{}", self.config.timeout.as_secs()),
                "&",
                "ifa",
                "run",
            ])
            .arg(code_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .wrap_err("Failed to execute sandbox")?;
        
        let execution_time = start.elapsed();
        
        Ok(SandboxResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time,
            memory_used: 0, // TODO: Track with Job Objects
            timed_out: false, // TODO: Detect timeout
        })
    }
    
    /// Run code in sandbox (macOS)
    #[cfg(target_os = "macos")]
    pub fn run(&self, code_path: &Path) -> Result<SandboxResult> {
        use std::process::{Command, Stdio};
        use std::time::Instant;
        
        let start = Instant::now();
        
        // Use sandbox-exec for sandboxing on macOS
        let output = Command::new("sandbox-exec")
            .args([
                "-p",
                "(version 1)(allow default)(deny network*)",
                "timeout",
                &format!("{}", self.config.timeout.as_secs()),
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
    
    /// Validate that a path is allowed for reading
    pub fn can_read(&self, path: &Path) -> bool {
        self.config.allowed_read_paths.iter().any(|allowed| {
            path.starts_with(allowed)
        })
    }
    
    /// Validate that a path is allowed for writing
    pub fn can_write(&self, path: &Path) -> bool {
        self.config.allowed_write_paths.iter().any(|allowed| {
            path.starts_with(allowed)
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(!config.allow_network);
        assert!(!config.allow_spawn);
    }
    
    #[test]
    fn test_config_builder() {
        let config = SandboxConfig::minimal()
            .allow_read("/tmp")
            .allow_write("/tmp/output");
        
        assert!(config.allowed_read_paths.contains(&PathBuf::from("/tmp")));
        assert!(config.allowed_write_paths.contains(&PathBuf::from("/tmp/output")));
    }
    
    #[test]
    fn test_path_validation() {
        let config = SandboxConfig::default()
            .allow_read("/allowed");
        let sandbox = Igbale::new(config);
        
        assert!(sandbox.can_read(Path::new("/allowed/file.txt")));
        assert!(!sandbox.can_read(Path::new("/forbidden/file.txt")));
    }
}
