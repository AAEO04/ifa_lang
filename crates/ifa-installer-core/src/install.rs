use crate::check::check_system_for;
use crate::config::InstallConfig;
use crate::net::{NetManager, find_asset_for_platform};
use crate::profiles::Component;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

// ── RAII transaction ────────────────────────────────────────────────────────

/// Tracks files written during installation and removes them on drop if not committed.
struct InstallTransaction {
    files: Vec<PathBuf>,
    committed: bool,
}

impl InstallTransaction {
    fn new() -> Self {
        Self {
            files: Vec::new(),
            committed: false,
        }
    }

    fn track(&mut self, path: PathBuf) {
        self.files.push(path);
    }

    /// Remove a specific path from the tracked list (by value, not position).
    fn untrack(&mut self, path: &Path) {
        self.files.retain(|p| p != path);
    }

    fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for InstallTransaction {
    fn drop(&mut self) {
        if !self.committed {
            println!(
                "[Rollback] Cleaning up {} partial file(s)...",
                self.files.len()
            );
            for file in &self.files {
                if file.exists() {
                    if let Err(e) = fs::remove_file(file) {
                        eprintln!("[Rollback] Failed to remove {:?}: {}", file, e);
                    }
                }
            }
        }
    }
}

// ── Error type ──────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum InstallError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Download error: {0}")]
    Download(#[from] crate::net::NetError),
    #[error("Extraction error: {0}")]
    Extraction(#[from] crate::extraction::ExtractionError),
    #[error("Platform error: {0}")]
    Platform(String),
    #[error("Security verification failed: {0}")]
    VerificationFailed(String),
    #[error("System requirements not met: {0}")]
    RequirementsNotMet(String),
}

// ── Constants ───────────────────────────────────────────────────────────────

/// Minimum available disk space required (1 GB)
const MIN_DISK_GB: u64 = 1;
/// Minimum total memory required (2 GB)
const MIN_MEMORY_GB: u64 = 2;

// ── Asset helpers ────────────────────────────────────────────────────────────

/// Returns true if the path looks like an archive (zip / tar.gz / tgz).
fn is_archive(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("zip") | Some("gz") | Some("tgz")
    )
}

/// Derives the final binary name from a versioned release asset filename.
///
/// Strips the version tag and platform suffix so the binary is installed with
/// a clean name regardless of the release tag:
/// - `ifa-v1.3.0-windows-x86_64.exe` → `ifa.exe`
/// - `ifa-v1.3.0-linux-x86_64`       → `ifa`
/// - `ifa-v1.3.0-macos-arm64`        → `ifa`
fn derive_binary_name(asset_path: &Path) -> String {
    let has_exe_ext = asset_path
        .extension()
        .map(|e| e.eq_ignore_ascii_case("exe"))
        .unwrap_or(false);

    let stem = asset_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("ifa");

    // Split on the first "-v" to discard the version+platform suffix.
    // "ifa-v1.3.0-linux-x86_64" → "ifa"
    let base = stem.split("-v").next().unwrap_or(stem);

    if has_exe_ext || cfg!(windows) {
        format!("{}.exe", base)
    } else {
        base.to_string()
    }
}

/// Sets the executable bit on Unix; no-op on other platforms.
fn set_executable(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;
    }
    let _ = path; // silence unused warning on non-Unix
    Ok(())
}

/// Installs a single downloaded asset into the install directory.
///
/// - Archives (zip/tar.gz) are extracted into `install_dir`.
/// - Bare binaries (.exe, no extension) are copied into `install_dir/bin/`
///   with a clean name (version tag stripped) and made executable.
fn install_asset(asset_path: &Path, install_dir: &Path) -> Result<(), InstallError> {
    let bin_dir = install_dir.join("bin");
    fs::create_dir_all(&bin_dir)?;

    if is_archive(asset_path) {
        crate::extraction::extract(asset_path, install_dir)?;
    } else {
        let bin_name = derive_binary_name(asset_path);
        let dest = bin_dir.join(&bin_name);
        fs::copy(asset_path, &dest)?;
        set_executable(&dest)?;
        println!("  → Installed binary: {:?}", dest);
    }
    Ok(())
}

/// Searches the directory adjacent to the installer executable for a local sidecar
/// matching the current component + OS + arch. Handles both versioned names
/// (`ifa-v1.3.0-linux-x86_64`) and bare names (`ifa-linux-x86_64.zip`).
fn find_local_asset(exe_dir: &Path, component: &str) -> Option<PathBuf> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let arch_alt = match arch {
        "aarch64" => "arm64",
        a => a,
    };

    // Scan the directory for any file that matches component + OS + (arch or arch_alt)
    if let Ok(entries) = fs::read_dir(exe_dir) {
        let mut candidates: Vec<PathBuf> = entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_lowercase();
                if name.starts_with(component)
                    && name.contains(os)
                    && (name.contains(arch) || name.contains(arch_alt))
                    && name != "sha256sums"
                    && !name.ends_with(".sha256")
                {
                    Some(e.path())
                } else {
                    None
                }
            })
            .collect();

        // Prefer versioned names (contain "-v") over bare names — they are more specific.
        candidates.sort_by_key(|p| {
            !p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.contains("-v"))
                .unwrap_or(false)
        });

        return candidates.into_iter().next();
    }

    None
}

// ── Version / marker files ───────────────────────────────────────────────────

fn write_version_file(install_dir: &Path, version: &str) -> std::io::Result<()> {
    let content = format!(
        "version = \"{}\"\nos = \"{}\"\narch = \"{}\"\n",
        version,
        std::env::consts::OS,
        std::env::consts::ARCH,
    );
    fs::write(install_dir.join(".ifa-version"), content)
}

fn write_marker_file(install_dir: &Path) -> std::io::Result<()> {
    // Empty sentinel file that `uninstall` checks before deleting the directory.
    fs::write(install_dir.join(".ifa-installed"), b"")
}

// ── Self-update ──────────────────────────────────────────────────────────────

/// Updates an existing installation to the latest release.
///
/// Reads the current version from `.ifa-version`, fetches the latest GitHub release,
/// and if a newer version is available downloads it atomically (write `.new`, rename over
/// existing binary).
pub fn self_update(install_dir: &Path) -> Result<(), InstallError> {
    let version_path = install_dir.join(".ifa-version");
    let current_version = if version_path.exists() {
        let content = fs::read_to_string(&version_path)?;
        content
            .lines()
            .find(|l| l.starts_with("version"))
            .and_then(|l| l.split('"').nth(1))
            .unwrap_or("unknown")
            .to_string()
    } else {
        "unknown".to_string()
    };

    println!("Current version: {}", current_version);
    println!("Checking for updates...");

    let net = NetManager::new()?;
    let release = net.fetch_latest_release()?;

    if current_version == release.tag_name {
        println!("✓ Already up to date ({})", current_version);
        return Ok(());
    }

    println!("New version available: {}", release.tag_name);

    let asset = find_asset_for_platform(&release, "ifa").ok_or_else(|| {
        InstallError::RequirementsNotMet(format!(
            "No asset found for {}/{} in release {}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            release.tag_name,
        ))
    })?;

    let checksums = net.fetch_checksums(&release).ok();

    let bin_dir = install_dir.join("bin");
    let bin_name = derive_binary_name(Path::new(&asset.name));
    let final_path = bin_dir.join(&bin_name);
    let temp_path = final_path.with_extension("new");

    println!("Downloading {}...", asset.name);

    if let Some(ref cs) = checksums {
        if let Some(expected) = cs.get(&asset.name) {
            net.download_and_verify(&asset.browser_download_url, &temp_path, expected)?;
            println!("✓ Checksum verified");
        } else {
            println!(
                "⚠ No checksum entry for {}, downloading unverified",
                asset.name
            );
            net.download_asset(&asset.browser_download_url, &temp_path)?;
        }
    } else {
        net.download_asset(&asset.browser_download_url, &temp_path)?;
    }

    set_executable(&temp_path)?;

    // Atomic replace: rename over the existing binary.
    fs::rename(&temp_path, &final_path)?;

    write_version_file(install_dir, &release.tag_name)?;
    println!("✅ Updated to {}", release.tag_name);

    Ok(())
}

// ── Main install entry point ─────────────────────────────────────────────────

pub fn install(config: &InstallConfig, components: &[Component]) -> Result<(), InstallError> {
    // 0. Pre-installation system check
    println!("Performing system checks...");
    let sys = check_system_for(&config.install_dir);

    if sys.available_disk_gb < MIN_DISK_GB {
        return Err(InstallError::RequirementsNotMet(format!(
            "Insufficient disk space. Need {}GB, have {}GB",
            MIN_DISK_GB, sys.available_disk_gb
        )));
    }

    if sys.total_memory_gb < MIN_MEMORY_GB {
        return Err(InstallError::RequirementsNotMet(format!(
            "Insufficient memory. Need {}GB, have {}GB",
            MIN_MEMORY_GB, sys.total_memory_gb
        )));
    }

    println!(
        "✓ System OK (Memory: {}GB, Disk: {}GB available on install target)",
        sys.total_memory_gb, sys.available_disk_gb
    );

    if !sys.rustc_available {
        println!();
        println!("ℹ  Note: `rustc` was not found on PATH.");
        println!("   The Ifá-Lang runtime and interpreter work without Rust.");
        println!("   However, `ifa build` (transpiler → native binary) requires Rust.");
        println!("   Install Rust at https://rustup.rs if you need `ifa build`.");
        println!();
    }

    // 1. Create install directory and bin/ subdirectory
    let bin_dir = config.bin_dir();
    fs::create_dir_all(&bin_dir)?;

    let net = NetManager::new()?;

    // RAII transaction — files tracked here are removed on drop if not committed
    let mut txn = InstallTransaction::new();

    let result: Result<Option<String>, InstallError> = (|| {
        // 2. Fetch release metadata and checksums
        let current_exe = std::env::current_exe().unwrap_or_default();
        let exe_dir = current_exe.parent().unwrap_or(Path::new("."));

        let (release_metadata, checksums) = if config.offline_mode {
            (None, None)
        } else {
            println!("Fetching release metadata...");
            match net.fetch_latest_release() {
                Ok(release) => {
                    println!("  Latest release: {}", release.tag_name);
                    println!("Fetching checksums for verification...");
                    let checksums = match net.fetch_checksums(&release) {
                        Ok(c) => {
                            println!("✓ Downloaded SHA256SUMS ({} entries)", c.len());
                            Some(c)
                        }
                        Err(e) => {
                            println!("⚠ Could not fetch checksums: {}", e);
                            println!("  Downloads will not be cryptographically verified.");
                            None
                        }
                    };
                    let tag = release.tag_name.clone();
                    (Some((release, tag)), checksums)
                }
                Err(e) => {
                    println!("Network unavailable: {}. Trying local sidecar only.", e);
                    (None, None)
                }
            }
        };

        let release_tag = release_metadata.as_ref().map(|(_, t)| t.clone());

        // 3. Process each component
        for component in components {
            if !component.selected {
                continue;
            }

            println!("Installing {}...", component.name);

            // Phase 1: local sidecar (next to the installer binary)
            if let Some(local_path) = find_local_asset(exe_dir, &component.name) {
                println!("  Found local asset: {:?}", local_path);

                if let Some(ref cs) = checksums {
                    let fname = local_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if let Some(expected_hash) = cs.get(&fname) {
                        println!("  Verifying local asset integrity...");
                        NetManager::verify_checksum(&local_path, expected_hash)?;
                        println!("  ✓ Checksum verified");
                    }
                }

                install_asset(&local_path, &config.install_dir)?;
                continue;
            }

            // Phase 2: network download
            if let Some((ref release, _)) = release_metadata {
                if let Some(asset) = find_asset_for_platform(release, &component.name) {
                    // Download to a temp path inside install_dir (same filesystem → atomic rename)
                    let temp_path = config.install_dir.join(&asset.name);
                    txn.track(temp_path.clone());

                    println!("  Downloading {}...", asset.name);

                    if let Some(ref cs) = checksums {
                        if let Some(expected_hash) = cs.get(&asset.name) {
                            println!("  🔐 Verifying against SHA256SUMS...");
                            net.download_and_verify(
                                &asset.browser_download_url,
                                &temp_path,
                                expected_hash,
                            )?;
                            println!("  ✓ Download verified");
                        } else {
                            println!(
                                "  ⚠ No checksum entry for {}, downloading unverified",
                                asset.name
                            );
                            net.download_asset(&asset.browser_download_url, &temp_path)?;
                        }
                    } else {
                        net.download_asset(&asset.browser_download_url, &temp_path)?;
                    }

                    install_asset(&temp_path, &config.install_dir)?;
                    let _ = fs::remove_file(&temp_path);
                    txn.untrack(&temp_path);
                } else {
                    let msg = format!(
                        "No release asset found for component '{}' on {}/{}",
                        component.name,
                        std::env::consts::OS,
                        std::env::consts::ARCH
                    );
                    if component.required {
                        return Err(InstallError::RequirementsNotMet(msg));
                    } else {
                        println!("  ⚠ {}", msg);
                    }
                }
            } else {
                let msg = format!(
                    "No local asset and no network access for '{}'",
                    component.name
                );
                if component.required {
                    return Err(InstallError::RequirementsNotMet(msg));
                } else {
                    println!("  ⚠ {}", msg);
                }
            }
        }

        Ok(release_tag)
    })();

    // 4. On success: update PATH, write marker and version files, commit transaction
    result.map(|release_tag| {
        // Add bin/ to PATH (not install_dir itself — bin/ is where the binary lives)
        if config.add_to_path {
            #[cfg(target_os = "windows")]
            {
                use crate::windows::add_to_path;
                if let Err(e) = add_to_path(&bin_dir) {
                    eprintln!("⚠ Could not update PATH: {}", e);
                }
            }

            #[cfg(unix)]
            {
                use crate::unix::add_to_path;
                if let Err(e) = add_to_path(&bin_dir) {
                    eprintln!("⚠ Could not update PATH: {}", e);
                }
            }
        }

        // Write marker file — checked by uninstall to prevent accidental directory deletion
        if let Err(e) = write_marker_file(&config.install_dir) {
            eprintln!("⚠ Could not write .ifa-installed marker: {}", e);
        }

        // Write version file if we know the installed version
        if let Some(tag) = release_tag {
            if let Err(e) = write_version_file(&config.install_dir, &tag) {
                eprintln!("⚠ Could not write .ifa-version: {}", e);
            }
        }

        txn.commit();
        println!("✅ Installation complete.");
        println!("   Run 'ifa --version' to verify (you may need to restart your terminal first).");
    })
}

// ── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_binary_name_windows_versioned() {
        let path = Path::new("ifa-v1.3.0-windows-x86_64.exe");
        assert_eq!(derive_binary_name(path), "ifa.exe");
    }

    #[test]
    fn test_derive_binary_name_linux_versioned() {
        let path = Path::new("ifa-v1.3.0-linux-x86_64");
        // On non-Windows hosts: no .exe suffix
        let result = derive_binary_name(path);
        if cfg!(windows) {
            assert_eq!(result, "ifa.exe");
        } else {
            assert_eq!(result, "ifa");
        }
    }

    #[test]
    fn test_derive_binary_name_macos_arm() {
        let path = Path::new("ifa-v2.0.0-macos-arm64");
        let result = derive_binary_name(path);
        if cfg!(windows) {
            assert_eq!(result, "ifa.exe");
        } else {
            assert_eq!(result, "ifa");
        }
    }

    #[test]
    fn test_is_archive() {
        assert!(is_archive(Path::new("foo.zip")));
        assert!(is_archive(Path::new("foo.tar.gz")));
        assert!(is_archive(Path::new("foo.tgz")));
        assert!(!is_archive(Path::new("foo.exe")));
        assert!(!is_archive(Path::new("foo")));
    }
}
