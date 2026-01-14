use std::fs;
use std::path::{Path, Component};
use std::io::{self, Read, Write};
use zip::ZipArchive;
use flate2::read::GzDecoder;
use tar::Archive;

/// Maximum total extraction size (1 GB)
const MAX_EXTRACTION_SIZE: u64 = 1024 * 1024 * 1024;

/// Maximum number of files to extract
const MAX_FILE_COUNT: usize = 10_000;

#[derive(Debug)]
pub enum ExtractionError {
    Io(io::Error),
    SizeLimitExceeded { size: u64, limit: u64 },
    FileCountExceeded { count: usize, limit: usize },
    PathTraversal(String),
    UnsupportedFormat(String),
}

impl From<io::Error> for ExtractionError {
    fn from(e: io::Error) -> Self {
        ExtractionError::Io(e)
    }
}

impl std::fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractionError::Io(e) => write!(f, "IO error: {}", e),
            ExtractionError::SizeLimitExceeded { size, limit } => {
                write!(f, "Extraction size {} exceeds limit of {} bytes", size, limit)
            }
            ExtractionError::FileCountExceeded { count, limit } => {
                write!(f, "File count {} exceeds limit of {}", count, limit)
            }
            ExtractionError::PathTraversal(path) => {
                write!(f, "Path traversal detected: {}", path)
            }
            ExtractionError::UnsupportedFormat(ext) => {
                write!(f, "Unsupported archive format: {}", ext)
            }
        }
    }
}

impl std::error::Error for ExtractionError {}

/// Validates that a path within an archive is safe to extract.
/// Returns the sanitized path relative to the target directory.
fn validate_archive_path(entry_path: &Path, target_dir: &Path) -> Result<std::path::PathBuf, ExtractionError> {
    // Check for path traversal components
    for component in entry_path.components() {
        match component {
            Component::ParentDir => {
                return Err(ExtractionError::PathTraversal(
                    entry_path.display().to_string()
                ));
            }
            Component::Prefix(_) | Component::RootDir => {
                // Absolute paths in archives are suspicious
                return Err(ExtractionError::PathTraversal(
                    format!("Absolute path in archive: {}", entry_path.display())
                ));
            }
            _ => {}
        }
    }
    
    let full_path = target_dir.join(entry_path);
    
    // Canonicalize and verify the path stays within target_dir
    // Note: We can't canonicalize if the path doesn't exist yet,
    // so we normalize manually
    let normalized = normalize_path(&full_path);
    
    // Ensure the normalized path starts with target_dir
    let target_canonical = if target_dir.exists() {
        target_dir.canonicalize().unwrap_or_else(|_| target_dir.to_path_buf())
    } else {
        normalize_path(target_dir)
    };
    
    if !normalized.starts_with(&target_canonical) {
        return Err(ExtractionError::PathTraversal(
            format!("Path {} escapes target directory", entry_path.display())
        ));
    }
    
    Ok(full_path)
}

/// Normalize a path by removing . and .. components
fn normalize_path(path: &Path) -> std::path::PathBuf {
    let mut components: Vec<Component> = Vec::new();
    
    for component in path.components() {
        match component {
            Component::ParentDir => {
                if !components.is_empty() {
                    components.pop();
                }
            }
            Component::CurDir => {}
            c => components.push(c),
        }
    }
    
    components.iter().collect()
}

pub fn extract(archive_path: &Path, target_dir: &Path) -> Result<(), ExtractionError> {
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?;
    }

    let extension = archive_path.extension().and_then(|s| s.to_str()).unwrap_or("");

    match extension {
        "zip" => extract_zip(archive_path, target_dir),
        "gz" | "tgz" => extract_tar_gz(archive_path, target_dir),
        _ => Err(ExtractionError::UnsupportedFormat(extension.to_string())),
    }
}

fn extract_zip(archive_path: &Path, target_dir: &Path) -> Result<(), ExtractionError> {
    let file = fs::File::open(archive_path)?;
    let mut archive = ZipArchive::new(file).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut total_size: u64 = 0;
    let file_count = archive.len();
    
    // Check file count limit
    if file_count > MAX_FILE_COUNT {
        return Err(ExtractionError::FileCountExceeded {
            count: file_count,
            limit: MAX_FILE_COUNT,
        });
    }

    for i in 0..file_count {
        let mut file = archive.by_index(i).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        // Use enclosed_name for basic path traversal protection
        let entry_path = match file.enclosed_name() {
            Some(path) => path.to_path_buf(),
            None => continue,
        };
        
        // Additional path validation
        let outpath = validate_archive_path(&entry_path, target_dir)?;

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            // Check cumulative size
            total_size += file.size();
            if total_size > MAX_EXTRACTION_SIZE {
                return Err(ExtractionError::SizeLimitExceeded {
                    size: total_size,
                    limit: MAX_EXTRACTION_SIZE,
                });
            }
            
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            
            // Extract with size limit enforcement
            let mut outfile = fs::File::create(&outpath)?;
            let mut buffer = [0u8; 8192];
            let mut written: u64 = 0;
            
            loop {
                let bytes_read = file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                written += bytes_read as u64;
                if written > file.size() + 1024 {
                    // File is larger than advertised (zip bomb)
                    return Err(ExtractionError::SizeLimitExceeded {
                        size: written,
                        limit: file.size(),
                    });
                }
                outfile.write_all(&buffer[..bytes_read])?;
            }
        }

        // Set permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                // Sanitize mode - remove setuid/setgid/sticky bits
                let safe_mode = mode & 0o777;
                fs::set_permissions(&outpath, fs::Permissions::from_mode(safe_mode))?;
            }
        }
    }
    Ok(())
}

fn extract_tar_gz(archive_path: &Path, target_dir: &Path) -> Result<(), ExtractionError> {
    let tar_gz = fs::File::open(archive_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    
    let mut total_size: u64 = 0;
    let mut file_count: usize = 0;
    
    for entry in archive.entries()? {
        let mut entry = entry?;
        
        file_count += 1;
        if file_count > MAX_FILE_COUNT {
            return Err(ExtractionError::FileCountExceeded {
                count: file_count,
                limit: MAX_FILE_COUNT,
            });
        }
        
        let entry_path = entry.path()?.to_path_buf();
        let outpath = validate_archive_path(&entry_path, target_dir)?;
        
        // Check size
        total_size += entry.size();
        if total_size > MAX_EXTRACTION_SIZE {
            return Err(ExtractionError::SizeLimitExceeded {
                size: total_size,
                limit: MAX_EXTRACTION_SIZE,
            });
        }
        
        // Create parent directories
        if let Some(parent) = outpath.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Extract entry
        entry.unpack(&outpath)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_path_traversal_rejected() {
        let target = Path::new("C:\\install");
        let malicious = Path::new("..\\..\\Windows\\System32\\evil.exe");
        assert!(matches!(
            validate_archive_path(malicious, target),
            Err(ExtractionError::PathTraversal(_))
        ));
    }
    
    #[test]
    fn test_validate_safe_path_accepted() {
        let target = Path::new("C:\\install");
        let safe = Path::new("bin\\ifa.exe");
        // This should not error on path validation
        // (it may error on canonicalization if target doesn't exist)
        let result = validate_archive_path(safe, target);
        assert!(result.is_ok() || matches!(result, Err(ExtractionError::Io(_))));
    }
    
    #[test]
    fn test_normalize_path() {
        let path = Path::new("a/b/../c/./d");
        let normalized = normalize_path(path);
        assert!(normalized.ends_with("a/c/d") || normalized.ends_with("a\\c\\d"));
    }
}
