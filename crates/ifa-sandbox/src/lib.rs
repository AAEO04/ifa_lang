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

pub mod capability;
pub mod config;
pub mod runtime;
pub mod omnibox;

pub use capability::{Ofun, CapabilitySet};
pub use config::{SandboxConfig, SecurityProfile};
pub use omnibox::OmniBox;
