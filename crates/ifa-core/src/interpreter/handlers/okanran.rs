//! # Ọ̀kànràn Handler - Errors/Assertions
//!
//! Handles error throwing and assertions.
//! Binary pattern: 0001

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{EnvRef, OduHandler};

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
        _env: &EnvRef,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        let arg0 = args.first();

        match method {
            // Throw error
            "ta" | "throw" | "error" => {
                let msg = arg0
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "Unknown error".to_string());
                Err(IfaError::Runtime(msg))
            }

            // Panic (fatal error)
            "jagun" | "panic" => {
                let msg = arg0
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "Panic!".to_string());
                panic!("{}", msg);
            }

            // Assert condition
            "jẹri" | "assert" => {
                let cond = arg0.map(|v| v.is_truthy()).unwrap_or(false);

                if cond {
                    Ok(IfaValue::bool(true))
                } else {
                    let msg = args
                        .get(1)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "Assertion failed".to_string());
                    Err(IfaError::Runtime(format!("[assert] {}", msg)))
                }
            }

            // Assert with message (explicit)
            "jẹri_asọ" | "assert_msg" => {
                if let Some(cond_val) = arg0 {
                    let cond = cond_val.is_truthy();
                    if !cond {
                        let msg = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                        return Err(IfaError::Runtime(format!("[assert] {}", msg)));
                    }
                    return Ok(IfaValue::bool(true));
                }
                Err(IfaError::Runtime(
                    "assert_msg requires condition and message".into(),
                ))
            }

            // Assert equal
            "jẹri_bakan" | "assert_eq" => {
                if let (Some(left), Some(right)) = (arg0, args.get(1)) {
                    if left.is_equal(right) {
                        return Ok(IfaValue::bool(true));
                    }
                    let msg = args
                        .get(2)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| format!("Expected {:?} == {:?}", left, right));
                    return Err(IfaError::Runtime(format!("[assert_eq] {}", msg)));
                }
                Err(IfaError::Runtime("assert_eq requires two values".into()))
            }

            // Assert not equal
            "jẹri_yato" | "assert_ne" => {
                if let (Some(left), Some(right)) = (arg0, args.get(1)) {
                    if !left.is_equal(right) {
                        return Ok(IfaValue::bool(true));
                    }
                    let msg = args
                        .get(2)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| format!("Expected {:?} != {:?}", left, right));
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
        &[
            "ta",
            "throw",
            "error",
            "jagun",
            "panic",
            "jẹri",
            "assert",
            "jẹri_asọ",
            "assert_msg",
            "jẹri_bakan",
            "assert_eq",
            "jẹri_yato",
            "assert_ne",
            "ko_ṣee_de",
            "unreachable",
        ]
    }
}
