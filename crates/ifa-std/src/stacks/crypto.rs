//! # Crypto Stack
//!
//! **Security-focused** cryptographic extensions for Ifá-Lang.
//!
//! ## Implementation status
//!
//! | Component | Status |
//! |---|---|
//! | `hash::sha256`, `hash::sha512` | ✅ Production-grade via `ring` |
//! | `hash::hmac_sha256`, `hash::hmac_verify` | ✅ Production-grade via `ring` |
//! | `password::hash`, `password::verify` | ✅ Argon2id via `argon2` crate |
//! | `SecureRng` | ✅ OS entropy via `rand::OsRng` |
//! | `base64`, `hex` | ✅ Pure-Rust RFC-compliant implementations |
//! | `hash_password` (free fn) | ⚠️ Simplified PBKDF2-like; use `password::hash` for production |
//!
//! For FIPS 140-3 compliance or HSM integration, replace `ring` with a certified provider.

use std::collections::HashMap;

/// Error types for crypto operations
#[derive(Debug, Clone)]
pub enum CryptoError {
    InvalidKeyLength { expected: usize, actual: usize },
    InvalidInput(String),
    VerificationFailed,
    RngError,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidKeyLength { expected, actual } => write!(
                f,
                "Invalid key length: expected {} bytes, got {}",
                expected, actual
            ),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Self::VerificationFailed => write!(f, "Verification failed"),
            Self::RngError => write!(f, "Random number generation failed"),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Secure hash functions backed by the `ring` crate.
///
/// All functions in this module are production-quality.
/// `sha256` and `sha512` use FIPS 180-4 compliant implementations.
/// `hmac_sha256` uses FIPS 198-1.
pub mod hash {
    use ring::digest::{SHA256, SHA512};
    use ring::hmac;

    /// SHA-256 hash using ring crate
    ///
    /// Production-quality cryptographic hash function.
    pub fn sha256(data: &[u8]) -> [u8; 32] {
        let digest = ring::digest::digest(&SHA256, data);
        let mut result = [0u8; 32];
        result.copy_from_slice(digest.as_ref());
        result
    }

    /// SHA-256 returning hex string
    pub fn sha256_hex(data: &[u8]) -> String {
        let digest = sha256(data);
        let mut s = String::with_capacity(64);
        use std::fmt::Write;
        for b in digest {
            write!(s, "{:02x}", b).unwrap();
        }
        s
    }

    /// SHA-512 hash using ring crate
    pub fn sha512(data: &[u8]) -> [u8; 64] {
        let digest = ring::digest::digest(&SHA512, data);
        let mut result = [0u8; 64];
        result.copy_from_slice(digest.as_ref());
        result
    }

    /// HMAC-SHA256 using ring crate
    ///
    /// Production-quality message authentication code.
    pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
        let key = hmac::Key::new(hmac::HMAC_SHA256, key);
        let tag = hmac::sign(&key, data);
        let mut result = [0u8; 32];
        result.copy_from_slice(tag.as_ref());
        result
    }

    /// Verify HMAC in constant time using ring
    pub fn hmac_verify(key: &[u8], data: &[u8], expected: &[u8; 32]) -> bool {
        let key = hmac::Key::new(hmac::HMAC_SHA256, key);
        hmac::verify(&key, data, expected).is_ok()
    }
}

/// Password hashing using Argon2id
///
/// Production-quality password hashing with memory-hard algorithm.
pub mod password {
    use super::*;
    use argon2::{
        Argon2,
        password_hash::{
            PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
        },
    };

    /// Hash a password using Argon2id
    ///
    /// Returns a PHC-formatted string containing the hash and parameters.
    /// Store this in your database instead of the raw password.
    ///
    /// # Example
    /// ```
    /// use ifa_std::stacks::crypto::password;
    /// let hash = password::hash("my_password").unwrap();
    /// // Store hash in database...
    /// ```
    pub fn hash(password: &str) -> Result<String, CryptoError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|_| CryptoError::InvalidInput("Password hashing failed".to_string()))
    }

    /// Verify a password against a stored hash
    ///
    /// Returns true if the password matches the hash.
    ///
    /// # Example
    /// ```
    /// use ifa_std::stacks::crypto::password;
    /// let hash = password::hash("my_password").unwrap();
    /// let matches = password::verify("my_password", &hash).unwrap();
    /// assert!(matches);
    /// ```
    pub fn verify(password: &str, hash: &str) -> Result<bool, CryptoError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| CryptoError::InvalidInput("Invalid hash format".to_string()))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

/// Constant-time comparison to prevent timing attacks
pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Base64 encoding/decoding with proper error handling
pub mod base64 {
    use super::CryptoError;

    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub fn encode(data: &[u8]) -> String {
        let mut result = String::with_capacity((data.len() + 2) / 3 * 4);

        for chunk in data.chunks(3) {
            let b0 = chunk[0] as usize;
            result.push(ALPHABET[b0 >> 2] as char);

            if chunk.len() > 1 {
                let b1 = chunk[1] as usize;
                result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

                if chunk.len() > 2 {
                    let b2 = chunk[2] as usize;
                    result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
                    result.push(ALPHABET[b2 & 0x3f] as char);
                } else {
                    result.push(ALPHABET[(b1 & 0x0f) << 2] as char);
                    result.push('=');
                }
            } else {
                result.push(ALPHABET[(b0 & 0x03) << 4] as char);
                result.push_str("==");
            }
        }

        result
    }

    pub fn decode(encoded: &str) -> Result<Vec<u8>, CryptoError> {
        let chars: Vec<char> = encoded
            .chars()
            .filter(|&c| c != '=' && !c.is_whitespace())
            .collect();
        let mut result = Vec::with_capacity(chars.len() * 3 / 4);

        for chunk in chars.chunks(4) {
            if chunk.is_empty() {
                break;
            }

            let indices: Result<Vec<usize>, _> = chunk
                .iter()
                .map(|&c| {
                    ALPHABET
                        .iter()
                        .position(|&b| b as char == c)
                        .ok_or_else(|| {
                            CryptoError::InvalidInput(format!("Invalid base64 char: {}", c))
                        })
                })
                .collect();

            let indices = indices?;

            if indices.len() >= 2 {
                result.push(((indices[0] << 2) | (indices[1] >> 4)) as u8);
            }
            if indices.len() >= 3 {
                result.push((((indices[1] & 0x0f) << 4) | (indices[2] >> 2)) as u8);
            }
            if indices.len() >= 4 {
                result.push((((indices[2] & 0x03) << 6) | indices[3]) as u8);
            }
        }

        Ok(result)
    }
}

/// Hex encoding/decoding
pub mod hex {
    use super::CryptoError;

    pub fn encode(data: &[u8]) -> String {
        let mut s = String::with_capacity(data.len() * 2);
        use std::fmt::Write;
        for b in data {
            write!(s, "{:02x}", b).unwrap();
        }
        s
    }

    pub fn decode(encoded: &str) -> Result<Vec<u8>, CryptoError> {
        if encoded.len() % 2 != 0 {
            return Err(CryptoError::InvalidInput(
                "Odd length hex string".to_string(),
            ));
        }

        let mut result = Vec::with_capacity(encoded.len() / 2);
        let chars: Vec<char> = encoded.chars().collect();

        for chunk in chars.chunks(2) {
            let byte = u8::from_str_radix(&format!("{}{}", chunk[0], chunk[1]), 16)
                .map_err(|_| CryptoError::InvalidInput("Invalid hex character".to_string()))?;
            result.push(byte);
        }

        Ok(result)
    }
}

/// Cryptographically secure random number generator
///
/// wrapper around `rand::rngs::OsRng`
pub struct SecureRng;

impl SecureRng {
    /// Create new RNG interface
    pub fn new() -> Result<Self, CryptoError> {
        Ok(SecureRng)
    }

    /// Fill buffer with random bytes using OS entropy
    pub fn fill_bytes(&self, dest: &mut [u8]) -> Result<(), CryptoError> {
        use rand::RngCore;
        use rand::rngs::OsRng;
        OsRng
            .try_fill_bytes(dest)
            .map_err(|_| CryptoError::RngError)
    }

    /// Generate random bytes
    pub fn gen_bytes(&self, count: usize) -> Result<Vec<u8>, CryptoError> {
        let mut bytes = vec![0u8; count];
        self.fill_bytes(&mut bytes)?;
        Ok(bytes)
    }

    /// Generate random u64
    pub fn gen_u64(&self) -> Result<u64, CryptoError> {
        use rand::RngCore;
        use rand::rngs::OsRng;
        Ok(OsRng.next_u64())
    }

    /// Generate random value in range [0, max)
    pub fn gen_range(&self, max: u64) -> Result<u64, CryptoError> {
        if max == 0 {
            return Err(CryptoError::InvalidInput(
                "Range max cannot be 0".to_string(),
            ));
        }
        use rand::Rng;
        use rand::rngs::OsRng;
        Ok(OsRng.gen_range(0..max))
    }
}

impl Default for SecureRng {
    fn default() -> Self {
        Self::new().expect("Failed to initialize RNG")
    }
}

/// Generate UUID v4
pub fn uuid_v4() -> Result<String, CryptoError> {
    let rng = SecureRng::new()?;
    let mut bytes = [0u8; 16];
    rng.fill_bytes(&mut bytes)?;

    // Set version 4
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    // Set variant
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    Ok(format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    ))
}

/// Secret storage with zeroization on drop
pub struct Secret {
    data: Vec<u8>,
}

impl Secret {
    pub fn new(data: Vec<u8>) -> Self {
        Secret { data }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        // Zero out memory before dropping
        // SAFETY: We use explicit pointer arithmetic with bounds checks to ensure
        // we never write outside the allocated buffer.
        let len = self.data.len();
        if len > 0 {
            let ptr = self.data.as_mut_ptr();
            for i in 0..len {
                unsafe {
                    // VERIFY: ptr + i is always within [ptr, ptr + len) because i < len
                    let target_ptr = ptr.add(i);
                    std::ptr::write_volatile(target_ptr, 0);
                }
            }
        }
        std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
    }
}

/// Secure secret store with zeroization
pub struct SecretStore {
    secrets: HashMap<String, Secret>,
}

impl SecretStore {
    pub fn new() -> Self {
        SecretStore {
            secrets: HashMap::new(),
        }
    }

    pub fn store(&mut self, key: &str, value: Vec<u8>) {
        self.secrets.insert(key.to_string(), Secret::new(value));
    }

    pub fn get(&self, key: &str) -> Option<&[u8]> {
        self.secrets.get(key).map(|s| s.as_bytes())
    }

    pub fn remove(&mut self, key: &str) -> bool {
        self.secrets.remove(key).is_some()
    }

    pub fn clear(&mut self) {
        self.secrets.clear();
    }
}

impl Default for SecretStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SecretStore {
    fn drop(&mut self) {
        self.clear();
    }
}

/// Simplified password hashing using HMAC-SHA256 iterated 1000 times.
///
/// # ⚠️ Deprecated
///
/// This is NOT Argon2id. It is a simplified PBKDF2-like construction intended
/// only for internal test fixtures. **Do not use it for storing real passwords.**
///
/// Use [`password::hash`] and [`password::verify`] for production password storage.
/// Those use Argon2id with a random salt via the `argon2` crate.
#[deprecated(since = "0.1.0", note = "Use password::hash / password::verify (Argon2id) instead")]
pub fn hash_password(password: &[u8], salt: &[u8]) -> [u8; 32] {
    // Iterating HMAC-SHA256 provides some key-stretching but lacks the
    // memory-hardness and parallelism resistance of Argon2.
    let mut result = hash::hmac_sha256(password, salt);
    for _ in 0..1000 {
        result = hash::hmac_sha256(password, &result);
    }
    result
}

/// Verify password against hash
pub fn verify_password(password: &[u8], salt: &[u8], expected: &[u8; 32]) -> bool {
    let computed = hash_password(password, salt);
    constant_time_compare(&computed, expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let original = b"Hello, World!";
        let encoded = base64::encode(original);
        let decoded = base64::decode(&encoded).unwrap();
        assert_eq!(original.to_vec(), decoded);
    }

    #[test]
    fn test_base64_padding() {
        assert_eq!(base64::encode(b"f"), "Zg==");
        assert_eq!(base64::encode(b"fo"), "Zm8=");
        assert_eq!(base64::encode(b"foo"), "Zm9v");
    }

    #[test]
    fn test_hex_roundtrip() {
        let original = b"Test123";
        let encoded = hex::encode(original);
        let decoded = hex::decode(&encoded).unwrap();
        assert_eq!(original.to_vec(), decoded);
    }

    #[test]
    fn test_constant_time_compare() {
        let a = [1u8, 2, 3, 4];
        let b = [1u8, 2, 3, 4];
        let c = [1u8, 2, 3, 5];

        assert!(constant_time_compare(&a, &b));
        assert!(!constant_time_compare(&a, &c));
    }

    #[test]
    fn test_hmac_verify() {
        let key = b"secret";
        let data = b"message";
        let mac = hash::hmac_sha256(key, data);

        assert!(hash::hmac_verify(key, data, &mac));

        let mut bad_mac = mac;
        bad_mac[0] ^= 1;
        assert!(!hash::hmac_verify(key, data, &bad_mac));
    }

    #[test]
    fn test_rng() {
        let rng = SecureRng::new().unwrap();
        let a = rng.gen_u64().unwrap();
        let b = rng.gen_u64().unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn test_uuid_format() {
        let uuid = uuid_v4().unwrap();
        assert_eq!(uuid.len(), 36);
        assert_eq!(&uuid[8..9], "-");
        assert_eq!(&uuid[13..14], "-");
        assert_eq!(&uuid[14..15], "4"); // Version 4
    }

    #[test]
    fn test_secret_store() {
        let mut store = SecretStore::new();
        store.store("key", b"secret".to_vec());
        assert_eq!(store.get("key"), Some(b"secret".as_slice()));
        assert!(store.remove("key"));
        assert!(store.get("key").is_none());
    }

    #[test]
    fn test_password_hash() {
        let password = b"mypassword";
        let salt = b"randomsalt";
        let hash = hash_password(password, salt);

        assert!(verify_password(password, salt, &hash));
        assert!(!verify_password(b"wrongpassword", salt, &hash));
    }
}
