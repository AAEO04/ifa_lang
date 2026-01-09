//! Integration tests for Crypto Stack
//!
//! Tests the ring-based SHA-256/512/HMAC and argon2 password hashing.

use ifa_std::stacks::crypto::{constant_time_compare, hash, password};

#[test]
fn test_sha256_produces_32_bytes() {
    let result = hash::sha256(b"hello world");
    assert_eq!(result.len(), 32);
}

#[test]
fn test_sha256_deterministic() {
    let a = hash::sha256(b"test input");
    let b = hash::sha256(b"test input");
    assert_eq!(a, b);
}

#[test]
fn test_sha256_different_inputs() {
    let a = hash::sha256(b"input1");
    let b = hash::sha256(b"input2");
    assert_ne!(a, b);
}

#[test]
fn test_sha256_empty_input() {
    // SHA-256 of empty string is well-known
    let result = hash::sha256(b"");
    // Expected: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    assert_eq!(result.len(), 32);
}

#[test]
fn test_sha256_hex_format() {
    let hex = hash::sha256_hex(b"test");
    assert_eq!(hex.len(), 64); // 32 bytes = 64 hex chars
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_sha512_produces_64_bytes() {
    let result = hash::sha512(b"hello world");
    assert_eq!(result.len(), 64);
}

#[test]
fn test_hmac_sha256_produces_32_bytes() {
    let key = b"secret_key";
    let data = b"message to authenticate";
    let result = hash::hmac_sha256(key, data);
    assert_eq!(result.len(), 32);
}

#[test]
fn test_hmac_verify_correct_tag() {
    let key = b"my_secret";
    let data = b"important data";
    let tag = hash::hmac_sha256(key, data);

    assert!(hash::hmac_verify(key, data, &tag));
}

#[test]
fn test_hmac_verify_wrong_tag() {
    let key = b"my_secret";
    let data = b"important data";
    let wrong_tag = [0u8; 32];

    assert!(!hash::hmac_verify(key, data, &wrong_tag));
}

#[test]
fn test_hmac_verify_wrong_key() {
    let key1 = b"key1";
    let key2 = b"key2";
    let data = b"data";
    let tag = hash::hmac_sha256(key1, data);

    assert!(!hash::hmac_verify(key2, data, &tag));
}

#[test]
fn test_constant_time_compare_equal() {
    assert!(constant_time_compare(b"hello", b"hello"));
}

#[test]
fn test_constant_time_compare_different() {
    assert!(!constant_time_compare(b"hello", b"world"));
}

#[test]
fn test_constant_time_compare_different_lengths() {
    assert!(!constant_time_compare(b"short", b"longer"));
}

#[test]
fn test_password_hash_and_verify() {
    let password_str = "my_secure_password_123!";

    // Hash the password
    let hash = password::hash(password_str).expect("Failed to hash password");

    // Verify correct password
    let valid = password::verify(password_str, &hash).expect("Failed to verify password");
    assert!(valid);

    // Verify wrong password
    let invalid = password::verify("wrong_password", &hash).expect("Failed to verify password");
    assert!(!invalid);
}

#[test]
fn test_password_hash_different_each_time() {
    let password_str = "same_password";

    let hash1 = password::hash(password_str).expect("Failed to hash");
    let hash2 = password::hash(password_str).expect("Failed to hash");

    // Hashes should be different due to random salt
    assert_ne!(hash1, hash2);

    // But both should verify correctly
    assert!(password::verify(password_str, &hash1).unwrap());
    assert!(password::verify(password_str, &hash2).unwrap());
}
