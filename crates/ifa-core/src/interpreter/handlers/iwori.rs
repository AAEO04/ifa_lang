//! # Ìwòrì Handler - Time/DateTime
//!
//! Handles time and date operations.
//! Binary pattern: 0110

use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

/// Handler for Ìwòrì (Time/DateTime) domain.
pub struct IworiHandler;

impl OduHandler for IworiHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Iwori
    }

    fn call(
        &self,
        method: &str,
        args: Vec<IfaValue>,
        _env: &mut Environment,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {

        let arg0 = args.get(0);
        let arg1 = args.get(1);

        match method {
            // Current Unix timestamp (seconds)
            "akoko" | "bayi" | "now" | "timestamp" => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                Ok(IfaValue::int(now as i64))
            }

            // Current Unix timestamp (milliseconds)
            "bayi_ms" | "now_ms" => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis();
                Ok(IfaValue::int(now as i64))
            }

            // Elapsed time measurement
            "aago" | "elapsed" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Int(start) = val {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as i64;
                        return Ok(IfaValue::int(now - *start));
                    }
                }
                Err(IfaError::Runtime("elapsed requires start timestamp".into()))
            }

            // Format timestamp to ISO string
            "ojo" | "ṣe_ọjọ" | "format" | "iso" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Int(ts) = val {
                        // Simple ISO-like format: just return the timestamp string
                        return Ok(IfaValue::str(format!("{}", ts)));
                    }
                }
                Err(IfaError::Runtime("format requires timestamp".into()))
            }

            // Parse date string to timestamp
            "ka_ọjọ" | "parse" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Str(s) = val {
                         // Simple parsing - just try to parse as integer
                         let ts = s.parse::<i64>().unwrap_or(0);
                         return Ok(IfaValue::int(ts));
                    }
                }
                Err(IfaError::Runtime("parse requires date string".into()))
            }

            // Create range for iteration
            "laarin" | "range" => {
                if let (Some(start_val), Some(end_val)) = (arg0, arg1) {
                    if let (IfaValue::Int(start), IfaValue::Int(end)) = (start_val, end_val) {
                        let list: Vec<IfaValue> = (*start..*end).map(IfaValue::int).collect();
                        return Ok(IfaValue::list(list));
                    }
                }
                Err(IfaError::Runtime(
                    "range requires start and end integers".into(),
                ))
            }

            _ => Err(IfaError::Runtime(format!(
                "Unknown Ìwòrì method: {}",
                method
            ))),
        }
    }

    fn methods(&self) -> &'static [&'static str] {
        &[
            "bayi",
            "now",
            "timestamp",
            "bayi_ms",
            "now_ms",
            "aago",
            "elapsed",
            "ṣe_ọjọ",
            "format",
            "iso",
            "ka_ọjọ",
            "parse",
            "laarin",
            "range",
        ]
    }
}
