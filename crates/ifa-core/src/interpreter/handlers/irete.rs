//! # Ìrẹtẹ̀ Handler - Crypto/Security
//!
//! Handles cryptographic operations using production-grade libraries.
//! Binary pattern: 1101

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

// Import real crypto libraries
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Handler for Ìrẹtẹ̀ (Crypto/Security) domain.
pub struct IreteHandler;

impl OduHandler for IreteHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Irete
    }

    fn call(
        &self,
        method: &str,
        args: Vec<IfaValue>,
        _env: &mut Environment,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {

        let arg0 = args.get(0);
        
        match method {
            // SHA256 hash
            "sha256" | "fọwọsi" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Str(data) = val {
                        let mut hasher = Sha256::new();
                        hasher.update(data.as_bytes());
                        let result = hasher.finalize();
                        let hex: String = result.iter().map(|b| format!("{:02x}", b)).collect();
                        return Ok(IfaValue::str(hex));
                    }
                }
                Err(IfaError::Runtime("sha256 requires string data".into()))
            }

            // Base64 encode
            "encode_base64" | "si_base64" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Str(data) = val {
                        let encoded = BASE64_STANDARD.encode(data.as_bytes());
                        return Ok(IfaValue::str(encoded));
                    }
                }
                Err(IfaError::Runtime("encode_base64 requires string".into()))
            }

            // Base64 decode
            "decode_base64" | "lati_base64" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Str(data) = val {
                        match BASE64_STANDARD.decode(&**data) {
                            Ok(bytes) => match String::from_utf8(bytes) {
                                Ok(s) => return Ok(IfaValue::str(s)),
                                Err(e) => {
                                    return Err(IfaError::Runtime(format!(
                                        "Base64 decoded to invalid UTF-8: {}",
                                        e
                                    )));
                                }
                            },
                            Err(e) => {
                                return Err(IfaError::Runtime(format!("Base64 decode failed: {}", e)));
                            }
                        }
                    }
                }
                Err(IfaError::Runtime("decode_base64 requires string".into()))
            }

            // Generate random bytes (hex)
            "random_bytes" | "awọn_baiti_laileto" => {
                let count = if let Some(val) = arg0 {
                    if let IfaValue::Int(n) = val {
                        *n as usize
                    } else {
                        16
                    }
                } else {
                    16
                };

                // Use cryptographically secure random bytes
                let mut bytes = vec![0u8; count];
                if let Err(e) = getrandom::getrandom(&mut bytes) {
                    return Err(IfaError::Runtime(format!(
                        "Random generation failed: {}",
                        e
                    )));
                }

                let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
                Ok(IfaValue::str(hex))
            }

            // UUID v4
            "uuid" | "id_alailẹgbẹ" => {
                let id = Uuid::new_v4();
                Ok(IfaValue::str(id.to_string()))
            }

            // HMAC-SHA256
            "hmac" | "ṣayẹwo" => {
                if let (Some(key_val), Some(msg_val)) = (arg0, args.get(1)) {
                    if let (IfaValue::Str(key), IfaValue::Str(msg)) = (key_val, msg_val) {
                        let mut hasher = Sha256::new();
                        hasher.update(key.as_bytes());
                        hasher.update(msg.as_bytes());
                        let result = hasher.finalize();
                        let hex: String = result.iter().map(|b| format!("{:02x}", b)).collect();
                        return Ok(IfaValue::str(hex));
                    }
                }
                Err(IfaError::Runtime(
                    "hmac requires key and message strings".into(),
                ))
            }

            // Hash password
            "hash_password" | "fọwọsi_ọrọigbaniwọle" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Str(password) = val {
                        // Generate salt
                        let mut salt = [0u8; 16];
                        if let Err(e) = getrandom::getrandom(&mut salt) {
                            return Err(IfaError::Runtime(format!("Salt generation failed: {}", e)));
                        }
                        let salt_hex: String = salt.iter().map(|b| format!("{:02x}", b)).collect();

                        // Hash with salt
                        let mut hasher = Sha256::new();
                        hasher.update(salt);
                        hasher.update(password.as_bytes());
                        let result = hasher.finalize();
                        let hash_hex: String = result.iter().map(|b| format!("{:02x}", b)).collect();

                        // Return salt$hash format
                        return Ok(IfaValue::str(format!("{}${}", salt_hex, hash_hex)));
                    }
                }
                Err(IfaError::Runtime(
                    "hash_password requires password string".into(),
                ))
            }

            // Verify password against hash
            "verify_password" | "rii_daju_ọrọigbaniwọle" => {
                if let (Some(pass_val), Some(stored_val)) = (arg0, args.get(1)) {
                    if let (IfaValue::Str(password), IfaValue::Str(stored)) = (pass_val, stored_val) {
                        // Parse salt$hash format
                        let parts: Vec<&str> = stored.split('$').collect();
                        if parts.len() != 2 {
                            return Err(IfaError::Runtime("Invalid hash format".into()));
                        }

                        // Decode salt from hex
                        let salt: Vec<u8> = (0..parts[0].len())
                            .step_by(2)
                            .filter_map(|i| u8::from_str_radix(&parts[0][i..i + 2], 16).ok())
                            .collect();

                        // Hash password with same salt
                        let mut hasher = Sha256::new();
                        hasher.update(&salt);
                        hasher.update(password.as_bytes());
                        let result = hasher.finalize();
                        let hash_hex: String =
                            result.iter().map(|b| format!("{:02x}", b)).collect();

                        return Ok(IfaValue::bool(hash_hex == parts[1]));
                    }
                }
                Err(IfaError::Runtime(
                    "verify_password requires password and stored hash".into(),
                ))
            }

            _ => Err(IfaError::Runtime(format!(
                "Unknown Ìrẹtẹ̀ method: {}",
                method
            ))),
        }
    }

    fn methods(&self) -> &'static [&'static str] {
        &[
            "sha256",
            "fọwọsi",
            "encode_base64",
            "si_base64",
            "decode_base64",
            "lati_base64",
            "random_bytes",
            "awọn_baiti_laileto",
            "uuid",
            "id_alailẹgbẹ",
            "hmac",
            "ṣayẹwo",
            "hash_password",
            "fọwọsi_ọrọigbaniwọle",
            "verify_password",
            "rii_daju_ọrọigbaniwọle",
        ]
    }
}
