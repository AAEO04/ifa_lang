//! # Ọ̀wọ́nrín Handler - Random
//!
//! Handles random number generation.
//! Binary pattern: 0011

use std::time::{SystemTime, UNIX_EPOCH};

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
    ) -> IfaResult<IfaValue> {
        let arg0 = args.first();
        let arg1 = args.get(1);

        match method {
            // Random integer (0-32767)
            "nọmba" | "random" | "rand" => {
                let random = self.generate_random();
                Ok(IfaValue::int(random as i64))
            }

            // Random in range [min, max]
            "pese" | "laarin" | "range" => {
                if let (Some(min_val), Some(max_val)) = (arg0, arg1) {
                    if let (IfaValue::Int(min), IfaValue::Int(max)) = (min_val, max_val) {
                        let random = self.generate_random();
                        let val = *min + (random as i64 % (*max - *min + 1));
                        return Ok(IfaValue::int(val));
                    }
                }
                Err(IfaError::Runtime(
                    "range requires min and max integers".into(),
                ))
            }

            // Random float [0.0, 1.0)
            "ida" | "float" => {
                let random = self.generate_random();
                Ok(IfaValue::float(random as f64 / 32768.0))
            }

            // Random boolean
            "boolean" | "bool" => {
                let random = self.generate_random();
                Ok(IfaValue::bool(random.is_multiple_of(2)))
            }

            // Shuffle a list
            "aruwo" | "shuffle" => {
                if let Some(IfaValue::List(l)) = arg0 {
                    let mut list = (**l).clone();
                    // Fisher-Yates shuffle
                    let len = list.len();
                    if len > 1 {
                        for i in (1..len).rev() {
                            let j = (self.generate_random() as usize) % (i + 1);
                            list.swap(i, j);
                        }
                    }
                    return Ok(IfaValue::list(list));
                }
                Err(IfaError::Runtime("shuffle requires a list".into()))
            }

            // Random choice from list
            "yan" | "choice" => {
                if let Some(IfaValue::List(list)) = arg0 {
                    if list.is_empty() {
                        return Ok(IfaValue::null());
                    }
                    let idx = (self.generate_random() as usize) % list.len();
                    return Ok(list[idx].clone());
                }
                Err(IfaError::Runtime("choice requires a non-empty list".into()))
            }

            _ => Err(IfaError::Runtime(format!(
                "Unknown Ọ̀wọ́nrín method: {}",
                method
            ))),
        }
    }

    fn methods(&self) -> &'static [&'static str] {
        &[
            "nọmba", "random", "rand", "laarin", "range", "ida", "float", "boolean", "bool",
            "aruwo", "shuffle", "yan", "choice",
        ]
    }
}

impl OwonrinHandler {
    /// Simple LCG-based random number generator
    fn generate_random(&self) -> u64 {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        (seed.wrapping_mul(1103515245).wrapping_add(12345) >> 16) & 0x7fff
    }
}
