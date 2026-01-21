//! # Ọ̀gbè Handler - System/Lifecycle
//!
//! Handles system operations, type introspection, and assertions.
//! Binary pattern: 1111

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

/// Handler for Ọ̀gbè (System/Lifecycle) domain.
pub struct OgbeHandler;

impl OduHandler for OgbeHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Ogbe
    }

    fn call(
        &self,
        method: &str,
        args: Vec<IfaValue>,
        _env: &mut Environment,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        match method {
            // Type introspection
            "type" | "iru" => {
                let type_name = args.first().map(|v| v.type_name()).unwrap_or("null");
                Ok(IfaValue::Str(type_name.to_string()))
            }

            // Length
            "len" | "gigun" => {
                let len = args.first().map(|v| v.len() as i64).unwrap_or(0);
                Ok(IfaValue::Int(len))
            }

            // Assertion
            "assert" | "jẹri" => {
                let cond = args.first().map(|v| v.is_truthy()).unwrap_or(false);

                if cond {
                    Ok(IfaValue::Bool(true))
                } else {
                    let msg = args
                        .get(1)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "Assertion failed".to_string());
                    Err(IfaError::Runtime(format!("[assert] {}", msg)))
                }
            }

            // Format/stringify
            "format" | "ṣẹda" => {
                let output: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                Ok(IfaValue::Str(output.join(" ")))
            }

            // Parse integer
            "parse_int" => {
                let val = args
                    .first()
                    .and_then(|v| match v {
                        IfaValue::Str(s) => s.trim().parse::<i64>().ok(),
                        IfaValue::Int(n) => Some(*n),
                        IfaValue::Float(f) => Some(*f as i64),
                        _ => None,
                    })
                    .unwrap_or(0);
                Ok(IfaValue::Int(val))
            }

            // Parse float
            "parse_float" => {
                let val = args
                    .first()
                    .and_then(|v| match v {
                        IfaValue::Str(s) => s.trim().parse::<f64>().ok(),
                        IfaValue::Float(f) => Some(*f),
                        IfaValue::Int(n) => Some(*n as f64),
                        _ => None,
                    })
                    .unwrap_or(0.0);
                Ok(IfaValue::Float(val))
            }

            _ => Err(IfaError::Runtime(format!(
                "Unknown Ọ̀gbè method: {}",
                method
            ))),
        }
    }

    fn methods(&self) -> &'static [&'static str] {
        &[
            "type",
            "iru",
            "len",
            "gigun",
            "assert",
            "jẹri",
            "format",
            "ṣẹda",
            "parse_int",
            "parse_float",
        ]
    }
}
