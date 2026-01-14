//! # Storage Infrastructure (The Seal)
//!
//! High-performance I/O and Native Persistence.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncSeekExt, SeekFrom};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::Serialize;

/// Error type for Storage
#[derive(Debug)]
pub enum StorageError {
    Io(std::io::Error),
    Serialization(bincode::Error),
    KeyNotFound,
}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self { StorageError::Io(e) }
}
impl From<bincode::Error> for StorageError {
    fn from(e: bincode::Error) -> Self { StorageError::Serialization(e) }
}

/// The Odù Store: A Native, Append-Only Key-Value DB.
///
/// Format: [KeyLen: u64][Key: bytes][ValLen: u64][Val: bytes]
pub struct OduStore {
    path: PathBuf,
    index: HashMap<String, u64>, // Key -> Offset in file
    file: Arc<Mutex<File>>,
}

impl OduStore {
    /// Open or create a store at the given path
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .await?;


        // Rebuild index from log (Placeholder)
        let index = HashMap::new();
        let _pos = 0;
        let _len = file.metadata().await?.len();
        
        // Simple replay (optimization: check checksums in future)
        // TODO: Full replay implementation needed for durability
        // For now, we just open.
        
        Ok(OduStore {
            path,
            index,
            file: Arc::new(Mutex::new(file)),
        })
    }

    /// Set a key-value pair
    pub async fn set<V: Serialize>(&mut self, key: &str, value: &V) -> Result<(), StorageError> {
        let serialized = bincode::serialize(value)?;
        let key_bytes = key.as_bytes();
        
        let mut file = self.file.lock().await;
        // Seek to end
        let offset = file.seek(SeekFrom::End(0)).await?;
        
        // Write: | KeyLen | Key | ValLen | Val |
        file.write_u64_le(key_bytes.len() as u64).await?;
        file.write_all(key_bytes).await?;
        file.write_u64_le(serialized.len() as u64).await?;
        file.write_all(&serialized).await?;
        file.flush().await?;
        
        // Update index (In-memory)
        // Note: Real LSM usage would compact this. 
        // Here we just map Key to the start of the RECORD. 
        // Wait, efficient read needs to know where Value starts, or we read Key again.
        // Let's store offset of Value for O(1) read.
        // Actually, let's keep it simple: Offset points to start of record.
        self.index.insert(key.to_string(), offset);
        
        Ok(())
    }



    /// Get a value
    pub async fn get<V: serde::de::DeserializeOwned>(&self, key: &str) -> Result<V, StorageError> {
        let offset = self.index.get(key).ok_or(StorageError::KeyNotFound)?;
        
        let mut file = self.file.lock().await;
        file.seek(SeekFrom::Start(*offset)).await?;
        
        // Read KeyLen
        let key_len = file.read_u64_le().await?;
        // Skip Key
        file.seek(SeekFrom::Current(key_len as i64)).await?;
        
        // Read ValLen
        let val_len = file.read_u64_le().await?;
        
        // Read Value
        let mut buffer = vec![0u8; val_len as usize];
        file.read_exact(&mut buffer).await?;
        
        let value = bincode::deserialize(&buffer)?;
        Ok(value)
    }
}
