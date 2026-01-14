use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UninstallError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Platform error: {0}")]
    Platform(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Validates that the directory being uninstalled is safe to delete.
/// Prevents accidental deletion of system directories or Home root.
fn validate_uninstall_dir(install_dir: &Path) -> Result<(), UninstallError> {
    if !install_dir.exists() {
        return Ok(());
    }
    
    // Canonicalize to resolve relative paths and symlinks
    let canonical = install_dir.canonicalize().map_err(UninstallError::Io)?;
    
    // Safety checks: refuse to delete critical directories
    let home = dirs::home_dir().ok_or_else(|| {
        UninstallError::InvalidPath("Could not determine home directory".to_string())
    })?;
    
    // 1. Never delete the home directory itself
    if canonical == home {
        return Err(UninstallError::InvalidPath("Cannot uninstall the entire home directory".to_string()));
    }
    
    // 2. Never delete root or critical system paths (basic check)
    #[cfg(unix)]
    {
        if canonical == Path::new("/") || canonical.starts_with("/usr") || canonical.starts_with("/bin") {
            return Err(UninstallError::InvalidPath(format!("Path {:?} is a protected system directory", canonical)));
        }
    }
    
    #[cfg(windows)]
    {
        let win_dir = std::env::var("windir").map(Path::new).unwrap_or(Path::new("C:\\Windows"));
        if canonical.starts_with(win_dir) || canonical == Path::new("C:\\") {
            return Err(UninstallError::InvalidPath(format!("Path {:?} is a protected system directory", canonical)));
        }
    }
    
    // 3. Must be a directory (not a file)
    if !canonical.is_dir() {
        return Err(UninstallError::InvalidPath("Uninstall target is not a directory".to_string()));
    }
    
    // 4. Heuristic: ensure it's likely an Ifa-lang directory 
    // (e.g., contains 'ifa' in path or a specific marker file - optional but recommended)
    
    Ok(())
}

pub fn uninstall(install_dir: &Path) -> Result<(), UninstallError> {
    println!("Uninstalling Ifá-Lang from {:?}...", install_dir);

    // Validate path before doing anything
    validate_uninstall_dir(install_dir)?;

    // 1. Remove from PATH
    #[cfg(target_os = "windows")]
    {
        use crate::platform::windows::remove_from_path;
        remove_from_path(install_dir).map_err(|e| UninstallError::Platform(e.to_string()))?;
    }
    
    #[cfg(unix)]
    {
        use crate::platform::unix::remove_from_path;
        remove_from_path(install_dir).map_err(|e| UninstallError::Platform(e.to_string()))?;
    }

    // 2. Delete files
    if install_dir.exists() {
        println!("Deleting files in {:?}...", install_dir);
        fs::remove_dir_all(install_dir)?;
    }

    println!("Uninstallation complete.");
    Ok(())
}
