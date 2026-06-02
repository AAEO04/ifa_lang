use std::path::PathBuf;

#[derive(Clone)]
pub struct InstallConfig {
    pub install_dir: PathBuf,
    pub add_to_path: bool,
    pub update_shell: bool,
    pub create_shortcut: bool,
    pub offline_mode: bool,
}

/// Returns the canonical default install directory, hard-erroring if $HOME is unavailable.
/// Prefer this over `InstallConfig::default()` in CLI/headless paths.
pub fn default_install_dir() -> Result<PathBuf, String> {
    dirs::home_dir()
        .ok_or_else(|| "Cannot determine home directory ($HOME is not set)".to_string())
        .map(|h| h.join(".ifa"))
}

impl InstallConfig {
    /// The directory where binaries are placed (`install_dir/bin`).
    /// This is what should be added to PATH, not `install_dir` itself.
    pub fn bin_dir(&self) -> PathBuf {
        self.install_dir.join("bin")
    }
}

impl Default for InstallConfig {
    fn default() -> Self {
        // Fall back to CWD/.ifa if $HOME is missing — never an empty PathBuf.
        let install_dir = dirs::home_dir()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".ifa");
        Self {
            install_dir,
            add_to_path: true,
            update_shell: true,
            create_shortcut: false,
            offline_mode: false,
        }
    }
}
