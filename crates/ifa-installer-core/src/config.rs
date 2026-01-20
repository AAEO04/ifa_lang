use std::path::PathBuf;

#[derive(Clone)]
pub struct InstallConfig {
    pub install_dir: PathBuf,
    pub add_to_path: bool,
    pub update_shell: bool,
    pub create_shortcut: bool,
    pub offline_mode: bool,
}

impl Default for InstallConfig {
    fn default() -> Self {
        let install_dir = dirs::home_dir().unwrap_or_default().join(".ifa");
        Self {
            install_dir,
            add_to_path: true,
            update_shell: true,
            create_shortcut: false,
            offline_mode: false,
        }
    }
}
