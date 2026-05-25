//! # Ìrẹtẹ̀ Handler - Crypto/Security
//!
//! Handles cryptographic operations using standard library and gating them with Esu.
//! Binary pattern: 1101

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;
use super::{EnvRef, OduHandler};

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
        _env: &EnvRef,
        _output: &mut Vec<String>,
        capabilities: &ifa_types::capability::CapabilitySet,
    ) -> IfaResult<IfaValue> {
        #[cfg(feature = "std")]
        {
            let esu = ifa_std::esu::Esu::new(capabilities.clone());
            let irete = ifa_std::odu::irete::Irete::new(esu.clone());
            let arg0 = args.get(0);

            match method {
                // SHA256 hash
                "sha256" | "fọwọsi" => {
                    if let Some(IfaValue::Str(data)) = arg0 {
                        let hex = irete.sha256_hex(data.as_bytes())?;
                        Ok(IfaValue::str(hex))
                    } else {
                        Err(IfaError::Runtime("sha256 requires string data".into()))
                    }
                }

                // Base64 encode
                "encode_base64" | "si_base64" => {
                    if let Some(IfaValue::Str(data)) = arg0 {
                        let encoded = irete.base64_encode(data.as_bytes())?;
                        Ok(IfaValue::str(encoded))
                    } else {
                        Err(IfaError::Runtime("encode_base64 requires string".into()))
                    }
                }

                // Base64 decode
                "decode_base64" | "lati_base64" => {
                    if let Some(IfaValue::Str(data)) = arg0 {
                        let bytes = irete.base64_decode(&data)?;
                        match String::from_utf8(bytes) {
                            Ok(s) => Ok(IfaValue::str(s)),
                            Err(e) => Err(IfaError::Runtime(format!(
                                "Base64 decoded to invalid UTF-8: {}",
                                e
                            ))),
                        }
                    } else {
                        Err(IfaError::Runtime("decode_base64 requires string".into()))
                    }
                }

                // Generate random bytes (hex)
                "random_bytes" | "awọn_baiti_laileto" => {
                    let count = if let Some(IfaValue::Int(n)) = arg0 {
                        *n as usize
                    } else {
                        16
                    };
                    let bytes = irete.random_bytes(count)?;
                    let hex = irete.hex_encode(&bytes)?;
                    Ok(IfaValue::str(hex))
                }

                // UUID v4
                "uuid" | "id_alailẹgbẹ" => {
                    esu.enforce_crossroads(&ifa_types::capability::Ofun::Crypto, "Irete.uuid")?;
                    let id = uuid::Uuid::new_v4();
                    Ok(IfaValue::str(id.to_string()))
                }

                // HMAC-SHA256
                "hmac" | "ṣayẹwo" => {
                    if let (Some(IfaValue::Str(key)), Some(IfaValue::Str(msg))) = (arg0, args.get(1)) {
                        let tag = irete.hmac_sha256(key.as_bytes(), msg.as_bytes())?;
                        let hex = irete.hex_encode(&tag)?;
                        Ok(IfaValue::str(hex))
                    } else {
                        Err(IfaError::Runtime("hmac requires key and message strings".into()))
                    }
                }

                // Hash password
                "hash_password" | "fọwọsi_ọrọigbaniwọle" => {
                    if let Some(IfaValue::Str(password)) = arg0 {
                        esu.enforce_crossroads(&ifa_types::capability::Ofun::Crypto, "Irete.hash_password")?;
                        // Generate salt using standard random_bytes under crypto gate
                        let salt = irete.random_bytes(16)?;
                        let salt_hex = irete.hex_encode(&salt)?;

                        // Hash with salt using sha256
                        let mut data_to_hash = salt;
                        data_to_hash.extend_from_slice(password.as_bytes());
                        let hashed = irete.sha256(&data_to_hash)?;
                        let hash_hex = irete.hex_encode(&hashed)?;

                        // Return salt$hash format
                        Ok(IfaValue::str(format!("{}${}", salt_hex, hash_hex)))
                    } else {
                        Err(IfaError::Runtime("hash_password requires password string".into()))
                    }
                }

                // Verify password against hash
                "verify_password" | "rii_daju_ọrọigbaniwọle" => {
                    if let (Some(IfaValue::Str(password)), Some(IfaValue::Str(stored))) = (arg0, args.get(1)) {
                        esu.enforce_crossroads(&ifa_types::capability::Ofun::Crypto, "Irete.verify_password")?;
                        // Parse salt$hash format
                        let parts: Vec<&str> = stored.split('$').collect();
                        if parts.len() != 2 {
                            return Err(IfaError::Runtime("Invalid hash format".into()));
                        }

                        // Decode salt from hex
                        let salt = irete.hex_decode(parts[0])?;

                        // Hash password with same salt
                        let mut data_to_hash = salt;
                        data_to_hash.extend_from_slice(password.as_bytes());
                        let hashed = irete.sha256(&data_to_hash)?;
                        let hash_hex = irete.hex_encode(&hashed)?;

                        Ok(IfaValue::bool(hash_hex == parts[1]))
                    } else {
                        Err(IfaError::Runtime("verify_password requires password and stored hash".into()))
                    }
                }

                _ => Err(IfaError::Runtime(format!("Unknown Ìrẹtẹ̀ method: {}", method))),
            }
        }
        #[cfg(not(feature = "std"))]
        {
            let _ = args;
            let _ = capabilities;
            Err(IfaError::Runtime("std library not enabled".into()))
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
