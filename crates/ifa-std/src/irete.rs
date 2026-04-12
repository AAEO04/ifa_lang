//! # Ìrẹtẹ̀ Domain (1101)
//!
//! The Presser - Cryptography and Compression
//!
//! Uses ring for audited crypto primitives and zstd for compression.

use crate::impl_odu_domain;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ifa_core::error::{IfaError, IfaResult};
use ring::rand::SecureRandom;
use ring::{aead, digest, hmac, rand as ring_rand, signature};

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
    // SYMMETRIC ENCRYPTION (ChaCha20-Poly1305)
    // =========================================================================

    /// Encrypt data using ChaCha20-Poly1305 (di_pa)
    pub fn chacha20_encrypt(&self, key: &[u8], nonce: &[u8], data: &[u8]) -> IfaResult<Vec<u8>> {
        let unbound_key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key)
            .map_err(|_| IfaError::Custom("Invalid ChaCha20 key length".into()))?;
        let less_safe_key = aead::LessSafeKey::new(unbound_key);
        let nonce_obj = aead::Nonce::try_assume_unique_for_key(nonce).map_err(|_| {
            IfaError::Custom("Invalid ChaCha20 nonce length (must be 12 bytes)".into())
        })?;

        let mut in_out = data.to_vec();
        less_safe_key
            .seal_in_place_append_tag(nonce_obj, aead::Aad::empty(), &mut in_out)
            .map_err(|e| IfaError::Custom(format!("Encryption failed: {}", e)))?;

        Ok(in_out)
    }

    /// Decrypt data using ChaCha20-Poly1305 (tu_pa)
    pub fn chacha20_decrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        encrypted_data: &[u8],
    ) -> IfaResult<Vec<u8>> {
        let unbound_key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key)
            .map_err(|_| IfaError::Custom("Invalid ChaCha20 key length".into()))?;
        let less_safe_key = aead::LessSafeKey::new(unbound_key);
        let nonce_obj = aead::Nonce::try_assume_unique_for_key(nonce).map_err(|_| {
            IfaError::Custom("Invalid ChaCha20 nonce length (must be 12 bytes)".into())
        })?;

        let mut in_out = encrypted_data.to_vec();
        let decrypted_data = less_safe_key
            .open_in_place(nonce_obj, aead::Aad::empty(), &mut in_out)
            .map_err(|e| IfaError::Custom(format!("Decryption failed (auth error): {}", e)))?;

        Ok(decrypted_data.to_vec())
    }

    // =========================================================================
    // ASYMMETRIC CRYPTOGRAPHY (Ed25519)
    // =========================================================================

    /// Generate Ed25519 Keypair (Private PKCS#8 bytes, Public bytes)
    pub fn ed25519_generate(&self) -> IfaResult<(Vec<u8>, Vec<u8>)> {
        let rng = ring_rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|_| IfaError::Custom("Key generation failed".into()))?;
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .map_err(|_| IfaError::Custom("Failed to format keypair".into()))?;

        Ok((
            pkcs8_bytes.as_ref().to_vec(),
            key_pair.public_key().as_ref().to_vec(),
        ))
    }

    /// Sign data with Ed25519 (fi_o)
    pub fn ed25519_sign(&self, private_pkcs8: &[u8], message: &[u8]) -> IfaResult<Vec<u8>> {
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(private_pkcs8)
            .map_err(|_| IfaError::Custom("Invalid private key format".into()))?;
        let sig = key_pair.sign(message);
        Ok(sig.as_ref().to_vec())
    }

    /// Verify Ed25519 Signature (yewo_fo)
    pub fn ed25519_verify(&self, public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
        let unparsed_pub = signature::UnparsedPublicKey::new(&signature::ED25519, public_key);
        unparsed_pub.verify(message, signature).is_ok()
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
    #[cfg(feature = "zstd")]
    pub fn funpo(&self, data: &[u8], level: i32) -> IfaResult<Vec<u8>> {
        zstd::encode_all(data, level)
            .map_err(|e| IfaError::Custom(format!("Compression error: {}", e)))
    }

    #[cfg(not(feature = "zstd"))]
    pub fn funpo(&self, _data: &[u8], _level: i32) -> IfaResult<Vec<u8>> {
        Err(IfaError::Runtime(
            "Compression disabled (zstd feature missing)".into(),
        ))
    }

    /// Decompress zstd data (tú)
    #[cfg(feature = "zstd")]
    pub fn tu(&self, data: &[u8]) -> IfaResult<Vec<u8>> {
        zstd::decode_all(data).map_err(|e| IfaError::Custom(format!("Decompression error: {}", e)))
    }

    #[cfg(not(feature = "zstd"))]
    pub fn tu(&self, _data: &[u8]) -> IfaResult<Vec<u8>> {
        Err(IfaError::Runtime(
            "Decompression disabled (zstd feature missing)".into(),
        ))
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

    #[test]
    fn test_chacha20_poly1305() {
        let irete = Irete;
        let key = b"01234567890123456789012345678901"; // 32 bytes
        let nonce = b"012345678901"; // 12 bytes
        let data = b"Secret Ifa message";

        let encrypted = irete.chacha20_encrypt(key, nonce, data).unwrap();
        // Encrypted length is data + 16 bytes tag
        assert_eq!(encrypted.len(), data.len() + 16);

        let decrypted = irete.chacha20_decrypt(key, nonce, &encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_ed25519() {
        let irete = Irete;
        let (priv_key, pub_key) = irete.ed25519_generate().unwrap();

        let message = b"Sign this";
        let signature = irete.ed25519_sign(&priv_key, message).unwrap();

        assert!(irete.ed25519_verify(&pub_key, message, &signature));
        assert!(!irete.ed25519_verify(&pub_key, b"Wrong data", &signature));
    }
}
