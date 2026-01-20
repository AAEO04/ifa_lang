use std::path::Path;
use std::io::{self, Write};
use std::fs::{self, OpenOptions};

/// Characters that are unsafe in PATH entries
const UNSAFE_PATH_CHARS: &[char] = &['|', '&', ';', '`', '$', '(', ')', '<', '>', '"', '\'', '\n', '\r'];

#[derive(Debug)]
pub enum PathError {
    Io(io::Error),
    InvalidPath(String),
    ShellNotSupported(String),
}

impl From<io::Error> for PathError {
    fn from(e: io::Error) -> Self {
        PathError::Io(e)
    }
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathError::Io(e) => write!(f, "IO error: {}", e),
            PathError::InvalidPath(reason) => write!(f, "Invalid path: {}", reason),
            PathError::ShellNotSupported(shell) => write!(f, "Shell not supported: {}", shell),
        }
    }
}

impl std::error::Error for PathError {}

/// Validates that an install directory is safe to add to PATH.
fn validate_install_path(install_dir: &Path) -> Result<(), PathError> {
    let path_str = install_dir.to_str().ok_or_else(|| {
        PathError::InvalidPath("Path contains invalid UTF-8".to_string())
    })?;
    
    // Check for path traversal attempts
    if path_str.contains("..") {
        return Err(PathError::InvalidPath("Path contains '..' traversal".to_string()));
    }
    
    // Check for unsafe characters
    for c in UNSAFE_PATH_CHARS {
        if path_str.contains(*c) {
            return Err(PathError::InvalidPath(format!(
                "Path contains unsafe character: '{}'", c
            )));
        }
    }
    
    // Ensure path is absolute
    if !install_dir.is_absolute() {
        return Err(PathError::InvalidPath("Path must be absolute".to_string()));
    }
    
    Ok(())
}

/// Detects the user's shell and returns the appropriate rc file path
fn get_shell_rc_path() -> Result<std::path::PathBuf, PathError> {
    let home = dirs::home_dir().ok_or_else(|| {
        PathError::Io(io::Error::new(io::ErrorKind::NotFound, "Could not find home directory"))
    })?;
    
    // Check SHELL environment variable
    let shell = std::env::var("SHELL").unwrap_or_default();
    
    let rc_file = if shell.contains("zsh") {
        home.join(".zshrc")
    } else if shell.contains("bash") {
        // Check for .bash_profile first (macOS), then .bashrc (Linux)
        let bash_profile = home.join(".bash_profile");
        if bash_profile.exists() {
            bash_profile
        } else {
            home.join(".bashrc")
        }
    } else if shell.contains("fish") {
        home.join(".config/fish/config.fish")
    } else {
        // Default to .profile for POSIX compliance
        home.join(".profile")
    };
    
    Ok(rc_file)
}

/// Adds install directory to PATH by appending to shell rc file
pub fn add_to_path(install_dir: &Path) -> Result<(), PathError> {
    validate_install_path(install_dir)?;
    
    let rc_path = get_shell_rc_path()?;
    let path_str = install_dir.to_str().ok_or_else(|| {
        PathError::InvalidPath("Path contains invalid UTF-8".to_string())
    })?;
    
    // Check if already in rc file
    if rc_path.exists() {
        let content = fs::read_to_string(&rc_path)?;
        if content.contains(path_str) {
            println!("PATH already configured in {:?}", rc_path);
            return Ok(());
        }
    }
    
    // Create parent directories if needed (for fish)
    if let Some(parent) = rc_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Append PATH export to rc file
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&rc_path)?;
    
    let export_line = format!("\n# Added by Ifa-Lang installer\nexport PATH=\"{}:$PATH\"\n", path_str);
    file.write_all(export_line.as_bytes())?;
    
    println!("Added {} to PATH in {:?}", path_str, rc_path);
    println!("Run 'source {:?}' or restart your terminal to apply changes.", rc_path);
    
    Ok(())
}

/// Removes install directory from PATH by removing the line from shell rc file
pub fn remove_from_path(install_dir: &Path) -> Result<(), PathError> {
    let rc_path = get_shell_rc_path()?;
    let path_str = install_dir.to_str().ok_or_else(|| {
        PathError::InvalidPath("Path contains invalid UTF-8".to_string())
    })?;
    
    if !rc_path.exists() {
        return Ok(());
    }
    
    let content = fs::read_to_string(&rc_path)?;
    let export_pattern = format!("export PATH=\"{}:$PATH\"", path_str);
    
    if content.contains(&export_pattern) {
        let new_content: String = content
            .lines()
            .filter(|line| !line.contains(&export_pattern) && !line.contains("# Added by Ifa-Lang installer"))
            .collect::<Vec<_>>()
            .join("\n");
        
        fs::write(&rc_path, new_content)?;
        println!("Removed {} from PATH in {:?}", path_str, rc_path);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_validate_rejects_path_traversal() {
        let path = PathBuf::from("/usr/../etc/passwd");
        assert!(matches!(validate_install_path(&path), Err(PathError::InvalidPath(_))));
    }
    
    #[test]
    fn test_validate_rejects_unsafe_chars() {
        let path = PathBuf::from("/home/user/bin|malicious");
        assert!(matches!(validate_install_path(&path), Err(PathError::InvalidPath(_))));
    }
    
    #[test]
    fn test_validate_rejects_relative_paths() {
        let path = PathBuf::from("relative/path");
        assert!(matches!(validate_install_path(&path), Err(PathError::InvalidPath(_))));
    }
}
