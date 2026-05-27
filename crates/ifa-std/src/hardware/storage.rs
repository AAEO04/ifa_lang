//! Storage Domain (Domain 20) wrapper
//! Bridges the VM dispatch to the async OduStore persistence layer.

use ifa_types::value_union::{FutureState, IfaValue};
use ifa_types::{IfaError, IfaResult};

#[cfg(all(feature = "tokio", feature = "persistence"))]
pub enum StorageCmd {
    Open {
        path: String,
        cell: ifa_types::value_union::FutureCell,
    },
    Set {
        id: u64,
        key: String,
        val: IfaValue,
        cell: ifa_types::value_union::FutureCell,
    },
    Get {
        id: u64,
        key: String,
        cell: ifa_types::value_union::FutureCell,
    },
    Delete {
        id: u64,
        key: String,
        cell: ifa_types::value_union::FutureCell,
    },
    Compact {
        id: u64,
        cell: ifa_types::value_union::FutureCell,
    },
}

#[derive(Clone)]
#[cfg(all(feature = "tokio", feature = "persistence"))]
pub struct StorageWorker {
    pub sender: std::sync::mpsc::SyncSender<StorageCmd>,
}

#[cfg(all(feature = "tokio", feature = "persistence"))]
impl StorageWorker {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::sync_channel::<StorageCmd>(64);

        std::thread::Builder::new()
            .name("ifa-storage".into())
            .spawn(move || {
                use ifa_infra::storage::OduStore;

                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("ifa-storage runtime");

                let mut stores: std::collections::HashMap<u64, OduStore> =
                    std::collections::HashMap::new();
                let mut next_id: u64 = 1;

                while let Ok(cmd) = rx.recv() {
                    match cmd {
                        StorageCmd::Open { path, cell } => {
                            let val = match rt.block_on(OduStore::open(&path)) {
                                Ok(store) => {
                                    let id = next_id;
                                    next_id += 1;
                                    stores.insert(id, store);
                                    IfaValue::int(id as i64)
                                }
                                Err(e) => IfaValue::str(format!("StorageError: {e}")),
                            };
                            *cell.lock().unwrap() = FutureState::Ready(val);
                        }
                        StorageCmd::Get { id, key, cell } => {
                            let val = match stores.get(&id) {
                                Some(store) => {
                                    match rt.block_on(async { store.get::<IfaValue>(&key).await }) {
                                        Ok(v) => v,
                                        Err(ifa_infra::storage::StorageError::KeyNotFound) => {
                                            IfaValue::null()
                                        }
                                        Err(e) => IfaValue::str(format!("StorageError: {e}")),
                                    }
                                }
                                None => IfaValue::str("StorageError: Invalid store handle"),
                            };
                            *cell.lock().unwrap() = FutureState::Ready(val);
                        }
                        StorageCmd::Set { id, key, val, cell } => {
                            let res_val = match stores.get_mut(&id) {
                                Some(store) => {
                                    match rt.block_on(async { store.set(&key, &val).await }) {
                                        Ok(_) => IfaValue::null(),
                                        Err(e) => IfaValue::str(format!("StorageError: {e}")),
                                    }
                                }
                                None => IfaValue::str("StorageError: Invalid store handle"),
                            };
                            *cell.lock().unwrap() = FutureState::Ready(res_val);
                        }
                        StorageCmd::Delete { id, key, cell } => {
                            let val = match stores.get_mut(&id) {
                                Some(store) => {
                                    match rt.block_on(async { store.delete(&key).await }) {
                                        Ok(_) => IfaValue::null(),
                                        Err(e) => IfaValue::str(format!("StorageError: {e}")),
                                    }
                                }
                                None => IfaValue::str("StorageError: Invalid store handle"),
                            };
                            *cell.lock().unwrap() = FutureState::Ready(val);
                        }
                        StorageCmd::Compact { id, cell } => {
                            let val = match stores.get_mut(&id) {
                                Some(store) => match rt.block_on(async { store.compact().await }) {
                                    Ok(_) => IfaValue::null(),
                                    Err(e) => IfaValue::str(format!("StorageError: {e}")),
                                },
                                None => IfaValue::str("StorageError: Invalid store handle"),
                            };
                            *cell.lock().unwrap() = FutureState::Ready(val);
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
        use std::sync::{Arc, Mutex};

        let cell = Arc::new(Mutex::new(FutureState::Pending));

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
                StorageCmd::Set {
                    id,
                    key,
                    val,
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

        Ok(IfaValue::Future(cell))
    }

    #[cfg(not(all(feature = "tokio", feature = "persistence")))]
    {
        let _ = (method, args);
        Err(IfaError::Runtime(
            "Storage requires the 'tokio' and 'persistence' features".into(),
        ))
    }
}

#[derive(Clone)]
#[cfg(not(all(feature = "tokio", feature = "persistence")))]
pub struct StorageWorker;

#[cfg(not(all(feature = "tokio", feature = "persistence")))]
impl StorageWorker {
    pub fn new() -> Self {
        Self
    }
}
