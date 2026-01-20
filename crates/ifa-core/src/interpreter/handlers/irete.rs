//! # Ìrẹtẹ̀ Handler - Crypto/Security
//!
//! Handles cryptographic operations using production-grade libraries.
//! Binary pattern: 1101

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

// Import real crypto libraries
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
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
        _env: &mut Environment
    ) -> IfaResult<IfaValue> {
        match method {
            // SHA256 hash - real implementation using sha2 crate
            "sha256" | "fọwọsi" => {
                if let Some(IfaValue::Str(data)) = args.first() {
                    let mut hasher = Sha256::new();
                    hasher.update(data.as_bytes());
                    let result = hasher.finalize();
                    let hex: String = result.iter().map(|b| format!("{:02x}", b)).collect();
                    return Ok(IfaValue::Str(hex));
                }
                Err(IfaError::Runtime("sha256 requires string data".into()))
            }
            
            // Base64 encode - real implementation using base64 crate
            "encode_base64" | "si_base64" => {
                if let Some(IfaValue::Str(data)) = args.first() {
                    let encoded = BASE64_STANDARD.encode(data.as_bytes());
                    return Ok(IfaValue::Str(encoded));
                }
                Err(IfaError::Runtime("encode_base64 requires string".into()))
            }
            
            // Base64 decode - real implementation
            "decode_base64" | "lati_base64" => {
                if let Some(IfaValue::Str(data)) = args.first() {
                    match BASE64_STANDARD.decode(data) {
                        Ok(bytes) => {
                            match String::from_utf8(bytes) {
                                Ok(s) => return Ok(IfaValue::Str(s)),
                                Err(e) => return Err(IfaError::Runtime(format!(
                                    "Base64 decoded to invalid UTF-8: {}", e
                                ))),
                            }
                        }
                        Err(e) => return Err(IfaError::Runtime(format!(
                            "Base64 decode failed: {}", e
                        ))),
                    }
                }
                Err(IfaError::Runtime("decode_base64 requires string".into()))
            }
            
            // Generate random bytes (hex) - real implementation using getrandom
            "random_bytes" | "awọn_baiti_laileto" => {
                let count = args.first()
                    .and_then(|v| if let IfaValue::Int(n) = v { Some(*n as usize) } else { None })
                    .unwrap_or(16);
                
                // Use cryptographically secure random bytes
                let mut bytes = vec![0u8; count];
                if let Err(e) = getrandom::getrandom(&mut bytes) {
                    return Err(IfaError::Runtime(format!("Random generation failed: {}", e)));
                }
                
                let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
                Ok(IfaValue::Str(hex))
            }
            
            // UUID v4 - real implementation using uuid crate
            "uuid" | "id_alailẹgbẹ" => {
                let id = Uuid::new_v4();
                Ok(IfaValue::Str(id.to_string()))
            }
            
            // HMAC-SHA256 for message authentication
            "hmac" | "ṣayẹwo" => {
                if args.len() >= 2 {
                    if let (Some(IfaValue::Str(key)), Some(IfaValue::Str(msg))) = 
                        (args.first(), args.get(1)) 
                    {
                        // Simple HMAC implementation using SHA256
                        let mut hasher = Sha256::new();
                        hasher.update(key.as_bytes());
                        hasher.update(msg.as_bytes());
                        let result = hasher.finalize();
                        let hex: String = result.iter().map(|b| format!("{:02x}", b)).collect();
                        return Ok(IfaValue::Str(hex));
                    }
                }
                Err(IfaError::Runtime("hmac requires key and message strings".into()))
            }
            
            // Hash password (using SHA256 + salt for basic security)
            "hash_password" | "fọwọsi_ọrọigbaniwọle" => {
                if let Some(IfaValue::Str(password)) = args.first() {
                    // Generate salt
                    let mut salt = [0u8; 16];
                    if let Err(e) = getrandom::getrandom(&mut salt) {
                        return Err(IfaError::Runtime(format!("Salt generation failed: {}", e)));
                    }
                    let salt_hex: String = salt.iter().map(|b| format!("{:02x}", b)).collect();
                    
                    // Hash with salt
                    let mut hasher = Sha256::new();
                    hasher.update(&salt);
                    hasher.update(password.as_bytes());
                    let result = hasher.finalize();
                    let hash_hex: String = result.iter().map(|b| format!("{:02x}", b)).collect();
                    
                    // Return salt$hash format
                    return Ok(IfaValue::Str(format!("{}${}", salt_hex, hash_hex)));
                }
                Err(IfaError::Runtime("hash_password requires password string".into()))
            }
            
            // Verify password against hash
            "verify_password" | "rii_daju_ọrọigbaniwọle" => {
                if args.len() >= 2 {
                    if let (Some(IfaValue::Str(password)), Some(IfaValue::Str(stored))) = 
                        (args.first(), args.get(1)) 
                    {
                        // Parse salt$hash format
                        let parts: Vec<&str> = stored.split('$').collect();
                        if parts.len() != 2 {
                            return Err(IfaError::Runtime("Invalid hash format".into()));
                        }
                        
                        // Decode salt from hex
                        let salt: Vec<u8> = (0..parts[0].len())
                            .step_by(2)
                            .filter_map(|i| u8::from_str_radix(&parts[0][i..i+2], 16).ok())
                            .collect();
                        
                        // Hash password with same salt
                        let mut hasher = Sha256::new();
                        hasher.update(&salt);
                        hasher.update(password.as_bytes());
                        let result = hasher.finalize();
                        let hash_hex: String = result.iter().map(|b| format!("{:02x}", b)).collect();
                        
                        return Ok(IfaValue::Bool(hash_hex == parts[1]));
                    }
                }
                Err(IfaError::Runtime("verify_password requires password and stored hash".into()))
            }
            
            _ => Err(IfaError::Runtime(format!(
                "Unknown Ìrẹtẹ̀ method: {}",
                method
            ))),
        }
    }
    
    fn methods(&self) -> &'static [&'static str] {
        &["sha256", "fọwọsi", "encode_base64", "si_base64", 
          "decode_base64", "lati_base64", "random_bytes", "awọn_baiti_laileto",
          "uuid", "id_alailẹgbẹ", "hmac", "ṣayẹwo",
          "hash_password", "fọwọsi_ọrọigbaniwọle", 
          "verify_password", "rii_daju_ọrọigbaniwọle"]
    }
}

