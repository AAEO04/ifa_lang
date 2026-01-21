//! # Ìrosù Handler - Console I/O
//!
//! Handles console input/output operations.
//! Binary pattern: 1100

use std::io::{self, Write};

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

/// Handler for Ìrosù (Console I/O) domain.
pub struct IrosuHandler;

impl OduHandler for IrosuHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Irosu
    }

    fn call(
        &self,
        method: &str,
        args: Vec<IfaValue>,
        _env: &mut Environment,
        output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        match method {
            // Print with newline
            "fo" | "sọ" | "print" | "println" => {
                let line_parts: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                let line = line_parts.join(" ");
                println!("{}", line);
                output.push(line); // Capture for WASM
                Ok(IfaValue::Null)
            }

            // Read input
            "ka" | "input" | "listen" | "gbo" => {
                print!("> ");
                io::stdout().flush().ok();
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .map_err(IfaError::IoError)?;
                Ok(IfaValue::Str(input.trim().to_string()))
            }

            // Error output
            "kigbe" | "error" => {
                let msg = args.first().map(|a| a.to_string()).unwrap_or_default();
                eprintln!("[ERROR] {}", msg);
                Ok(IfaValue::Null)
            }

            _ => Err(IfaError::Runtime(format!(
                "Unknown Ìrosù method: {}",
                method
            ))),
        }
    }

    fn methods(&self) -> &'static [&'static str] {
        &[
            "fo", "sọ", "print", "println", "ka", "input", "listen", "gbo", "kigbe", "error",
        ]
    }
}
