//! # Òfún Domain (0101)
//!
//! The Reflector - Permissions and Reflection
//!
//! Capability-based permissions and introspection macros.

use crate::impl_odu_domain;
use crate::sandbox_shim::CapabilitySet;
use ifa_vm::IfaValue;

/// Òfún - The Reflector (Permissions/Reflection)
pub struct Ofun {
    capabilities: CapabilitySet,
}

impl_odu_domain!(Ofun, "Òfún", "0101", "The Reflector - Permissions");

impl Default for Ofun {
    fn default() -> Self {
        let mut caps = CapabilitySet::new();
        // Default grants (full access for un-sandboxed mode)
        caps.grant(crate::sandbox_shim::Ofun::ReadFiles {
            root: std::path::PathBuf::from("/"),
        });
        caps.grant(crate::sandbox_shim::Ofun::WriteFiles {
            root: std::path::PathBuf::from("/"),
        });
        caps.grant(crate::sandbox_shim::Ofun::Network {
            domains: vec!["*".to_string()],
        });
        caps.grant(crate::sandbox_shim::Ofun::Execute {
            programs: vec!["*".to_string()],
        });
        caps.grant(crate::sandbox_shim::Ofun::Environment {
            keys: vec!["*".to_string()],
        });
        caps.grant(crate::sandbox_shim::Ofun::Time);
        caps.grant(crate::sandbox_shim::Ofun::Random);
        caps.grant(crate::sandbox_shim::Ofun::Stdio);
        caps.grant(crate::sandbox_shim::Ofun::Crypto);
        Ofun { capabilities: caps }
    }
}

impl Ofun {
    /// Create with specific capabilities
    pub fn with_capabilities(caps: CapabilitySet) -> Self {
        Ofun { capabilities: caps }
    }

    /// Check if capability is allowed
    pub fn le(&self, cap: &str) -> bool {
        match cap {
            "read" | "ka" => self
                .capabilities
                .all()
                .iter()
                .any(|c| matches!(c, crate::sandbox_shim::Ofun::ReadFiles { .. })),
            "write" | "ko" => self
                .capabilities
                .all()
                .iter()
                .any(|c| matches!(c, crate::sandbox_shim::Ofun::WriteFiles { .. })),
            "network" | "nẹtiwọki" => self
                .capabilities
                .all()
                .iter()
                .any(|c| matches!(c, crate::sandbox_shim::Ofun::Network { .. })),
            "spawn" | "bere" | "execute" => self
                .capabilities
                .all()
                .iter()
                .any(|c| matches!(c, crate::sandbox_shim::Ofun::Execute { .. })),
            "env" | "ayika" => self
                .capabilities
                .all()
                .iter()
                .any(|c| matches!(c, crate::sandbox_shim::Ofun::Environment { .. })),
            "crypto" | "irete" => self
                .capabilities
                .all()
                .iter()
                .any(|c| matches!(c, crate::sandbox_shim::Ofun::Crypto)),
            s if s.starts_with("bridge:") => {
                let lang = &s[7..];
                self.capabilities.check(&crate::sandbox_shim::Ofun::Bridge {
                    language: lang.to_string(),
                })
            }
            _ => false,
        }
    }

    /// Drop capability (can only remove, never add)
    pub fn ju(&mut self, cap: &str) {
        match cap {
            "read" | "ka" => self
                .capabilities
                .remove_matching(|c| matches!(c, crate::sandbox_shim::Ofun::ReadFiles { .. })),
            "write" | "ko" => self
                .capabilities
                .remove_matching(|c| matches!(c, crate::sandbox_shim::Ofun::WriteFiles { .. })),
            "network" | "nẹtiwọki" => self
                .capabilities
                .remove_matching(|c| matches!(c, crate::sandbox_shim::Ofun::Network { .. })),
            "spawn" | "bere" => self
                .capabilities
                .remove_matching(|c| matches!(c, crate::sandbox_shim::Ofun::Execute { .. })),
            "env" | "ayika" => self
                .capabilities
                .remove_matching(|c| matches!(c, crate::sandbox_shim::Ofun::Environment { .. })),
            "crypto" | "irete" => self
                .capabilities
                .remove_matching(|c| matches!(c, crate::sandbox_shim::Ofun::Crypto)),
            s if s.starts_with("bridge:") => {
                let lang = &s[7..];
                self.capabilities.remove_matching(|c| {
                    if let crate::sandbox_shim::Ofun::Bridge { language } = c {
                        language == lang
                    } else {
                        false
                    }
                });
            }
            _ => {}
        }
    }

    /// Get current capabilities reference
    pub fn awon_agbara(&self) -> &CapabilitySet {
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
            return Err($crate::ifa_vm::error::IfaError::PermissionDenied(format!(
                "Capability '{}' not allowed",
                $cap
            )));
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
        let ofun = Ofun::with_capabilities(CapabilitySet::new());
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
