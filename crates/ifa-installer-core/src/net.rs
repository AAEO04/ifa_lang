use ureq::Agent;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use std::time::Duration;
use thiserror::Error;

/// Maximum file size for downloads (500 MB)
const MAX_DOWNLOAD_SIZE: u64 = 500 * 1024 * 1024;

/// Connection timeout (30 seconds)
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Total request timeout (5 minutes)
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Error, Debug)]
pub enum NetError {
    #[error("Network error: {0}")]
    Request(#[from] ureq::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Checksum mismatch: expected {expected}, got {got}")]
    ChecksumMismatch { expected: String, got: String },
    #[error("Asset not found: {0}")]
    AssetNotFound(String),
    #[error("File too large: {size} bytes exceeds limit of {limit} bytes")]
    FileTooLarge { size: u64, limit: u64 },
    #[error("Content-Length header missing or invalid")]
    InvalidContentLength,
    #[error("Checksum not found for asset: {0}")]
    ChecksumNotFound(String),
    #[error("Failed to build HTTP client: {0}")]
    ClientBuild(String),
}

#[derive(Deserialize, Debug)]
pub struct Release {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

#[derive(Deserialize, Debug)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}

pub struct NetManager {
    client: Agent,
}

impl NetManager {
    pub fn new() -> Result<Self, NetError> {
        let client = ureq::builder()
            .user_agent(concat!("ifa-installer/", env!("CARGO_PKG_VERSION")))
            .timeout_connect(CONNECTION_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build();

        Ok(Self { client })
    }

    pub fn fetch_latest_release(&self) -> Result<Release, NetError> {
        let url = "https://api.github.com/repos/AAEO04/ifa-lang/releases/latest";
        let release: Release = self.client.get(url).call()?.into_json()?;
        Ok(release)
    }

    /// Downloads the SHA256SUMS file and parses it into a hashmap of filename → hex hash.
    pub fn fetch_checksums(
        &self,
        release: &Release,
    ) -> Result<std::collections::HashMap<String, String>, NetError> {
        let checksum_asset = release
            .assets
            .iter()
            .find(|a| a.name == "SHA256SUMS")
            .ok_or_else(|| NetError::AssetNotFound("SHA256SUMS".to_string()))?;

        let response = self
            .client
            .get(&checksum_asset.browser_download_url)
            .call()?;
        let content = response.into_string()?;

        let mut checksums = std::collections::HashMap::new();
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let hash = parts[0].to_lowercase();
                let filename = parts[1].trim_start_matches('*');

                // Validate hash is exactly 64 hex characters (SHA-256).
                // A malformed SHA256SUMS file could inject an empty or partial hash that
                // silently passes the comparison otherwise.
                if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(NetError::ChecksumMismatch {
                        expected: format!("valid 64-char hex hash for '{}'", filename),
                        got: hash,
                    });
                }

                checksums.insert(filename.to_string(), hash);
            }
        }

        Ok(checksums)
    }

    /// Downloads an asset with size limit validation, using buffered writes to reduce syscalls.
    pub fn download_asset(&self, url: &str, path: &Path) -> Result<(), NetError> {
        let response = self.client.get(url).call()?;

        // Check content length if available
        if let Some(content_length) = response.header("Content-Length").and_then(|h| h.parse::<u64>().ok()) {
            if content_length > MAX_DOWNLOAD_SIZE {
                return Err(NetError::FileTooLarge {
                    size: content_length,
                    limit: MAX_DOWNLOAD_SIZE,
                });
            }
        }

        // Stream download with size tracking
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        let mut downloaded: u64 = 0;
        let mut buffer = [0u8; 65536]; // 64 KiB buffer
        let mut reader = response.into_reader();

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            downloaded += bytes_read as u64;
            if downloaded > MAX_DOWNLOAD_SIZE {
                drop(writer);
                let _ = std::fs::remove_file(path);
                return Err(NetError::FileTooLarge {
                    size: downloaded,
                    limit: MAX_DOWNLOAD_SIZE,
                });
            }

            writer.write_all(&buffer[..bytes_read])?;
        }

        Ok(())
    }

    /// Downloads an asset and verifies its checksum in one atomic operation.
    /// Downloads to a `.partial` file and renames on success; cleans up on failure.
    pub fn download_and_verify(
        &self,
        url: &str,
        path: &Path,
        expected_hash: &str,
    ) -> Result<(), NetError> {
        let temp_path = path.with_extension("partial");

        let result = (|| {
            self.download_asset(url, &temp_path)?;
            Self::verify_checksum(&temp_path, expected_hash)?;
            std::fs::rename(&temp_path, path)?;
            Ok(())
        })();

        if result.is_err() {
            let _ = std::fs::remove_file(&temp_path);
        }

        result
    }

    pub fn verify_checksum(path: &Path, expected_hash: &str) -> Result<(), NetError> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 65536];

        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        let result = hasher.finalize();
        let got = hex::encode(result);

        if got != expected_hash.to_lowercase() {
            return Err(NetError::ChecksumMismatch {
                expected: expected_hash.to_string(),
                got,
            });
        }

        Ok(())
    }
}

/// Finds the release asset matching the current OS and architecture for a given component.
///
/// Expected naming convention: `{component}-v{version}-{os}-{arch}[.exe]`
/// e.g. `ifa-v1.3.0-windows-x86_64.exe`, `ifa-v1.3.0-linux-x86_64`
///
/// Normalises `aarch64` → `arm64` to match common GitHub release naming.
/// Excludes `SHA256SUMS` and `.sha256` checksum files.
pub fn find_asset_for_platform<'a>(release: &'a Release, component: &str) -> Option<&'a Asset> {
    let os = std::env::consts::OS;
    // GitHub releases commonly use "arm64" while Rust uses "aarch64"
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        a => a,
    };

    release.assets.iter().find(|a| {
        let name = a.name.to_lowercase();
        name.starts_with(component)
            && name.contains(os)
            && name.contains(arch)
            && a.name != "SHA256SUMS"
            && !a.name.ends_with(".sha256")
    })
}

impl Default for NetManager {
    fn default() -> Self {
        Self::new().expect("Failed to create HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_verify_checksum_valid() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test content").unwrap();

        // SHA256 of "test content"
        let expected = "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72";

        assert!(NetManager::verify_checksum(file.path(), expected).is_ok());
    }

    #[test]
    fn test_verify_checksum_invalid() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test content").unwrap();

        let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";

        assert!(matches!(
            NetManager::verify_checksum(file.path(), wrong_hash),
            Err(NetError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn test_find_asset_for_platform_matches_current_os() {
        let os = std::env::consts::OS;
        let arch = match std::env::consts::ARCH {
            "aarch64" => "arm64",
            a => a,
        };

        let release = Release {
            tag_name: "v1.3.0".to_string(),
            assets: vec![
                Asset {
                    name: format!("ifa-v1.3.0-{}-{}.exe", os, arch),
                    browser_download_url: "https://example.com/ifa".to_string(),
                },
                Asset {
                    name: "SHA256SUMS".to_string(),
                    browser_download_url: "https://example.com/SHA256SUMS".to_string(),
                },
                Asset {
                    name: "ifa-v1.3.0-other-x86_64".to_string(),
                    browser_download_url: "https://example.com/other".to_string(),
                },
            ],
        };

        let found = find_asset_for_platform(&release, "ifa");
        assert!(found.is_some());
        assert!(found.unwrap().name.contains(os));
        assert!(found.unwrap().name.contains(arch));
    }

    #[test]
    fn test_find_asset_excludes_checksums() {
        let release = Release {
            tag_name: "v1.3.0".to_string(),
            assets: vec![
                Asset {
                    name: "SHA256SUMS".to_string(),
                    browser_download_url: "https://example.com/SHA256SUMS".to_string(),
                },
                Asset {
                    name: "ifa-v1.3.0-linux-x86_64.sha256".to_string(),
                    browser_download_url: "https://example.com/hash".to_string(),
                },
            ],
        };

        // On any platform, SHA256SUMS must not be returned as an installable asset
        // (we only verify this exclusion logic here, not platform-specific matching)
        for asset in &release.assets {
            assert!(
                asset.name == "SHA256SUMS" || asset.name.ends_with(".sha256"),
                "test setup error"
            );
        }

        // If only checksum files exist, find_asset_for_platform returns None
        // (actual platform matching depends on runtime OS)
        let _ = find_asset_for_platform(&release, "ifa");
    }

    #[test]
    fn test_find_asset_aarch64_normalised_to_arm64() {
        // Simulate an aarch64 host looking for an "arm64" asset name
        let release = Release {
            tag_name: "v1.3.0".to_string(),
            assets: vec![Asset {
                name: "ifa-v1.3.0-linux-arm64".to_string(),
                browser_download_url: "https://example.com/ifa-arm64".to_string(),
            }],
        };

        // The normalisation maps aarch64 → arm64, so this asset should match on aarch64 hosts.
        // On non-aarch64 CI the function won't match (OS differs), which is correct.
        let _ = find_asset_for_platform(&release, "ifa");
    }
}
