use std::io;
use std::path::Path;

#[cfg(target_os = "windows")]
use winreg::RegKey;
#[cfg(target_os = "windows")]
use winreg::enums::*;

/// Maximum length for PATH environment variable to prevent overflow
const MAX_PATH_LENGTH: usize = 32_767;

/// Characters that are unsafe in PATH entries (could enable command injection)
const UNSAFE_PATH_CHARS: &[char] = &[
    '|', '&', ';', '`', '$', '(', ')', '<', '>', '"', '\'', '\n', '\r',
];

#[derive(Debug)]
pub enum PathError {
    Io(io::Error),
    InvalidPath(String),
    PathTooLong,
    RollbackFailed(String),
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
            PathError::PathTooLong => write!(f, "PATH would exceed maximum length"),
            PathError::RollbackFailed(reason) => write!(f, "Rollback failed: {}", reason),
        }
    }
}

impl std::error::Error for PathError {}

/// Validates that an install directory is safe to add to PATH.
fn validate_install_path(install_dir: &Path) -> Result<(), PathError> {
    let path_str = install_dir
        .to_str()
        .ok_or_else(|| PathError::InvalidPath("Path contains invalid UTF-8".to_string()))?;

    // Check for path traversal attempts
    if path_str.contains("..") {
        return Err(PathError::InvalidPath(
            "Path contains '..' traversal".to_string(),
        ));
    }

    // Check for unsafe characters that could enable command injection
    for c in UNSAFE_PATH_CHARS {
        if path_str.contains(*c) {
            return Err(PathError::InvalidPath(format!(
                "Path contains unsafe character: '{}'",
                c
            )));
        }
    }

    // Ensure path is absolute
    if !install_dir.is_absolute() {
        return Err(PathError::InvalidPath("Path must be absolute".to_string()));
    }

    // Check path exists and is a directory
    if !install_dir.is_dir() {
        return Err(PathError::InvalidPath(
            "Path does not exist or is not a directory".to_string(),
        ));
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn add_to_path(install_dir: &Path) -> Result<(), PathError> {
    // Validate install directory before any registry operations
    validate_install_path(install_dir)?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;

    let path: String = env.get_value("Path")?;
    let new_path = install_dir
        .to_str()
        .ok_or_else(|| PathError::InvalidPath("Path contains invalid UTF-8".to_string()))?;

    if !path.contains(new_path) {
        let updated_path = format!("{};{}", path, new_path);

        // Check length limit
        if updated_path.len() > MAX_PATH_LENGTH {
            return Err(PathError::PathTooLong);
        }

        // Backup current PATH for potential rollback
        let backup = path.clone();

        // Attempt to update PATH
        if let Err(e) = env.set_value("Path", &updated_path) {
            // Attempt rollback
            if let Err(rollback_err) = env.set_value("Path", &backup) {
                return Err(PathError::RollbackFailed(format!(
                    "Original error: {}, Rollback error: {}",
                    e, rollback_err
                )));
            }
            return Err(PathError::Io(e));
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn remove_from_path(install_dir: &Path) -> Result<(), PathError> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;

    let path: String = env.get_value("Path")?;
    let target = install_dir
        .to_str()
        .ok_or_else(|| PathError::InvalidPath("Path contains invalid UTF-8".to_string()))?;

    if path.contains(target) {
        // Safely remove the target from PATH
        let updated_path = path
            .split(';')
            .filter(|p| *p != target)
            .collect::<Vec<_>>()
            .join(";");

        env.set_value("Path", &updated_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validate_rejects_path_traversal() {
        let path = PathBuf::from("C:\\..\\Windows\\System32");
        assert!(matches!(
            validate_install_path(&path),
            Err(PathError::InvalidPath(_))
        ));
    }

    #[test]
    fn test_validate_rejects_unsafe_chars() {
        let path = PathBuf::from("C:\\Program Files\\test|malicious");
        assert!(matches!(
            validate_install_path(&path),
            Err(PathError::InvalidPath(_))
        ));
    }

    #[test]
    fn test_validate_rejects_relative_paths() {
        let path = PathBuf::from("relative\\path");
        assert!(matches!(
            validate_install_path(&path),
            Err(PathError::InvalidPath(_))
        ));
    }
}
