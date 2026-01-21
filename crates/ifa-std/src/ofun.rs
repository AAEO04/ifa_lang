//! # Òfún Domain (0101)
//!
//! The Reflector - Permissions and Reflection
//!
//! Capability-based permissions and introspection macros.

use crate::impl_odu_domain;
use ifa_core::IfaValue;

/// Capability flags for sandboxing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities {
    /// Can read files
    pub read_files: bool,
    /// Can write files
    pub write_files: bool,
    /// Can access network
    pub network: bool,
    /// Can spawn processes
    pub spawn: bool,
    /// Can access environment
    pub env: bool,
    /// Can use polyglot bridges
    pub bridges: Vec<String>,
}

impl Default for Capabilities {
    fn default() -> Self {
        Capabilities {
            read_files: true,
            write_files: true,
            network: true,
            spawn: true,
            env: true,
            bridges: vec![],
        }
    }
}

impl Capabilities {
    /// Full capabilities (no sandbox)
    pub fn full() -> Self {
        Capabilities {
            read_files: true,
            write_files: true,
            network: true,
            spawn: true,
            env: true,
            bridges: vec!["*".to_string()],
        }
    }

    /// No capabilities (fully sandboxed)
    pub fn none() -> Self {
        Capabilities {
            read_files: false,
            write_files: false,
            network: false,
            spawn: false,
            env: false,
            bridges: vec![],
        }
    }

    /// Read-only mode
    pub fn read_only() -> Self {
        Capabilities {
            read_files: true,
            write_files: false,
            network: false,
            spawn: false,
            env: true,
            bridges: vec![],
        }
    }
}

/// Òfún - The Reflector (Permissions/Reflection)
#[derive(Default)]
pub struct Ofun {
    capabilities: Capabilities,
}

impl_odu_domain!(Ofun, "Òfún", "0101", "The Reflector - Permissions");

impl Ofun {
    /// Create with specific capabilities
    pub fn with_capabilities(caps: Capabilities) -> Self {
        Ofun { capabilities: caps }
    }

    /// Check if capability is allowed
    pub fn le(&self, cap: &str) -> bool {
        match cap {
            "read" | "ka" => self.capabilities.read_files,
            "write" | "ko" => self.capabilities.write_files,
            "network" | "nẹtiwọki" => self.capabilities.network,
            "spawn" | "bere" => self.capabilities.spawn,
            "env" | "ayika" => self.capabilities.env,
            s if s.starts_with("bridge:") => {
                let lang = &s[7..];
                self.capabilities.bridges.contains(&lang.to_string())
                    || self.capabilities.bridges.contains(&"*".to_string())
            }
            _ => false,
        }
    }

    /// Drop capability (can only remove, never add)
    pub fn ju(&mut self, cap: &str) {
        match cap {
            "read" | "ka" => self.capabilities.read_files = false,
            "write" | "ko" => self.capabilities.write_files = false,
            "network" | "nẹtiwọki" => self.capabilities.network = false,
            "spawn" | "bere" => self.capabilities.spawn = false,
            "env" | "ayika" => self.capabilities.env = false,
            s if s.starts_with("bridge:") => {
                let lang = &s[7..];
                self.capabilities.bridges.retain(|x| x != lang);
            }
            _ => {}
        }
    }

    /// Get current capabilities
    pub fn awon_agbara(&self) -> &Capabilities {
        &self.capabilities
    }

    // =========================================================================
    // REFLECTION (Type introspection)
    // =========================================================================

    /// Get type name of value (irú)
    pub fn iru(&self, value: &IfaValue) -> &'static str {
        value.type_name()
    }

    /// Check if value is of type
    pub fn je(&self, value: &IfaValue, type_name: &str) -> bool {
        value.type_name().eq_ignore_ascii_case(type_name)
    }

    /// Get value as debug string
    pub fn afiwe(&self, value: &IfaValue) -> String {
        format!("{:?}", value)
    }
}

/// Macro for requiring capability
#[macro_export]
macro_rules! require_cap {
    ($ofun:expr, $cap:expr) => {
        if !$ofun.le($cap) {
            return Err($crate::ifa_core::error::IfaError::PermissionDenied(
                format!("Capability '{}' not allowed", $cap),
            ));
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capabilities() {
        let ofun = Ofun::default();
        assert!(ofun.le("read"));
        assert!(ofun.le("write"));
    }

    #[test]
    fn test_drop_capability() {
        let mut ofun = Ofun::default();
        assert!(ofun.le("write"));
        ofun.ju("write");
        assert!(!ofun.le("write"));
    }

    #[test]
    fn test_sandboxed() {
        let ofun = Ofun::with_capabilities(Capabilities::none());
        assert!(!ofun.le("read"));
        assert!(!ofun.le("network"));
    }

    #[test]
    fn test_reflection() {
        let ofun = Ofun::default();
        let value = IfaValue::Int(42);
        assert_eq!(ofun.iru(&value), "Int");
        assert!(ofun.je(&value, "int"));
    }
}
