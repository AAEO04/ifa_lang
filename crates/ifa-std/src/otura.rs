//! # Òtúrá Domain (1011)
//!
//! The Messenger - Networking
//!
//! Tokio async networking with rustls for TLS and SSRF protection.

use crate::impl_odu_domain;
// Unused imports removed
use std::net::IpAddr;
#[cfg(feature = "full")]
use tokio::net::{TcpListener, TcpStream};

/// Blocked hosts for SSRF protection
const BLOCKED_HOSTS: &[&str] = &[
    "localhost",
    "127.0.0.1",
    "0.0.0.0",
    "::1",
    "metadata.google.internal",
    "169.254.169.254", // AWS/Cloud metadata
];

use ifa_sandbox::{CapabilitySet, Ofun};

/// Òtúrá - The Messenger (Networking)
#[derive(Default)]
pub struct Otura {
    capabilities: CapabilitySet,
}

impl_odu_domain!(Otura, "Òtúrá", "1011", "The Messenger - Networking");

impl Otura {
    pub fn new(capabilities: CapabilitySet) -> Self {
        Otura { capabilities }
    }

    /// Check if host is allowed (SSRF protection + Capability)
    pub fn ṣàyẹ̀wò(&self, host: &str) -> bool {
        // First check capability
        if !self.capabilities.check(&Ofun::Network {
            domains: vec![host.to_string()],
        }) {
            return false;
        }

        let host_lower = host.to_lowercase();

        // Block known dangerous hosts
        for blocked in BLOCKED_HOSTS {
            if host_lower == *blocked || host_lower.ends_with(blocked) {
                return false;
            }
        }

        // Block private IP ranges
        if let Ok(ip) = host.parse::<IpAddr>() {
            match ip {
                IpAddr::V4(v4) => {
                    if v4.is_loopback() || v4.is_private() || v4.is_link_local() {
                        return false;
                    }
                }
                IpAddr::V6(v6) => {
                    if v6.is_loopback() {
                        return false;
                    }
                }
            }
        }

        true
    }
}

#[cfg(feature = "full")]
impl Otura {
    /// HTTP GET request (gbà)
    pub async fn gba(&self, url: &str) -> IfaResult<String> {
        // Extract host for SSRF check
        if let Some(host) = url.split("://").nth(1).and_then(|s| s.split('/').next()) {
            let host = host.split(':').next().unwrap_or(host);
            if !self.ṣàyẹ̀wò(host) {
                return Err(IfaError::SsrfBlocked(host.to_string()));
            }
        } else {
            return Err(IfaError::Runtime("Invalid URL".into()));
        }

        reqwest::get(url)
            .await
            .map_err(|e| IfaError::ConnectionFailed(e.to_string()))?
            .text()
            .await
            .map_err(|e| IfaError::Custom(format!("Response error: {}", e)))
    }

    /// HTTP POST request (rán)
    pub async fn ran(&self, url: &str, body: &str) -> IfaResult<String> {
        // SSFR Check
        if let Some(host) = url.split("://").nth(1).and_then(|s| s.split('/').next()) {
            let host = host.split(':').next().unwrap_or(host);
            if !self.ṣàyẹ̀wò(host) {
                return Err(IfaError::SsrfBlocked(host.to_string()));
            }
        }

        let client = reqwest::Client::new();
        client
            .post(url)
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| IfaError::ConnectionFailed(e.to_string()))?
            .text()
            .await
            .map_err(|e| IfaError::Custom(format!("Response error: {}", e)))
    }

    /// TCP listen (dè)
    pub async fn de(&self, addr: &str) -> IfaResult<TcpListener> {
        if !self.capabilities.check(&Ofun::Network {
            domains: vec!["*".to_string()],
        }) {
            return Err(IfaError::PermissionDenied("Network bind denied".into()));
        }

        TcpListener::bind(addr)
            .await
            .map_err(|e| IfaError::Custom(format!("Bind error: {}", e)))
    }

    /// TCP connect (sọ̀rọ̀)
    pub async fn soro(&self, addr: &str) -> IfaResult<TcpStream> {
        // Extract host
        let host = addr.split(':').next().unwrap_or(addr);
        if !self.ṣàyẹ̀wò(host) {
            return Err(IfaError::SsrfBlocked(host.to_string()));
        }

        TcpStream::connect(addr)
            .await
            .map_err(|e| IfaError::ConnectionFailed(e.to_string()))
    }
}

#[cfg(not(feature = "full"))]
impl Otura {
    /// Placeholder for minimal builds
    pub fn placeholder(&self) {
        // Networking not available in minimal mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssrf_protection() {
        // Create Otura with network capabilities granted for test domains
        let mut caps = CapabilitySet::default();
        caps.grant(Ofun::Network {
            domains: vec!["example.com".to_string(), "8.8.8.8".to_string()],
        });
        let otura = Otura::new(caps);

        // Should block (localhost blocked even with capability due to SSRF protection)
        assert!(!otura.ṣàyẹ̀wò("localhost"));
        assert!(!otura.ṣàyẹ̀wò("127.0.0.1"));
        assert!(!otura.ṣàyẹ̀wò("169.254.169.254"));

        // Should allow (granted in capability)
        assert!(otura.ṣàyẹ̀wò("example.com"));
        assert!(otura.ṣàyẹ̀wò("8.8.8.8"));
    }
}
