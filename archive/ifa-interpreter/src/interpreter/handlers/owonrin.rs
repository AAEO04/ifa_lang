//! # Ọ̀wọ́nrín Handler - Random
//!
//! Handles random number generation. Delegates to ifa-std::odu::owonrin::Owonrin.
//! Binary pattern: 0011

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;
use super::{EnvRef, OduHandler};

/// Handler for Ọ̀wọ́nrín (Random) domain.
pub struct OwonrinHandler;

impl OduHandler for OwonrinHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Owonrin
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
            let mut owonrin = ifa_std::odu::owonrin::Owonrin::new(capabilities.clone());
            let arg0 = args.first();
            let arg1 = args.get(1);

            match method {
                // Random integer (0-32767)
                "nọmba" | "random" | "rand" => {
                    let val = owonrin.pese(0, 32767);
                    Ok(IfaValue::int(val))
                }

                // Random in range [min, max]
                "pese" | "laarin" | "range" => {
                    if let (Some(IfaValue::Int(min)), Some(IfaValue::Int(max))) = (arg0, arg1) {
                        let val = owonrin.pese(*min, *max);
                        Ok(IfaValue::int(val))
                    } else {
                        Err(IfaError::Runtime("range requires min and max integers".into()))
                    }
                }

                // Random float [0.0, 1.0)
                "ida" | "float" => {
                    let val = owonrin.pese_odidi();
                    Ok(IfaValue::float(val))
                }

                // Random boolean
                "boolean" | "bool" => {
                    let val = owonrin.boya(0.5);
                    Ok(IfaValue::bool(val))
                }

                // Shuffle a list
                "aruwo" | "shuffle" => {
                    if let Some(IfaValue::List(l)) = arg0 {
                        let mut list = (**l).clone();
                        owonrin.dapo(&mut list);
                        Ok(IfaValue::list(list))
                    } else {
                        Err(IfaError::Runtime("shuffle requires a list".into()))
                    }
                }

                // Random choice from list
                "yan" | "choice" => {
                    if let Some(IfaValue::List(list)) = arg0 {
                        if list.is_empty() {
                            return Ok(IfaValue::null());
                        }
                        if let Some(choice) = owonrin.yan(list) {
                            Ok(choice.clone())
                        } else {
                            Ok(IfaValue::null())
                        }
                    } else {
                        Err(IfaError::Runtime("choice requires a non-empty list".into()))
                    }
                }

                _ => Err(IfaError::Runtime(format!("Unknown Ọ̀wọ́nrín method: {}", method))),
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
            "nọmba", "random", "rand", "laarin", "range", "ida", "float", "boolean", "bool",
            "aruwo", "shuffle", "yan", "choice",
        ]
    }
}
