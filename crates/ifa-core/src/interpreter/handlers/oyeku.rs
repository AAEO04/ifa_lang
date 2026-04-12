//! # Ọ̀yẹ̀kú Handler - Exit/Sleep
//!
//! Handles program exit and sleep operations.
//! Binary pattern: 0000

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{EnvRef, OduHandler};

/// Handler for Ọ̀yẹ̀kú (Exit/Sleep) domain.
pub struct OyekuHandler;

impl OduHandler for OyekuHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Oyeku
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
            // Exit program
            "ku" | "jade" | "exit" => {
                let code = if let Some(IfaValue::Int(n)) = arg0 {
                    *n as i32
                } else {
                    0
                };
                // Remove direct host abort: std::process::exit(code);
                _output.push(format!("[exit] requested with code {}", code));
                // Signal exit via error in embedded AST mode
                Err(IfaError::Runtime(format!(
                    "Process exit requested with code {}",
                    code
                )))
            }

            // Wait/sleep (milliseconds)
            "duro" | "sun" | "sleep" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Int(ms) = val {
                        // Remove direct host block: std::thread::sleep(...);
                        _output.push(format!("[sleep] requested {} ms", ms));
                    }
                }
                Ok(IfaValue::null())
            }

            // Sleep for seconds
            "sun_sẹkọndi" | "sleep_sec" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Int(sec) = val {
                        // Remove direct host block: std::thread::sleep(...);
                        _output.push(format!("[sleep] requested {} sec", sec));
                    }
                }
                Ok(IfaValue::null())
            }

            _ => Err(IfaError::Runtime(format!(
                "Unknown Ọ̀yẹ̀kú method: {}",
                method
            ))),
        }
    }

    fn methods(&self) -> &'static [&'static str] {
        &["jade", "exit", "sun", "sleep", "sun_sẹkọndi", "sleep_sec"]
    }
}
