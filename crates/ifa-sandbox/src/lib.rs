//! # Ìgbálẹ̀ Sandbox (OmniBox)
//!
//! Secure runtime environment for Ifá-Lang, providing both native and WASM isolation.
//!
//! ## Ọ̀fún Capability System
//!
//! The sandbox enforces a deny-by-default security model using the Ọ̀fún system.
//! No code can access files, network, or environment without explicit capability grants.
//!
//! ## Modules
//!
//! - `capability` - Definition of the Ọ̀fún capability types
//! - `config` - Sandbox configuration and security profiles
//! - `runtime` - Native runtime capability enforcement
//! - `omnibox` - WASM-based isolated runtime
//! - `sandbox` - Process isolation wrapper
//! - `monitor` - Resource monitoring (memory, CPU, files, network)

pub mod capability;
pub mod config;
pub mod monitor;
pub mod omnibox;
pub mod runtime;
pub mod sandbox;

pub use capability::{CapabilitySet, Ofun};
pub use config::{SandboxConfig, SecurityProfile};
pub use monitor::ResourceMonitor;
pub use omnibox::OmniBox;
pub use sandbox::Sandbox;
