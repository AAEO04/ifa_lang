//! # Ọ̀sá Domain (0111)
//!
//! The Runner - Concurrency and Async
//!
//! Tokio-based async tasks with channels and synchronization.

use crate::impl_odu_domain;
// Unused imports removed
#[cfg(feature = "full")]
use tokio::sync::{Mutex, RwLock, mpsc, oneshot};
#[cfg(feature = "full")]
use tokio::task::JoinHandle;

/// Ọ̀sá - The Runner (Concurrency)
pub struct Osa;

impl_odu_domain!(Osa, "Ọ̀sá", "0111", "The Runner - Concurrency");

#[cfg(feature = "full")]
impl Osa {
    /// Spawn async task (sá)
    pub fn sa<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        tokio::spawn(future)
    }

    /// Sleep async (sùn)
    pub async fn sun(&self, ms: u64) {
        tokio::time::sleep(Duration::from_millis(ms)).await;
    }

    /// Create channel (ojú ọ̀nà)
    pub fn oju_ona<T>(&self, buffer: usize) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
        mpsc::channel(buffer)
    }

    /// Create oneshot channel
    pub fn oju_ona_kan<T>(&self) -> (oneshot::Sender<T>, oneshot::Receiver<T>) {
        oneshot::channel()
    }

    /// Create mutex (tìtìpẹ̀)
    pub fn titipe<T>(&self, value: T) -> Arc<Mutex<T>> {
        Arc::new(Mutex::new(value))
    }

    /// Create rwlock
    pub fn kaka<T>(&self, value: T) -> Arc<RwLock<T>> {
        Arc::new(RwLock::new(value))
    }

    /// Run future with timeout
    pub async fn pẹlu_akoko<F, T>(&self, future: F, timeout_ms: u64) -> Option<T>
    where
        F: std::future::Future<Output = T>,
    {
        tokio::time::timeout(Duration::from_millis(timeout_ms), future)
            .await
            .ok()
    }

    /// Yield control to scheduler
    pub async fn jeki(&self) {
        tokio::task::yield_now().await;
    }
}

#[cfg(not(feature = "full"))]
impl Osa {
    /// Placeholder for minimal builds
    pub fn placeholder(&self) {
        // Async not available in minimal mode
    }
}

#[cfg(test)]
#[cfg(feature = "full")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spawn() {
        let osa = Osa;
        let handle = osa.sa(async { 42 });
        assert_eq!(handle.await.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_channel() {
        let osa = Osa;
        let (tx, mut rx) = osa.oju_ona::<i32>(10);

        tx.send(42).await.unwrap();
        assert_eq!(rx.recv().await, Some(42));
    }
}
