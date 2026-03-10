//! # Sandbox Shim
//!
//! Re-exports `ifa_sandbox` types if available, or provides stubs if not.
//! This allows compilation on targets without `ifa-sandbox` (e.g. WASM).

#[cfg(feature = "ifa-sandbox")]
pub use ifa_sandbox::{CapabilitySet, Ofun};

#[cfg(not(feature = "ifa-sandbox"))]
pub mod stub {
    use super::*;
    use std::path::PathBuf;

    #[derive(Default, Clone)]
    pub struct CapabilitySet;

    impl CapabilitySet {
        pub fn check(&self, _capability: &Ofun) -> bool {
            true // Allow everything on WASM/NoSandbox
        }

        pub fn grant(&mut self, _capability: Ofun) {}
    }

    #[derive(Clone)]
    pub enum Ofun {
        Time,
        Network { domains: Vec<String> },
        ReadFiles { root: PathBuf },
        WriteFiles { root: PathBuf },
        Stdio,
        Random,
        Environment { keys: Vec<String> },
    }
}

#[cfg(not(feature = "ifa-sandbox"))]
pub use stub::*;
