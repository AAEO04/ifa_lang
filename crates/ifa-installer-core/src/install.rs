use crate::check::check_system;
use crate::config::InstallConfig;
use crate::net::NetManager;
use crate::profiles::Component;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// RAII transaction guard for installation rollback
/// Files are tracked and automatically cleaned up if not committed
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

    fn untrack_last(&mut self) {
        self.files.pop();
    }

    fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for InstallTransaction {
    fn drop(&mut self) {
        if !self.committed {
            println!(
                "[Rollback] Cleaning up {} partial files...",
                self.files.len()
            );
            for file in &self.files {
                if file.exists()
                    && let Err(e) = fs::remove_file(file)
                {
                    eprintln!("[Rollback] Failed to remove {:?}: {}", file, e);
                }
            }
        }
    }
}

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

/// Minimum available disk space required (1 GB)
const MIN_DISK_GB: u64 = 1;
/// Minimum total memory required (2 GB)
const MIN_MEMORY_GB: u64 = 2;

pub fn install(config: &InstallConfig, components: &[Component]) -> Result<(), InstallError> {
    // 0. Pre-installation Checks
    println!("Performing system checks...");
    let sys = check_system();

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
        "✓ System requirements met (Memory: {}GB, Disk: {}GB available)",
        sys.total_memory_gb, sys.available_disk_gb
    );

    let net = NetManager::new();

    // RAII transaction - automatically rolls back on drop if not committed
    let mut txn = InstallTransaction::new();

    // 1. Create install directory
    if !config.install_dir.exists() {
        fs::create_dir_all(&config.install_dir)?;
    }

    // Wrap installation logic to ensure rollback on error
    let result = (|| {
        // 2. Offline Sidecar Check & Metadata
        let current_exe = std::env::current_exe().unwrap_or_default();
        let exe_dir = current_exe.parent().unwrap_or(Path::new("."));

        // Fetch release metadata and checksums
        let (release_metadata, checksums) = if config.offline_mode {
            (None, None)
        } else {
            println!("Fetching release metadata...");
            match net.fetch_latest_release() {
                Ok(release) => {
                    println!("Fetching checksums for verification...");
                    let checksums = match net.fetch_checksums(&release) {
                        Ok(c) => {
                            println!("✓ Downloaded SHA256SUMS ({} entries)", c.len());
                            Some(c)
                        }
                        Err(e) => {
                            println!("⚠ Warning: Could not fetch checksums: {}", e);
                            println!("  Downloads will not be verified!");
                            None
                        }
                    };
                    (Some(release), checksums)
                }
                Err(e) => {
                    println!("Network error or offline: {}. Trying local assets only.", e);
                    (None, None)
                }
            }
        };

        // 3. Process components
        for component in components {
            if component.selected {
                println!("Installing {}...", component.name);

                // Try to find local sidecar first
                let expected_name = format!(
                    "{}-{}-{}.zip",
                    component.name,
                    std::env::consts::OS,
                    std::env::consts::ARCH
                );
                let local_path = exe_dir.join(&expected_name);

                if local_path.exists() {
                    println!("Found local asset: {:?}", local_path);

                    // Verify local asset if checksums available
                    if let Some(ref checksums) = checksums
                        && let Some(expected_hash) = checksums.get(&expected_name)
                    {
                        println!("Verifying local asset integrity...");
                        NetManager::verify_checksum(&local_path, expected_hash)?;
                        println!("✓ Checksum verified");
                    }

                    crate::extraction::extract(&local_path, &config.install_dir)?;
                    continue;
                }

                // Asset name matches component name directly (e.g., "ifa")
                let asset_name_fragment = component.name.as_str();

                // Fallback to Network with verification
                if let Some(release) = &release_metadata {
                    if let Some(asset) = release
                        .assets
                        .iter()
                        .find(|a| a.name.contains(asset_name_fragment))
                    {
                        let target_path = config.install_dir.join(&asset.name);

                        // Register for rollback before download completes
                        txn.track(target_path.clone());

                        println!("Downloading {} to {:?}...", asset.name, target_path);

                        // Download with verification if checksums available
                        if let Some(ref checksums) = checksums {
                            if let Some(expected_hash) = checksums.get(&asset.name) {
                                println!("🔐 Download will be verified against SHA256SUMS");
                                net.download_and_verify(
                                    &asset.browser_download_url,
                                    &target_path,
                                    expected_hash,
                                )?;
                                println!("✓ Download verified successfully");
                            } else {
                                println!(
                                    "⚠ No checksum found for {}, downloading unverified",
                                    asset.name
                                );
                                net.download_asset(&asset.browser_download_url, &target_path)?;
                            }
                        } else {
                            // No checksums available, download without verification
                            net.download_asset(&asset.browser_download_url, &target_path)?;
                        }

                        crate::extraction::extract(&target_path, &config.install_dir)?;
                        // Clean up downloaded archive
                        let _ = fs::remove_file(&target_path);
                        txn.untrack_last(); // Archive successfully extracted and removed
                    } else {
                        println!(
                            "Warning: Could not find asset for component {} (looked for '{}')",
                            component.name, asset_name_fragment
                        );
                    }
                } else {
                    println!(
                        "Error: No local asset found for {} and cannot fetch metadata (offline or network error).",
                        component.name
                    );
                    if component.required {
                        return Err(
                            crate::net::NetError::AssetNotFound(component.name.clone()).into()
                        );
                    }
                }
            }
        }

        // 4. Update Path (Platform specific)
        if config.add_to_path {
            #[cfg(target_os = "windows")]
            {
                use crate::windows::add_to_path;
                add_to_path(&config.install_dir)
                    .map_err(|e| InstallError::Platform(e.to_string()))?;
            }

            #[cfg(unix)]
            {
                use crate::unix::add_to_path;
                add_to_path(&config.install_dir)
                    .map_err(|e| InstallError::Platform(e.to_string()))?;
            }
        }

        Ok(())
    })();

    // Commit transaction on success (prevents Drop-based rollback)
    result.map(|()| {
        txn.commit();
        println!("✅ Installation complete.");
    })
}
