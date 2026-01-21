use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
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
    Request(#[from] reqwest::Error),
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
    client: Client,
}

impl NetManager {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("ifa-installer/1.0")
                .connect_timeout(CONNECTION_TIMEOUT)
                .timeout(REQUEST_TIMEOUT)
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub fn fetch_latest_release(&self) -> Result<Release, NetError> {
        let url = "https://api.github.com/repos/AAEO04/ifa-lang/releases/latest";
        let release = self.client.get(url).send()?.json::<Release>()?;
        Ok(release)
    }

    /// Downloads the SHA256SUMS file and parses it into a hashmap
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
            .send()?;
        let content = response.text()?;

        let mut checksums = std::collections::HashMap::new();
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let hash = parts[0].to_lowercase();
                let filename = parts[1].trim_start_matches('*');
                checksums.insert(filename.to_string(), hash);
            }
        }

        Ok(checksums)
    }

    /// Downloads an asset with size limit validation
    pub fn download_asset(&self, url: &str, path: &Path) -> Result<(), NetError> {
        let response = self.client.get(url).send()?;

        // Check content length if available
        if let Some(content_length) = response.content_length()
            && content_length > MAX_DOWNLOAD_SIZE
        {
            return Err(NetError::FileTooLarge {
                size: content_length,
                limit: MAX_DOWNLOAD_SIZE,
            });
        }

        // Stream download with size tracking
        let mut file = File::create(path)?;
        let mut downloaded: u64 = 0;
        let mut buffer = [0u8; 8192];
        let mut reader = response;

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            downloaded += bytes_read as u64;
            if downloaded > MAX_DOWNLOAD_SIZE {
                // Clean up partial download
                drop(file);
                let _ = std::fs::remove_file(path);
                return Err(NetError::FileTooLarge {
                    size: downloaded,
                    limit: MAX_DOWNLOAD_SIZE,
                });
            }

            file.write_all(&buffer[..bytes_read])?;
        }

        Ok(())
    }

    /// Downloads an asset and verifies its checksum in one operation
    pub fn download_and_verify(
        &self,
        url: &str,
        path: &Path,
        expected_hash: &str,
    ) -> Result<(), NetError> {
        // Download to temporary location first
        let temp_path = path.with_extension("partial");

        // Ensure cleanup on error
        let result = (|| {
            self.download_asset(url, &temp_path)?;
            Self::verify_checksum(&temp_path, expected_hash)?;
            std::fs::rename(&temp_path, path)?;
            Ok(())
        })();

        // Clean up temp file on error
        if result.is_err() {
            let _ = std::fs::remove_file(&temp_path);
        }

        result
    }

    pub fn verify_checksum(path: &Path, expected_hash: &str) -> Result<(), NetError> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        let result = hasher.finalize();
        let got = hex::encode(result);

        if got.to_lowercase() != expected_hash.to_lowercase() {
            return Err(NetError::ChecksumMismatch {
                expected: expected_hash.to_string(),
                got,
            });
        }

        Ok(())
    }
}

impl Default for NetManager {
    fn default() -> Self {
        Self::new()
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
}
