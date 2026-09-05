//! Storage Domain (Domain 20) wrapper
//! Bridges the VM dispatch to the async OduStore persistence layer.
//!
//! The storage worker thread owns a `SysRuntime` (from `ifa-infra`) rather than
//! building a raw tokio runtime inline. This ensures the runtime lifecycle is
//! managed through the shared abstraction layer used by the rest of `ifa-infra`.

use ifa_types::value_union::IfaValue;
use ifa_types::{IfaError, IfaResult};

#[cfg(all(feature = "tokio", feature = "persistence"))]
pub enum StorageCmd {
    Open {
        path: String,
        cell: ifa_types::value_union::NativeFutureCell,
    },
    Set {
        id: u64,
        key: String,
        /// The pre-serialized value bytes (bincode(IfaValue)). Passing Vec<u8> ensures
        /// StorageCmd is Send (since IfaValue contains IfaGc which is !Send).
        /// OduStore::set_bytes writes these raw bytes directly into the store so that
        /// OduStore::get::<IfaValue> deserializes them in a single pass.
        val_bytes: Vec<u8>,
        cell: ifa_types::value_union::NativeFutureCell,
    },
    Get {
        id: u64,
        key: String,
        cell: ifa_types::value_union::NativeFutureCell,
    },
    Delete {
        id: u64,
        key: String,
        cell: ifa_types::value_union::NativeFutureCell,
    },
    Compact {
        id: u64,
        cell: ifa_types::value_union::NativeFutureCell,
    },
}

#[derive(Clone)]
#[cfg(all(feature = "tokio", feature = "persistence"))]
pub struct StorageWorker {
    pub sender: std::sync::mpsc::SyncSender<StorageCmd>,
}

#[cfg(all(feature = "tokio", feature = "persistence"))]
impl Default for StorageWorker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(feature = "tokio", feature = "persistence"))]
impl StorageWorker {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::sync_channel::<StorageCmd>(64);

        std::thread::Builder::new()
            .name("ifa-storage".into())
            .spawn(move || {
                use ifa_infra::runtime::SysRuntime;
                use ifa_infra::storage::OduStore;

                // Use the shared SysRuntime abstraction instead of building a raw
                // tokio runtime inline. The .expect() is safe: SysRuntime::new()
                // only fails if tokio's builder fails, which cannot happen here
                // since this block is already gated on feature = "tokio".
                let rt = SysRuntime::new().expect("ifa-storage SysRuntime failed");

                let mut stores: std::collections::HashMap<u64, OduStore> =
                    std::collections::HashMap::new();
                let mut next_id: u64 = 1;

                while let Ok(cmd) = rx.recv() {
                    match cmd {
                        StorageCmd::Open { path, cell } => {
                            // SysRuntime::block_on returns IfaResult<F::Output>.
                            // The outer .expect() panics on runtime failure (impossible here).
                            // The inner match handles OduStore's own Result.
                            let val = match rt
                                .block_on(OduStore::open(&path))
                                .expect("ifa-storage runtime")
                            {
                                Ok(store) => {
                                    let id = next_id;
                                    next_id += 1;
                                    stores.insert(id, store);
                                    IfaValue::int(id as i64)
                                }
                                Err(e) => IfaValue::str(format!("StorageError: {e}")),
                            };
                            // Lock poison handling: if the VM thread panicked while holding
                            // the future lock, recover the inner value so the future resolves.
                            *cell.write().unwrap_or_else(|e| e.into_inner()) =
                                ifa_types::value_union::NativeFutureState::Ready(
                                    bincode::serialize(&val).unwrap(),
                                );
                        }
                        StorageCmd::Get { id, key, cell } => {
                            let val = match stores.get(&id) {
                                Some(store) => {
                                    match rt
                                        .block_on(async { store.get::<IfaValue>(&key).await })
                                        .expect("ifa-storage runtime")
                                    {
                                        Ok(v) => v,
                                        Err(ifa_infra::storage::StorageError::KeyNotFound) => {
                                            IfaValue::null()
                                        }
                                        Err(e) => IfaValue::str(format!("StorageError: {e}")),
                                    }
                                }
                                None => IfaValue::str("StorageError: Invalid store handle"),
                            };
                            // Lock poison handling: recover the inner value if the lock is poisoned.
                            *cell.write().unwrap_or_else(|e| e.into_inner()) =
                                ifa_types::value_union::NativeFutureState::Ready(
                                    bincode::serialize(&val).unwrap(),
                                );
                        }
                        StorageCmd::Set {
                            id,
                            key,
                            val_bytes,
                            cell,
                        } => {
                            let res_val = match stores.get_mut(&id) {
                                Some(store) => {
                                    match rt
                                        .block_on(async { store.set_bytes(&key, &val_bytes).await })
                                        .expect("ifa-storage runtime")
                                    {
                                        Ok(_) => IfaValue::null(),
                                        Err(e) => IfaValue::str(format!("StorageError: {e}")),
                                    }
                                }
                                None => IfaValue::str("StorageError: Invalid store handle"),
                            };
                            // Lock poison handling: if the worker thread panicked, force-write
                            // the Ready state so surviving handles can still access the result.
                            *cell.write().unwrap_or_else(|e| e.into_inner()) =
                                ifa_types::value_union::NativeFutureState::Ready(
                                    bincode::serialize(&res_val).unwrap(),
                                );
                        }
                        StorageCmd::Delete { id, key, cell } => {
                            let val = match stores.get_mut(&id) {
                                Some(store) => {
                                    match rt
                                        .block_on(async { store.delete(&key).await })
                                        .expect("ifa-storage runtime")
                                    {
                                        Ok(_) => IfaValue::null(),
                                        Err(e) => IfaValue::str(format!("StorageError: {e}")),
                                    }
                                }
                                None => IfaValue::str("StorageError: Invalid store handle"),
                            };
                            // Lock poison handling: recover the inner value if the lock is poisoned.
                            *cell.write().unwrap_or_else(|e| e.into_inner()) =
                                ifa_types::value_union::NativeFutureState::Ready(
                                    bincode::serialize(&val).unwrap(),
                                );
                        }
                        StorageCmd::Compact { id, cell } => {
                            let val = match stores.get_mut(&id) {
                                Some(store) => match rt
                                    .block_on(async { store.compact().await })
                                    .expect("ifa-storage runtime")
                                {
                                    Ok(_) => IfaValue::null(),
                                    Err(e) => IfaValue::str(format!("StorageError: {e}")),
                                },
                                None => IfaValue::str("StorageError: Invalid store handle"),
                            };
                            // Lock poison handling: recover the inner value if the lock is poisoned.
                            *cell.write().unwrap_or_else(|e| e.into_inner()) =
                                ifa_types::value_union::NativeFutureState::Ready(
                                    bincode::serialize(&val).unwrap(),
                                );
                        }
                    }
                }
            })
            .expect("Failed to spawn storage thread");

        Self { sender: tx }
    }
}

pub fn dispatch(worker: &StorageWorker, method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    #[cfg(all(feature = "tokio", feature = "persistence"))]
    {
        let cell = std::sync::Arc::new(std::sync::RwLock::new(
            ifa_types::value_union::NativeFutureState::Pending,
        ));

        let cmd = match method {
            "open" => {
                let path = args.first().map(|v| v.to_string()).unwrap_or_default();
                StorageCmd::Open {
                    path,
                    cell: cell.clone(),
                }
            }
            "get" => {
                let id = match args.first() {
                    Some(IfaValue::Int(i)) => *i as u64,
                    _ => {
                        return Err(IfaError::ArgumentError(
                            "storage.get: first arg must be store handle (Int)".into(),
                        ));
                    }
                };
                let key = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                StorageCmd::Get {
                    id,
                    key,
                    cell: cell.clone(),
                }
            }
            "set" => {
                let id = match args.first() {
                    Some(IfaValue::Int(i)) => *i as u64,
                    _ => {
                        return Err(IfaError::ArgumentError(
                            "storage.set: first arg must be store handle (Int)".into(),
                        ));
                    }
                };
                let key = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                let val = args.into_iter().nth(2).unwrap_or(IfaValue::null());
                let val_bytes = bincode::serialize(&val)
                    .map_err(|e| IfaError::Runtime(format!("Storage serialization error: {e}")))?;
                StorageCmd::Set {
                    id,
                    key,
                    val_bytes,
                    cell: cell.clone(),
                }
            }
            "delete" | "del" => {
                let id = match args.first() {
                    Some(IfaValue::Int(i)) => *i as u64,
                    _ => {
                        return Err(IfaError::ArgumentError(
                            "storage.delete: first arg must be store handle (Int)".into(),
                        ));
                    }
                };
                let key = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                StorageCmd::Delete {
                    id,
                    key,
                    cell: cell.clone(),
                }
            }
            "compact" => {
                let id = match args.first() {
                    Some(IfaValue::Int(i)) => *i as u64,
                    _ => {
                        return Err(IfaError::ArgumentError(
                            "storage.compact: first arg must be store handle (Int)".into(),
                        ));
                    }
                };
                StorageCmd::Compact {
                    id,
                    cell: cell.clone(),
                }
            }
            _ => {
                return Err(IfaError::Custom(format!(
                    "Storage: unknown method '{}'",
                    method
                )));
            }
        };

        worker
            .sender
            .send(cmd)
            .map_err(|_| IfaError::Runtime("Storage worker channel closed".into()))?;

        Ok(IfaValue::NativeFuture(cell))
    }

    #[cfg(not(all(feature = "tokio", feature = "persistence")))]
    {
        let _ = (worker, method, args);
        Err(IfaError::Runtime(
            "Storage requires the 'tokio' and 'persistence' features".into(),
        ))
    }
}

#[derive(Clone, Default)]
#[cfg(not(all(feature = "tokio", feature = "persistence")))]
pub struct StorageWorker;

#[cfg(not(all(feature = "tokio", feature = "persistence")))]
impl StorageWorker {
    pub fn new() -> Self {
        Self
    }
}
