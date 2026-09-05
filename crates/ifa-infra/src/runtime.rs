use ifa_types::error::{IfaError, IfaResult};
use std::future::Future;

#[cfg(feature = "persistence")]
use crate::storage::OduStore;
#[cfg(feature = "persistence")]
use std::sync::Arc;
#[cfg(all(feature = "persistence", feature = "tokio"))]
use tokio::sync::Mutex as AsyncMutex;

/// Abstraction for the System Runtime (Async Executor)
///
/// Hides the complexity of `tokio` vs `no-tokio` (WASM/Embedded).
pub struct SysRuntime {
    #[cfg(feature = "tokio")]
    inner: tokio::runtime::Runtime,
    
    #[cfg(all(feature = "persistence", feature = "tokio"))]
    iranti_store: Option<Arc<AsyncMutex<OduStore>>>,
}

impl SysRuntime {
    /// Create a new runtime
    pub fn new() -> IfaResult<Self> {
        #[cfg(feature = "tokio")]
        {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| IfaError::Runtime(format!("Failed to create runtime: {}", e)))?;
            Ok(Self { 
                inner: rt,
                #[cfg(feature = "persistence")]
                iranti_store: None,
            })
        }
        #[cfg(not(feature = "tokio"))]
        {
            // On non-tokio targets, we just pretend to exist.
            // Actual calls to block_on will fail gracefully.
            Ok(Self {})
        }
    }

    /// Block on a future and return the result
    ///
    /// On 'tokio' feature: Runs the future to completion.
    /// On no 'tokio': Returns Error (cannot execute async code synchronously without runtime).
    pub fn block_on<F: Future>(&self, future: F) -> IfaResult<F::Output> {
        #[cfg(feature = "tokio")]
        {
            Ok(self.inner.block_on(future))
        }
        #[cfg(not(feature = "tokio"))]
        {
            // Suppress unused variable warning for future
            let _ = future;
            Err(IfaError::Runtime(
                "Async runtime not available (requires 'tokio' feature)".into(),
            ))
        }
    }

    #[cfg(all(feature = "persistence", feature = "tokio"))]
    pub fn init_iranti(&mut self, path: std::path::PathBuf) -> IfaResult<()> {
        let store_res = self.block_on(OduStore::open(path))?;
        let store = store_res.map_err(|e| {
            IfaError::Runtime(format!("Failed to initialize Iranti store: {:?}", e))
        })?;
        self.iranti_store = Some(Arc::new(AsyncMutex::new(store)));
        Ok(())
    }

    #[cfg(all(feature = "persistence", feature = "tokio"))]
    pub fn memoize_get<V: serde::de::DeserializeOwned + Send + 'static>(&self, key: &str) -> IfaResult<Option<V>> {
        if let Some(store_arc) = &self.iranti_store {
            let store_arc_clone = Arc::clone(store_arc);
            let key_str = key.to_string();
            self.block_on(async move {
                let store = store_arc_clone.lock().await;
                store.get::<V>(&key_str).await
            })?
            .map(Some)
            .or_else(|e| {
                if matches!(e, crate::storage::StorageError::KeyNotFound) {
                    Ok(None)
                } else {
                    Err(IfaError::Runtime(format!("Iranti error: {:?}", e)))
                }
            })
        } else {
            Ok(None)
        }
    }

    #[cfg(all(feature = "persistence", feature = "tokio"))]
    pub fn memoize_set<V: serde::Serialize + Send + 'static>(&self, key: String, value: V) -> IfaResult<()> {
        if let Some(store_arc) = &self.iranti_store {
            let store_arc_clone = Arc::clone(store_arc);
            self.block_on(async move {
                let mut store = store_arc_clone.lock().await;
                store.set(&key, &value).await
            })?
            .map_err(|e| IfaError::Runtime(format!("Iranti set error: {:?}", e)))?;
        }
        Ok(())
    }
}
