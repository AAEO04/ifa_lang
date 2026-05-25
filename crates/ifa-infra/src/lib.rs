// infra/mod.rs content
//! # Infrastructure Layer
//!
//! The providers of performance.

pub mod compute;

#[cfg(feature = "parallel")]
pub mod cpu;

#[cfg(feature = "gpu")]
pub mod gpu;

#[cfg(feature = "gpu")]
pub mod shaders;

#[cfg(feature = "persistence")]
pub mod storage;

pub mod kernel;
pub mod runtime;

pub use compute::{ComputeBackend, DeviceInfo};
