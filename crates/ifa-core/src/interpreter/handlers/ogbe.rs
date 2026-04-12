//! # Ọ̀gbè Handler - System/Lifecycle
//!
//! Handles system operations, type introspection, and assertions.
//! Binary pattern: 1111

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{EnvRef, OduHandler};

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
        _env: &EnvRef,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        let arg0 = args.first();

        match method {
            // Type introspection
            "type" | "iru" => {
                let type_name = arg0.map(|v| v.type_name()).unwrap_or("null");
                Ok(IfaValue::str(type_name))
            }

            // Length
            "len" | "gigun" => {
                let len = arg0
                    .map(|v| match v {
                        IfaValue::Str(s) => s.len() as i64,
                        IfaValue::List(l) => l.len() as i64,
                        #[cfg(feature = "std")]
                        IfaValue::Map(m) => m.len() as i64,
                        _ => 0,
                    })
                    .unwrap_or(0);
                Ok(IfaValue::int(len))
            }

            // Assertion
            "assert" | "jẹri" => {
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

            // Format/stringify
            "format" | "ṣẹda" => {
                let output: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                Ok(IfaValue::str(output.join(" ")))
            }

            // Parse integer
            "parse_int" => {
                let val = if let Some(v) = arg0 {
                    match v {
                        IfaValue::Str(s) => s.trim().parse::<i64>().ok(),
                        IfaValue::Int(n) => Some(*n),
                        IfaValue::Float(f) => Some(*f as i64),
                        _ => None,
                    }
                } else {
                    None
                }
                .unwrap_or(0);
                Ok(IfaValue::int(val))
            }

            // Parse float
            "parse_float" => {
                let val = if let Some(v) = arg0 {
                    match v {
                        IfaValue::Str(s) => s.trim().parse::<f64>().ok(),
                        IfaValue::Float(f) => Some(*f),
                        IfaValue::Int(n) => Some(*n as f64),
                        _ => None,
                    }
                } else {
                    None
                }
                .unwrap_or(0.0);
                Ok(IfaValue::float(val))
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
