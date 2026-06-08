use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    pub fn new() -> Self {
        let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        dir.push(".ifa_cache");
        Self { cache_dir: dir }
    }

    fn get_cache_path(&self, namespace: &str, source_path: &Path) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        // Hash the absolute path so different files with same name don't collide
        let abs_path = fs::canonicalize(source_path).unwrap_or_else(|_| source_path.to_path_buf());
        abs_path.to_string_lossy().hash(&mut hasher);
        let hash = hasher.finish();

        let mut path = self.cache_dir.clone();
        path.push(namespace);
        path.push(format!("{:016x}.result", hash));
        path
    }

    /// Checks if a cached result is available and valid (newer than source file)
    pub fn check_cache(&self, namespace: &str, source_path: &Path) -> Option<bool> {
        let cache_path = self.get_cache_path(namespace, source_path);

        let source_meta = fs::metadata(source_path).ok()?;
        let source_mtime = source_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);

        let cache_meta = fs::metadata(&cache_path).ok()?;
        let cache_mtime = cache_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);

        // Cache is valid if it was created at or after the source file's last modification
        if cache_mtime >= source_mtime {
            let content = fs::read_to_string(&cache_path).ok()?;
            Some(content.trim() == "true")
        } else {
            None
        }
    }

    /// Updates the cache with a new result
    pub fn update_cache(&self, namespace: &str, source_path: &Path, passed: bool) {
        let cache_path = self.get_cache_path(namespace, source_path);
        if let Some(parent) = cache_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(cache_path, if passed { "true" } else { "false" });
    }
}
