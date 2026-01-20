//! # Ọ̀kànràn Handler - Errors/Assertions
//!
//! Handles error throwing and assertions.
//! Binary pattern: 0001

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

/// Handler for Ọ̀kànràn (Errors/Assertions) domain.
pub struct OkanranHandler;

impl OduHandler for OkanranHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Okanran
    }
    
    fn call(
        &self, 
        method: &str, 
        args: Vec<IfaValue>, 
        _env: &mut Environment
    ) -> IfaResult<IfaValue> {
        match method {
            // Throw error
            "ta" | "throw" | "error" => {
                let msg = args.first()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "Unknown error".to_string());
                Err(IfaError::Runtime(msg))
            }
            
            // Panic (fatal error)
            "jagun" | "panic" => {
                let msg = args.first()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "Panic!".to_string());
                panic!("{}", msg);
            }
            
            // Assert condition
            "jẹri" | "assert" => {
                let cond = args.first()
                    .map(|v| v.is_truthy())
                    .unwrap_or(false);
                    
                if cond {
                    Ok(IfaValue::Bool(true))
                } else {
                    let msg = args.get(1)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "Assertion failed".to_string());
                    Err(IfaError::Runtime(format!("[assert] {}", msg)))
                }
            }
            
            // Assert with message (explicit)
            "jẹri_asọ" | "assert_msg" => {
                if args.len() >= 2 {
                    let cond = args[0].is_truthy();
                    if !cond {
                        return Err(IfaError::Runtime(format!("[assert] {}", args[1])));
                    }
                    return Ok(IfaValue::Bool(true));
                }
                Err(IfaError::Runtime("assert_msg requires condition and message".into()))
            }
            
            // Assert equal
            "jẹri_bakan" | "assert_eq" => {
                if args.len() >= 2 {
                    if args[0] == args[1] {
                        return Ok(IfaValue::Bool(true));
                    }
                    let msg = args.get(2)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| format!("Expected {:?} == {:?}", args[0], args[1]));
                    return Err(IfaError::Runtime(format!("[assert_eq] {}", msg)));
                }
                Err(IfaError::Runtime("assert_eq requires two values".into()))
            }
            
            // Assert not equal
            "jẹri_yato" | "assert_ne" => {
                if args.len() >= 2 {
                    if args[0] != args[1] {
                        return Ok(IfaValue::Bool(true));
                    }
                    let msg = args.get(2)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| format!("Expected {:?} != {:?}", args[0], args[1]));
                    return Err(IfaError::Runtime(format!("[assert_ne] {}", msg)));
                }
                Err(IfaError::Runtime("assert_ne requires two values".into()))
            }
            
            // Unreachable code marker
            "ko_ṣee_de" | "unreachable" => {
                Err(IfaError::Runtime("Reached unreachable code".into()))
            }
            
            _ => Err(IfaError::Runtime(format!(
                "Unknown Ọ̀kànràn method: {}",
                method
            ))),
        }
    }
    
    fn methods(&self) -> &'static [&'static str] {
        &["ta", "throw", "error", "jagun", "panic", "jẹri", "assert",
          "jẹri_asọ", "assert_msg", "jẹri_bakan", "assert_eq", 
          "jẹri_yato", "assert_ne", "ko_ṣee_de", "unreachable"]
    }
}
