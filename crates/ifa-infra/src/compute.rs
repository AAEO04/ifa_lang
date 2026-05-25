//! Compute abstraction for zero-cost backend selection.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceInfo {
    pub name: &'static str,
    pub unified_memory: bool,
}

pub trait ComputeBackend: Send + Sync {
    fn par_map<T, U, F>(&self, data: &[T], f: F) -> Vec<U>
    where
        T: Send + Sync,
        U: Send,
        F: Fn(&T) -> U + Send + Sync;

    fn matmul(&self, a: &[f32], b: &[f32], m: usize, n: usize, k: usize) -> Vec<f32>;

    fn reduce_sum(&self, data: &[f32]) -> f32;

    fn device_info(&self) -> DeviceInfo;
}
