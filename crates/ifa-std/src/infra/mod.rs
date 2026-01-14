
// infra/mod.rs content
//! # Infrastructure Layer
//! 
//! The providers of performance.

#[cfg(feature = "parallel")]
pub mod cpu;

#[cfg(feature = "gpu")]
pub mod gpu;

#[cfg(feature = "persistence")]
pub mod storage;

pub mod kernel;
