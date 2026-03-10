use ifa_core::error::{IfaError, IfaResult};
use std::future::Future;

/// Abstraction for the System Runtime (Async Executor)
///
/// Hides the complexity of `tokio` vs `no-tokio` (WASM/Embedded).
pub struct SysRuntime {
    #[cfg(feature = "tokio")]
    inner: tokio::runtime::Runtime,
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
            Ok(Self { inner: rt })
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
}
