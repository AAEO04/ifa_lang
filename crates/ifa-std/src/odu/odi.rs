//! # Òdí Domain (1001)
//!
//! The Seal - File I/O and Database
//!
//! Safe file operations with sandboxed paths and rusqlite for SQLite.

use crate::impl_odu_domain;
use crate::esu::Esu;
use ifa_vm::error::{IfaError, IfaResult};
use memmap2::MmapOptions;
use rusqlite::Connection;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::sandbox_shim::Ofun;

/// Òdí - The Seal (Files/DB)
#[derive(Default)]
pub struct Odi {
    esu: Esu,
}

impl_odu_domain!(Odi, "Òdí", "1001", "The Seal - Files/Database");

impl Odi {
    /// Create with capabilities
    pub fn new(esu: Esu) -> Self {
        Odi { esu }
    }

    /// Check if read capability exists for path
    fn check_read(&self, path: &Path) -> IfaResult<PathBuf> {
        if path.is_symlink() {
            return Err(IfaError::PermissionDenied(format!("Symlinks forbidden: {}", path.display())));
        }
        let canonical = path.canonicalize().map_err(|e| IfaError::IoError(e.to_string()))?;
        self.esu.enforce_crossroads(&Ofun::ReadFiles {
            root: canonical.clone(),
        }, &format!("Òdí::ka({})", canonical.display()))?;
        Ok(canonical)
    }

    /// Check if write capability exists for path
    fn check_write(&self, path: &Path) -> IfaResult<PathBuf> {
        if path.is_symlink() {
            return Err(IfaError::PermissionDenied(format!("Symlinks forbidden: {}", path.display())));
        }
        // File might not exist, canonicalize parent
        let parent = path.parent().unwrap_or(Path::new(""));
        let parent_canon = parent.canonicalize().unwrap_or_else(|_| parent.to_path_buf());
        let canonical = parent_canon.join(path.file_name().unwrap_or_default());
        
        self.esu.enforce_crossroads(&Ofun::WriteFiles {
            root: canonical.clone(),
        }, &format!("Òdí::kọ({})", canonical.display()))?;
        Ok(canonical)
    }

    // =========================================================================
    // FILE OPERATIONS
    // =========================================================================

    /// Read file contents (kà)
    pub fn ka(&self, path: &str) -> IfaResult<String> {
        let canonical = self.check_read(Path::new(path))?;
        fs::read_to_string(&canonical).map_err(|e| IfaError::IoError(e.to_string()))
    }

    /// Read file as bytes
    pub fn ka_bytes(&self, path: &str) -> IfaResult<Vec<u8>> {
        let canonical = self.check_read(Path::new(path))?;
        fs::read(&canonical).map_err(|e| IfaError::IoError(e.to_string()))
    }

    /// Memory map file for zero-copy access (returns base64-encoded string for now, to fit value system)
    /// In a pure VM integration, this would return an opaque Bytes handle.
    pub fn ka_mmap(&self, path: &str) -> IfaResult<String> {
        let canonical = self.check_read(Path::new(path))?;
        let file = File::open(&canonical).map_err(|e| IfaError::IoError(e.to_string()))?;
        let mmap = unsafe {
            MmapOptions::new()
                .map(&file)
                .map_err(|e| IfaError::IoError(format!("Mmap error: {}", e)))?
        };
        // Zero-copy abstraction natively passes mmap bytes directly into Regex or parser without BufReader
        // But since AST needs an IfaValue return, we serialize it.
        // A true zero-copy would hold the Mmap object natively in IfaValue::Bytes or IfaValue::Object.
        // We simulate native speed over the bridge by passing as UTF-8 directly.
        String::from_utf8(mmap.to_vec())
            .map_err(|e| IfaError::IoError(format!("Invalid text in mmap: {}", e)))
    }

    /// Read file lines
    pub fn ka_ila(&self, path: &str) -> IfaResult<Vec<String>> {
        let canonical = self.check_read(Path::new(path))?;
        let file = File::open(&canonical).map_err(|e| IfaError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);
        reader
            .lines()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| IfaError::IoError(e.to_string()))
    }

    /// Write file (kọ)
    pub fn ko(&self, path: &str, content: &str) -> IfaResult<()> {
        let canonical = self.check_write(Path::new(path))?;
        fs::write(&canonical, content).map_err(|e| IfaError::IoError(e.to_string()))
    }

    /// Append to file (fí)
    pub fn fi(&self, path: &str, content: &str) -> IfaResult<()> {
        let canonical = self.check_write(Path::new(path))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&canonical)
            .map_err(|e| IfaError::IoError(e.to_string()))?;
        file.write_all(content.as_bytes())
            .map_err(|e| IfaError::IoError(e.to_string()))
    }

    /// Check if file exists (wà)
    pub fn wa(&self, path: &str) -> bool {
        // Checking existence requires read perm on parent or file
        let path_obj = Path::new(path);
        let parent = path_obj.parent().unwrap_or(Path::new(""));
        let parent_canon = parent.canonicalize().unwrap_or_else(|_| parent.to_path_buf());
        let canonical = parent_canon.join(path_obj.file_name().unwrap_or_default());
        if self.esu.enforce_crossroads(&Ofun::ReadFiles { root: canonical.clone() }, "Odi::wa").is_err() {
            return false;
        }
        canonical.exists()
    }

    /// Delete file (pa fáìlì)
    pub fn pa_faili(&self, path: &str) -> IfaResult<()> {
        let canonical = self.check_write(Path::new(path))?;
        fs::remove_file(&canonical).map_err(|e| IfaError::IoError(e.to_string()))
    }

    /// Create directory (ṣẹ̀dá àpótí)
    pub fn seda_apoti(&self, path: &str) -> IfaResult<()> {
        let canonical = self.check_write(Path::new(path))?;
        fs::create_dir_all(&canonical).map_err(|e| IfaError::IoError(e.to_string()))
    }

    /// List directory (àkójọ)
    pub fn akojo(&self, path: &str) -> IfaResult<Vec<String>> {
        let canonical = self.check_read(Path::new(path))?;
        let entries = fs::read_dir(&canonical).map_err(|e| IfaError::IoError(e.to_string()))?;

        Ok(entries
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect())
    }

    /// Get file size
    pub fn iwon(&self, path: &str) -> IfaResult<u64> {
        let path = ifa_types::capability::canonicalize_safe(Path::new(path));
        self.check_read(&path)?;
        let meta = fs::metadata(&path).map_err(|e| IfaError::IoError(e.to_string()))?;
        Ok(meta.len())
    }

    // =========================================================================
    // DATABASE (SQLite)
    // =========================================================================

    /// Open SQLite database
    pub fn so_db(&self, path: &str) -> IfaResult<Connection> {
        let path = ifa_types::capability::canonicalize_safe(Path::new(path));
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

    // =========================================================================
    // Ergonomic Aliases
    // =========================================================================
    
    pub fn read(&self, path: &str) -> IfaResult<String> { self.ka(path) }
    pub fn write(&self, path: &str, content: &str) -> IfaResult<()> { self.ko(path, content) }
    pub fn append(&self, path: &str, content: &str) -> IfaResult<()> { self.fi(path, content) }
    pub fn exists(&self, path: &str) -> bool { self.wa(path) }
    pub fn remove(&self, path: &str) -> IfaResult<()> { self.pa_faili(path) }
    pub fn delete(&self, path: &str) -> IfaResult<()> { self.pa_faili(path) }
    pub fn mkdir(&self, path: &str) -> IfaResult<()> { self.seda_apoti(path) }
    pub fn ls(&self, path: &str) -> IfaResult<Vec<String>> { self.akojo(path) }
    pub fn list(&self, path: &str) -> IfaResult<Vec<String>> { self.akojo(path) }
    pub fn size(&self, path: &str) -> IfaResult<u64> { self.iwon(path) }
    pub fn open_db(&self, path: &str) -> IfaResult<Connection> { self.so_db(path) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox_shim::CapabilitySet;
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
        let odi = Odi::new(crate::Esu::new(caps));

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
        let odi = Odi::new(crate::Esu::new(caps));

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
