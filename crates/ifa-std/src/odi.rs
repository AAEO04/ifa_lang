//! # Òdí Domain (1001)
//!
//! The Seal - File I/O and Database
//!
//! Safe file operations with sandboxed paths and rusqlite for SQLite.

use crate::impl_odu_domain;
use ifa_core::error::{IfaError, IfaResult};
use rusqlite::Connection;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use ifa_sandbox::{CapabilitySet, Ofun};

/// Òdí - The Seal (Files/DB)
#[derive(Default)]
pub struct Odi {
    capabilities: CapabilitySet,
}

impl_odu_domain!(Odi, "Òdí", "1001", "The Seal - Files/Database");

impl Odi {
    /// Create with capabilities
    pub fn new(capabilities: CapabilitySet) -> Self {
        Odi { capabilities }
    }

    /// Check if read capability exists for path
    fn check_read(&self, path: &Path) -> IfaResult<()> {
        if self.capabilities.check(&Ofun::ReadFiles {
            root: path.to_path_buf(),
        }) {
            Ok(())
        } else {
            Err(IfaError::PermissionDenied(format!(
                "Read permission denied for: {}",
                path.display()
            )))
        }
    }

    /// Check if write capability exists for path
    fn check_write(&self, path: &Path) -> IfaResult<()> {
        if self.capabilities.check(&Ofun::WriteFiles {
            root: path.to_path_buf(),
        }) {
            Ok(())
        } else {
            Err(IfaError::PermissionDenied(format!(
                "Write permission denied for: {}",
                path.display()
            )))
        }
    }

    // =========================================================================
    // FILE OPERATIONS
    // =========================================================================

    /// Read file contents (kà)
    pub fn ka(&self, path: &str) -> IfaResult<String> {
        let path = PathBuf::from(path);
        self.check_read(&path)?;
        fs::read_to_string(&path).map_err(IfaError::IoError)
    }

    /// Read file as bytes
    pub fn ka_bytes(&self, path: &str) -> IfaResult<Vec<u8>> {
        let path = PathBuf::from(path);
        self.check_read(&path)?;
        fs::read(&path).map_err(IfaError::IoError)
    }

    /// Read file lines
    pub fn ka_ila(&self, path: &str) -> IfaResult<Vec<String>> {
        let path = PathBuf::from(path);
        self.check_read(&path)?;
        let file = File::open(&path).map_err(IfaError::IoError)?;
        let reader = BufReader::new(file);
        reader
            .lines()
            .collect::<Result<Vec<_>, _>>()
            .map_err(IfaError::IoError)
    }

    /// Write file (kọ)
    pub fn ko(&self, path: &str, content: &str) -> IfaResult<()> {
        let path = PathBuf::from(path);
        self.check_write(&path)?;
        fs::write(&path, content).map_err(IfaError::IoError)
    }

    /// Append to file (fí)
    pub fn fi(&self, path: &str, content: &str) -> IfaResult<()> {
        let path = PathBuf::from(path);
        self.check_write(&path)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(IfaError::IoError)?;
        file.write_all(content.as_bytes())
            .map_err(IfaError::IoError)
    }

    /// Check if file exists (wà)
    pub fn wa(&self, path: &str) -> bool {
        // Checking existence requires read perm on parent or file
        let path = PathBuf::from(path);
        if self.check_read(&path).is_err() {
            return false;
        }
        path.exists()
    }

    /// Delete file (pa fáìlì)
    pub fn pa_faili(&self, path: &str) -> IfaResult<()> {
        let path = PathBuf::from(path);
        self.check_write(&path)?;
        fs::remove_file(&path).map_err(IfaError::IoError)
    }

    /// Create directory (ṣẹ̀dá àpótí)
    pub fn seda_apoti(&self, path: &str) -> IfaResult<()> {
        let path = PathBuf::from(path);
        self.check_write(&path)?;
        fs::create_dir_all(&path).map_err(IfaError::IoError)
    }

    /// List directory (àkójọ)
    pub fn akojo(&self, path: &str) -> IfaResult<Vec<String>> {
        let path = PathBuf::from(path);
        self.check_read(&path)?;
        let entries = fs::read_dir(&path).map_err(IfaError::IoError)?;

        Ok(entries
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect())
    }

    /// Get file size
    pub fn iwon(&self, path: &str) -> IfaResult<u64> {
        let path = PathBuf::from(path);
        self.check_read(&path)?;
        let meta = fs::metadata(&path).map_err(IfaError::IoError)?;
        Ok(meta.len())
    }

    // =========================================================================
    // DATABASE (SQLite)
    // =========================================================================

    /// Open SQLite database
    pub fn so_db(&self, path: &str) -> IfaResult<Connection> {
        let path = PathBuf::from(path);
        // DB requires both read and write
        self.check_read(&path)?;
        self.check_write(&path)?;
        Connection::open(&path).map_err(|e| IfaError::Custom(format!("Database error: {}", e)))
    }

    /// Open in-memory database
    pub fn so_db_iranti(&self) -> IfaResult<Connection> {
        // No perms needed for in-memory
        Connection::open_in_memory().map_err(|e| IfaError::Custom(format!("Database error: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_file_ops() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let path_str = file_path.to_str().unwrap();

        // Grant read/write access to temp directory
        let mut caps = CapabilitySet::default();
        caps.grant(Ofun::ReadFiles {
            root: dir.path().to_path_buf(),
        });
        caps.grant(Ofun::WriteFiles {
            root: dir.path().to_path_buf(),
        });
        let odi = Odi::new(caps);

        // Write
        odi.ko(path_str, "Hello, Ifá!").unwrap();
        assert!(odi.wa(path_str));

        // Read
        let content = odi.ka(path_str).unwrap();
        assert_eq!(content, "Hello, Ifá!");

        // Append
        odi.fi(path_str, "\nMore text").unwrap();
        let content = odi.ka(path_str).unwrap();
        assert!(content.contains("More text"));
    }

    #[test]
    fn test_sandbox() {
        let dir = tempdir().unwrap();

        // Create Odi with only read access to the temp directory
        let mut caps = CapabilitySet::default();
        caps.grant(Ofun::ReadFiles {
            root: dir.path().to_path_buf(),
        });
        let odi = Odi::new(caps);

        // Reading within sandbox should work (if file exists)
        // Reading outside sandbox should fail capability check
        // 1. Reading allowed path should pass (logic check, file doesn't exist but capability does)
        let allowed_path = dir.path().join("allowed.txt");
        // We expect IoError (NotFound) not PermissionDenied
        match odi.ka(allowed_path.to_str().unwrap()) {
            Err(IfaError::PermissionDenied(_)) => panic!("Should have permission!"),
            _ => {} // IoError is expected
        }

        // 2. Reading disallowed path should fail
        let denied_path = if cfg!(windows) {
            PathBuf::from("C:\\Windows\\System32\\drivers\\etc\\hosts")
        } else {
            PathBuf::from("/etc/passwd")
        };

        match odi.ka(denied_path.to_str().unwrap()) {
            Err(IfaError::PermissionDenied(_)) => {} // Success!
            Err(e) => panic!("Expected PermissionDenied, got {:?}", e),
            Ok(_) => panic!("Should have been denied!"),
        }
    }
}
