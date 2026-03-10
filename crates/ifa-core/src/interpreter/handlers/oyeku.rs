//! # Ọ̀yẹ̀kú Handler - Exit/Sleep
//!
//! Handles program exit and sleep operations.
//! Binary pattern: 0000

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

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
        _env: &mut Environment,
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
                std::process::exit(code);
            }

            // Wait/sleep (milliseconds)
            "duro" | "sun" | "sleep" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Int(ms) = val {
                        std::thread::sleep(std::time::Duration::from_millis(*ms as u64));
                    }
                }
                Ok(IfaValue::null())
            }

            // Sleep for seconds
            "sun_sẹkọndi" | "sleep_sec" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Int(sec) = val {
                        std::thread::sleep(std::time::Duration::from_secs(*sec as u64));
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
