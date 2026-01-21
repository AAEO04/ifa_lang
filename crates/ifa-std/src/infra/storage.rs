//! # Storage Infrastructure (The Seal)
//!
//! High-performance I/O and Native Persistence with production-grade durability.

use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::sync::Mutex;

/// Maximum allowed key size (1 MB)
pub const MAX_KEY_SIZE: u64 = 1024 * 1024;
/// Maximum allowed value size (100 MB)
pub const MAX_VALUE_SIZE: u64 = 100 * 1024 * 1024;
/// Magic bytes for file format validation
const MAGIC: &[u8; 4] = b"IFAS";
/// Current file format version
const VERSION: u8 = 2;

/// Error type for Storage operations
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Key not found")]
    KeyNotFound,

    #[error("Key size exceeds maximum ({0} > {MAX_KEY_SIZE})")]
    KeyTooLarge(u64),

    #[error("Value size exceeds maximum ({0} > {MAX_VALUE_SIZE})")]
    ValueTooLarge(u64),

    #[error("Corrupted log: {0}")]
    CorruptedLog(String),

    #[error("Checksum mismatch: expected {expected:08x}, got {actual:08x}")]
    ChecksumMismatch { expected: u32, actual: u32 },

    #[error("Invalid file format")]
    InvalidFormat,
}

/// Record type for the log
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum RecordType {
    Put = 1,
    Delete = 2,
}

impl TryFrom<u8> for RecordType {
    type Error = StorageError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(RecordType::Put),
            2 => Ok(RecordType::Delete),
            _ => Err(StorageError::CorruptedLog(format!(
                "Invalid record type: {}",
                value
            ))),
        }
    }
}

/// CRC32 checksum computation
fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB88320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// The Odù Store: A Native, Append-Only Key-Value DB.
///
/// Format v2: [MAGIC:4][VERSION:1][Records...]
/// Record: [CRC32:4][Type:1][KeyLen:4][Key][ValLen:4][Val]
pub struct OduStore {
    path: PathBuf,
    index: HashMap<String, u64>, // Key -> Offset in file
    tombstones: std::collections::HashSet<String>, // Deleted keys
    file: Arc<Mutex<File>>,
    record_count: usize,
    stale_count: usize, // Overwritten/deleted records
}

impl OduStore {
    /// Open or create a store at the given path, replaying the log to rebuild the index
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref().to_path_buf();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .await?;

        let file_len = file.metadata().await?.len();

        // Initialize new file with header
        if file_len == 0 {
            file.write_all(MAGIC).await?;
            file.write_u8(VERSION).await?;
            file.flush().await?;
        } else {
            // Validate header
            let mut magic = [0u8; 4];
            file.read_exact(&mut magic).await?;
            if &magic != MAGIC {
                return Err(StorageError::InvalidFormat);
            }
            let version = file.read_u8().await?;
            if version > VERSION {
                return Err(StorageError::CorruptedLog(format!(
                    "Unsupported version: {}",
                    version
                )));
            }
        }

        let file = Arc::new(Mutex::new(file));

        // Rebuild index from log
        let (index, tombstones, record_count, stale_count) =
            Self::rebuild_index(file.clone()).await?;

        Ok(OduStore {
            path,
            index,
            tombstones,
            file,
            record_count,
            stale_count,
        })
    }

    /// Replay the log file to rebuild the in-memory index
    async fn rebuild_index(
        file: Arc<Mutex<File>>,
    ) -> Result<
        (
            HashMap<String, u64>,
            std::collections::HashSet<String>,
            usize,
            usize,
        ),
        StorageError,
    > {
        let mut index = HashMap::new();
        let mut tombstones = std::collections::HashSet::new();
        let mut file_guard = file.lock().await;

        // Skip header
        file_guard.seek(SeekFrom::Start(5)).await?;
        let file_len = file_guard.metadata().await?.len();

        let mut pos: u64 = 5;
        let mut record_count = 0;
        let mut stale_count = 0;

        while pos < file_len {
            let record_start = pos;

            // Read CRC32
            let stored_crc = match file_guard.read_u32_le().await {
                Ok(crc) => crc,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(StorageError::Io(e)),
            };
            pos += 4;

            // Read record type
            let record_type = RecordType::try_from(file_guard.read_u8().await?)?;
            pos += 1;

            // Read KeyLen (u32 for v2)
            let key_len = file_guard.read_u32_le().await? as u64;
            pos += 4;

            // Validate key length
            if key_len > MAX_KEY_SIZE {
                return Err(StorageError::CorruptedLog(format!(
                    "Invalid key length {} at offset {}",
                    key_len, record_start
                )));
            }

            // Read Key
            let key_len_usize = key_len as usize;
            let mut key_bytes = vec![0u8; key_len_usize];
            file_guard.read_exact(&mut key_bytes).await?;
            pos += key_len;

            let key = String::from_utf8(key_bytes.clone())
                .map_err(|_| StorageError::CorruptedLog("Invalid UTF-8 in key".into()))?;

            // For Put records, read value
            let val_bytes = if record_type == RecordType::Put {
                let val_len = file_guard.read_u32_le().await? as u64;
                pos += 4;

                if val_len > MAX_VALUE_SIZE {
                    return Err(StorageError::CorruptedLog(format!(
                        "Invalid value length {} at offset {}",
                        val_len, record_start
                    )));
                }

                let mut val_bytes = vec![0u8; val_len as usize];
                file_guard.read_exact(&mut val_bytes).await?;
                pos += val_len;
                val_bytes
            } else {
                Vec::new()
            };

            // Verify checksum
            let mut check_data = vec![record_type as u8];
            check_data.extend_from_slice(&(key_len as u32).to_le_bytes());
            check_data.extend_from_slice(&key_bytes);
            if record_type == RecordType::Put {
                check_data.extend_from_slice(&(val_bytes.len() as u32).to_le_bytes());
                check_data.extend_from_slice(&val_bytes);
            }

            let computed_crc = crc32(&check_data);
            if computed_crc != stored_crc {
                return Err(StorageError::ChecksumMismatch {
                    expected: stored_crc,
                    actual: computed_crc,
                });
            }

            // Update index
            record_count += 1;
            match record_type {
                RecordType::Put => {
                    if index.contains_key(&key) || tombstones.contains(&key) {
                        stale_count += 1;
                    }
                    tombstones.remove(&key);
                    index.insert(key, record_start);
                }
                RecordType::Delete => {
                    if index.remove(&key).is_some() {
                        stale_count += 1;
                    }
                    tombstones.insert(key);
                }
            }
        }

        Ok((index, tombstones, record_count, stale_count))
    }

    /// Write a record with CRC32 checksum
    async fn write_record(
        &mut self,
        record_type: RecordType,
        key: &str,
        value: Option<&[u8]>,
    ) -> Result<u64, StorageError> {
        let key_bytes = key.as_bytes();
        let key_len = key_bytes.len() as u32;

        // Build record data for checksum
        let mut check_data = vec![record_type as u8];
        check_data.extend_from_slice(&key_len.to_le_bytes());
        check_data.extend_from_slice(key_bytes);

        if let Some(val) = value {
            check_data.extend_from_slice(&(val.len() as u32).to_le_bytes());
            check_data.extend_from_slice(val);
        }

        let crc = crc32(&check_data);

        let mut file = self.file.lock().await;
        let offset = file.seek(SeekFrom::End(0)).await?;

        // Write: [CRC32][Type][KeyLen][Key][ValLen?][Val?]
        file.write_u32_le(crc).await?;
        file.write_u8(record_type as u8).await?;
        file.write_u32_le(key_len).await?;
        file.write_all(key_bytes).await?;

        if let Some(val) = value {
            file.write_u32_le(val.len() as u32).await?;
            file.write_all(val).await?;
        }

        file.flush().await?;
        self.record_count += 1;

        Ok(offset)
    }

    /// Set a key-value pair
    pub async fn set<V: Serialize>(&mut self, key: &str, value: &V) -> Result<(), StorageError> {
        let serialized = bincode::serialize(value)?;

        // Validate sizes
        let key_len = key.len() as u64;
        let val_len = serialized.len() as u64;

        if key_len > MAX_KEY_SIZE {
            return Err(StorageError::KeyTooLarge(key_len));
        }
        if val_len > MAX_VALUE_SIZE {
            return Err(StorageError::ValueTooLarge(val_len));
        }

        // Track stale entries
        if self.index.contains_key(key) {
            self.stale_count += 1;
        }

        let offset = self
            .write_record(RecordType::Put, key, Some(&serialized))
            .await?;
        self.tombstones.remove(key);
        self.index.insert(key.to_string(), offset);

        Ok(())
    }

    /// Get a value by key
    pub async fn get<V: serde::de::DeserializeOwned>(&self, key: &str) -> Result<V, StorageError> {
        let offset = self.index.get(key).ok_or(StorageError::KeyNotFound)?;

        let mut file = self.file.lock().await;
        file.seek(SeekFrom::Start(*offset)).await?;

        // Skip CRC32
        file.seek(SeekFrom::Current(4)).await?;

        // Skip record type
        let _record_type = file.read_u8().await?;

        // Read and skip key
        let key_len = file.read_u32_le().await?;
        let key_len_i64 = i64::try_from(key_len)
            .map_err(|_| StorageError::CorruptedLog("Key length overflow".into()))?;
        file.seek(SeekFrom::Current(key_len_i64)).await?;

        // Read ValLen
        let val_len = file.read_u32_le().await?;

        // Validate before allocation
        if val_len as u64 > MAX_VALUE_SIZE {
            return Err(StorageError::ValueTooLarge(val_len as u64));
        }

        // Read Value
        let mut buffer = vec![0u8; val_len as usize];
        file.read_exact(&mut buffer).await?;

        let value = bincode::deserialize(&buffer)?;
        Ok(value)
    }

    /// Delete a key
    pub async fn delete(&mut self, key: &str) -> Result<bool, StorageError> {
        if !self.index.contains_key(key) {
            return Ok(false);
        }

        self.write_record(RecordType::Delete, key, None).await?;
        self.index.remove(key);
        self.tombstones.insert(key.to_string());
        self.stale_count += 1;

        Ok(true)
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.index.contains_key(key)
    }

    /// Get all keys
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.index.keys()
    }

    /// Get number of live keys
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Get stale record ratio (for compaction decision)
    pub fn stale_ratio(&self) -> f64 {
        if self.record_count == 0 {
            0.0
        } else {
            self.stale_count as f64 / self.record_count as f64
        }
    }

    /// Compact the log by rewriting only live entries
    ///
    /// Creates a new file, writes only current key-value pairs, then atomically swaps.
    pub async fn compact(&mut self) -> Result<(), StorageError> {
        if self.stale_count == 0 {
            return Ok(()); // Nothing to compact
        }

        let compact_path = self.path.with_extension("compact");

        // Create new compacted file
        let mut compact_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&compact_path)
            .await?;

        // Write header
        compact_file.write_all(MAGIC).await?;
        compact_file.write_u8(VERSION).await?;

        // Copy all live entries
        let mut new_index = HashMap::new();
        let file = self.file.lock().await;

        for (key, &old_offset) in &self.index {
            // Read original record
            let mut file_clone = file.try_clone().await?;
            file_clone.seek(SeekFrom::Start(old_offset + 4)).await?; // Skip CRC

            // Skip record type
            let _record_type = file_clone.read_u8().await?;

            // Read key
            let key_len = file_clone.read_u32_le().await?;
            let mut key_bytes = vec![0u8; key_len as usize];
            file_clone.read_exact(&mut key_bytes).await?;

            // Read value
            let val_len = file_clone.read_u32_le().await?;
            let mut val_bytes = vec![0u8; val_len as usize];
            file_clone.read_exact(&mut val_bytes).await?;

            // Write to compact file
            let new_offset = compact_file.seek(SeekFrom::End(0)).await?;

            // Build record data for checksum
            let mut check_data = vec![RecordType::Put as u8];
            check_data.extend_from_slice(&key_len.to_le_bytes());
            check_data.extend_from_slice(&key_bytes);
            check_data.extend_from_slice(&val_len.to_le_bytes());
            check_data.extend_from_slice(&val_bytes);

            let crc = crc32(&check_data);

            compact_file.write_u32_le(crc).await?;
            compact_file.write_u8(RecordType::Put as u8).await?;
            compact_file.write_u32_le(key_len).await?;
            compact_file.write_all(&key_bytes).await?;
            compact_file.write_u32_le(val_len).await?;
            compact_file.write_all(&val_bytes).await?;

            new_index.insert(key.clone(), new_offset);
        }

        compact_file.flush().await?;
        drop(file);
        drop(compact_file);

        // Atomic swap
        tokio::fs::rename(&compact_path, &self.path).await?;

        // Reopen file
        let new_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.path)
            .await?;

        *self.file.lock().await = new_file;
        self.index = new_index;
        self.tombstones.clear();
        self.record_count = self.index.len();
        self.stale_count = 0;

        Ok(())
    }
}
