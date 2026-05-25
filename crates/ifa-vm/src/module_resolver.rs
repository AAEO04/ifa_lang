//! # Unified Module Resolver
//!
//! Single canonical implementation for resolving `.ifa` and `.ifab` module paths.
//! Both the AST interpreter (`ifa run`) and the Bytecode VM (`ifa runb`) delegate
//! to this resolver, ensuring consistent behaviour across all backends.
//!
//! ## Resolution Order
//! For a module path like `utils.math`:
//! 1. Search each base dir for `utils/math.ifa`
//! 2. Search each base dir for `utils/math/mod.ifa`
//! 3. If `.ifab` is requested, repeat with that extension
//!
//! ## Canonical Module Key Format  
//! `utils.math` (dot-separated, normalised from slashes)

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::{IfaError, IfaResult};

/// Result of a successful module resolution.
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    /// Absolute path to the source or bytecode file on disk.
    pub path: PathBuf,
    /// Whether this is a pre-compiled `.ifab` binary module.
    pub is_binary: bool,
    /// Canonical dot-separated key (e.g. `"utils.math"`).
    pub key: String,
}

/// Stateless module path resolver shared across all backends.
///
/// Holds the ordered list of base directories to search.
/// The resolver is intentionally stateless — caching and circular-import
/// tracking live in each backend's execution context, not here.
#[derive(Debug, Clone, Default)]
pub struct ModuleResolver {
    /// Ordered list of base directories to search for modules.
    pub search_paths: Vec<PathBuf>,
}

impl ModuleResolver {
    /// Create a resolver with the given search paths.
    pub fn new(search_paths: Vec<PathBuf>) -> Self {
        Self { search_paths }
    }

    /// Create a resolver that only searches the directory containing
    /// the given entry-point file (i.e. the "project root").
    pub fn from_entry_file(entry: &Path) -> Self {
        let base = entry
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            search_paths: vec![base],
        }
    }

    /// Normalise an import path into a canonical dot-separated key.
    ///
    /// Accepts both slash-separated (`utils/math`) and dot-separated (`utils.math`).
    pub fn normalise(raw: &str) -> String {
        raw.replace('\\', "/")
            .split(['/', '.'])
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Resolve a user-provided import path to a file on disk.
    ///
    /// Returns `Err` with a descriptive message listing all searched paths.
    pub fn resolve(&self, raw: &str) -> IfaResult<ResolvedModule> {
        let key = Self::normalise(raw);
        if key.is_empty() {
            return Err(IfaError::FileNotFound("Empty module path".to_string()));
        }

        let parts: Vec<&str> = key.split('.').collect();
        let sep = std::path::MAIN_SEPARATOR_STR;
        let joined = parts.join(sep);

        // Build candidate relative paths (source preferred, then binary)
        let candidate_list: Vec<(String, bool)> = vec![
            (format!("{joined}.ifa"), false),
            (format!("{joined}{sep}mod.ifa"), false),
            (format!("{joined}.ifab"), true),
            (format!("{joined}{sep}mod.ifab"), true),
        ];

        let mut searched: Vec<PathBuf> = Vec::new();
        for base in &self.search_paths {
            for (rel, is_binary) in &candidate_list {
                let full = base.join(rel);
                if full.exists() {
                    return Ok(ResolvedModule {
                        path: full,
                        is_binary: *is_binary,
                        key,
                    });
                }
                searched.push(full);
            }
        }

        Err(IfaError::FileNotFound(format!(
            "Module '{}' not found. Searched:\n{}",
            key,
            searched
                .iter()
                .map(|p| format!("  - {}", p.display()))
                .collect::<Vec<_>>()
                .join("\n")
        )))
    }

    /// Returns `true` if the given raw path refers to the standard library.
    /// Std imports are handled by the runtime registry, not the file system.
    pub fn is_std(raw: &str) -> bool {
        let key = Self::normalise(raw);
        key.starts_with("std.") || key == "std"
    }

    /// Add a search path if it isn't already present.
    pub fn add_search_path(&mut self, path: PathBuf) {
        if !self.search_paths.contains(&path) {
            self.search_paths.push(path);
        }
    }
}

/// Lightweight guard for detecting circular imports at runtime.
/// Each backend creates one per execution session.
#[derive(Debug, Default)]
pub struct ImportGuard {
    importing: HashSet<String>,
}

impl ImportGuard {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark a module as currently being imported.
    /// Returns `Err` if circular import is detected.
    pub fn enter(&mut self, key: &str) -> IfaResult<()> {
        if self.importing.contains(key) {
            return Err(IfaError::Runtime(format!(
                "Circular import detected: module '{}' is already being imported",
                key
            )));
        }
        self.importing.insert(key.to_string());
        Ok(())
    }

    /// Mark a module as finished importing (successful or failed).
    pub fn exit(&mut self, key: &str) {
        self.importing.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalise() {
        assert_eq!(ModuleResolver::normalise("utils/math"), "utils.math");
        assert_eq!(ModuleResolver::normalise("utils.math"), "utils.math");
        assert_eq!(ModuleResolver::normalise("utils\\math"), "utils.math");
        assert_eq!(ModuleResolver::normalise("std.otura"), "std.otura");
    }

    #[test]
    fn test_is_std() {
        assert!(ModuleResolver::is_std("std.otura"));
        assert!(ModuleResolver::is_std("std/otura"));
        assert!(ModuleResolver::is_std("std"));
        assert!(!ModuleResolver::is_std("utils.math"));
    }

    #[test]
    fn test_import_guard_circular() {
        let mut guard = ImportGuard::new();
        assert!(guard.enter("utils.math").is_ok());
        assert!(guard.enter("utils.math").is_err()); // circular
        guard.exit("utils.math");
        assert!(guard.enter("utils.math").is_ok()); // ok again
    }

    #[test]
    fn test_resolve_not_found() {
        let resolver = ModuleResolver::new(vec![PathBuf::from("/nonexistent")]);
        let result = resolver.resolve("utils.math");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("utils.math"));
    }
}
