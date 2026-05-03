//! # Ìrosù Handler - Console I/O
//!
//! Handles console input/output operations.
//! Binary pattern: 1100


use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{EnvRef, OduHandler};

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
        _env: &EnvRef,
        output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        match method {
            "fo" | "sọ" | "so" | "print" | "println" => {
                let line_parts: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                let line = line_parts.join(" ");
                
                // Native host output
                #[cfg(feature = "native")]
                {
                    if method == "fo" || method == "println" {
                        println!("{}", line);
                    } else {
                        print!("{}", line);
                        use std::io::Write;
                        std::io::stdout().flush().ok();
                    }
                }

                output.push(line);
                Ok(IfaValue::null())
            }

            // Read input
            "ka" | "input" | "listen" | "gbo" => {
                #[cfg(feature = "native")]
                {
                    use std::io::{self, Write};
                    print!("> ");
                    io::stdout().flush().ok();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).ok();
                    return Ok(IfaValue::str(input.trim()));
                }

                #[cfg(not(feature = "native"))]
                {
                    output.push("[input] requested".into());
                    Ok(IfaValue::str(""))
                }
            }

            // Error output
            "kigbe" | "error" => {
                let msg = args.first().map(|a| a.to_string()).unwrap_or_default();
                
                #[cfg(feature = "native")]
                {
                    eprintln!("[ERROR] {}", msg);
                }

                output.push(format!("[ERROR] {}", msg));
                Ok(IfaValue::null())
            }

            _ => Err(IfaError::Runtime(format!(
                "Unknown Ìrosù method: {}",
                method
            ))),
        }
    }

    fn methods(&self) -> &'static [&'static str] {
        &[
            "fo", "sọ", "so", "print", "println", "ka", "input", "listen", "gbo", "kigbe", "error",
        ]
    }
}
