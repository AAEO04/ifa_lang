//! # Ìrẹtẹ̀ Domain (1101)
//!
//! The Presser - Cryptography and Compression
//!
//! Uses ring for audited crypto primitives and zstd for compression.

use crate::impl_odu_domain;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use ifa_core::error::{IfaError, IfaResult};
use ring::rand::SecureRandom;
use ring::{digest, hmac, rand as ring_rand};

/// Ìrẹtẹ̀ - The Presser (Crypto/Compression)
pub struct Irete;

impl_odu_domain!(Irete, "Ìrẹtẹ̀", "1101", "The Presser - Crypto/Compression");

impl Irete {
    // =========================================================================
    // HASHING
    // =========================================================================

    /// SHA-256 hash (ẹ̀rí)
    pub fn sha256(&self, data: &[u8]) -> Vec<u8> {
        let result = digest::digest(&digest::SHA256, data);
        result.as_ref().to_vec()
    }

    /// SHA-256 as hex string
    pub fn sha256_hex(&self, data: &[u8]) -> String {
        self.sha256(data)
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }

    /// SHA-512 hash
    pub fn sha512(&self, data: &[u8]) -> Vec<u8> {
        let result = digest::digest(&digest::SHA512, data);
        result.as_ref().to_vec()
    }

    /// HMAC-SHA256
    pub fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Vec<u8> {
        let key = hmac::Key::new(hmac::HMAC_SHA256, key);
        let tag = hmac::sign(&key, data);
        tag.as_ref().to_vec()
    }

    /// Verify HMAC
    pub fn hmac_verify(&self, key: &[u8], data: &[u8], signature: &[u8]) -> bool {
        let key = hmac::Key::new(hmac::HMAC_SHA256, key);
        hmac::verify(&key, data, signature).is_ok()
    }

    // =========================================================================
    // RANDOM (Cryptographic)
    // =========================================================================

    /// Generate secure random bytes
    pub fn random_bytes(&self, count: usize) -> IfaResult<Vec<u8>> {
        let rng = ring_rand::SystemRandom::new();
        let mut bytes = vec![0u8; count];
        rng.fill(&mut bytes)
            .map_err(|_| IfaError::Custom("Random generation failed".to_string()))?;
        Ok(bytes)
    }

    // =========================================================================
    // ENCODING
    // =========================================================================

    /// Base64 encode
    pub fn base64_encode(&self, data: &[u8]) -> String {
        BASE64.encode(data)
    }

    /// Base64 decode
    pub fn base64_decode(&self, data: &str) -> IfaResult<Vec<u8>> {
        BASE64
            .decode(data)
            .map_err(|e| IfaError::Custom(format!("Base64 decode error: {}", e)))
    }

    /// Hex encode
    pub fn hex_encode(&self, data: &[u8]) -> String {
        data.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Hex decode
    pub fn hex_decode(&self, hex: &str) -> IfaResult<Vec<u8>> {
        if hex.len() % 2 != 0 {
            return Err(IfaError::Custom("Invalid hex length".to_string()));
        }

        (0..hex.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&hex[i..i + 2], 16)
                    .map_err(|e| IfaError::Custom(format!("Invalid hex: {}", e)))
            })
            .collect()
    }

    // =========================================================================
    // COMPRESSION (zstd)
    // =========================================================================

    /// Compress data with zstd (fún pọ̀)
    pub fn funpo(&self, data: &[u8], level: i32) -> IfaResult<Vec<u8>> {
        zstd::encode_all(data, level)
            .map_err(|e| IfaError::Custom(format!("Compression error: {}", e)))
    }

    /// Decompress zstd data (tú)
    pub fn tu(&self, data: &[u8]) -> IfaResult<Vec<u8>> {
        zstd::decode_all(data).map_err(|e| IfaError::Custom(format!("Decompression error: {}", e)))
    }

    /// Get compression ratio
    pub fn iwon_funpo(&self, original: usize, compressed: usize) -> f64 {
        if original == 0 {
            return 0.0;
        }
        1.0 - (compressed as f64 / original as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let irete = Irete;
        let hash = irete.sha256_hex(b"hello");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hmac() {
        let irete = Irete;
        let key = b"secret";
        let data = b"message";

        let sig = irete.hmac_sha256(key, data);
        assert!(irete.hmac_verify(key, data, &sig));
        assert!(!irete.hmac_verify(key, b"wrong", &sig));
    }

    #[test]
    fn test_base64() {
        let irete = Irete;
        let original = b"Hello, Ifa!";
        let encoded = irete.base64_encode(original);
        let decoded = irete.base64_decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_compression() {
        let irete = Irete;
        let data = b"Hello Hello Hello Hello Hello".repeat(100);
        let compressed = irete.funpo(&data, 3).unwrap();
        let decompressed = irete.tu(&compressed).unwrap();

        assert!(compressed.len() < data.len());
        assert_eq!(decompressed, data);
    }
}
