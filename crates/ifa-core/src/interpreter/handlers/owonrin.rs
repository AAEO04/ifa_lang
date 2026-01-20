//! # Ọ̀wọ́nrín Handler - Random
//!
//! Handles random number generation.
//! Binary pattern: 0011

use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

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
        _env: &mut Environment
    ) -> IfaResult<IfaValue> {
        match method {
            // Random integer (0-32767)
            "nọmba" | "random" | "rand" => {
                let random = self.generate_random();
                Ok(IfaValue::Int(random as i64))
            }
            
            // Random in range [min, max]
            "laarin" | "range" => {
                if args.len() >= 2 {
                    if let (IfaValue::Int(min), IfaValue::Int(max)) = (&args[0], &args[1]) {
                        let random = self.generate_random();
                        let val = min + (random as i64 % (max - min + 1));
                        return Ok(IfaValue::Int(val));
                    }
                }
                Err(IfaError::Runtime("range requires min and max integers".into()))
            }
            
            // Random float [0.0, 1.0)
            "ida" | "float" => {
                let random = self.generate_random();
                Ok(IfaValue::Float(random as f64 / 32768.0))
            }
            
            // Random boolean
            "boolean" | "bool" => {
                let random = self.generate_random();
                Ok(IfaValue::Bool(random.is_multiple_of(2)))
            }
            
            // Shuffle a list
            "aruwo" | "shuffle" => {
                if let Some(IfaValue::List(mut list)) = args.first().cloned() {
                    // Fisher-Yates shuffle
                    let len = list.len();
                    for i in (1..len).rev() {
                        let j = (self.generate_random() as usize) % (i + 1);
                        list.swap(i, j);
                    }
                    return Ok(IfaValue::List(list));
                }
                Err(IfaError::Runtime("shuffle requires a list".into()))
            }
            
            // Random choice from list
            "yan" | "choice" => {
                if let Some(IfaValue::List(list)) = args.first() {
                    if list.is_empty() {
                        return Ok(IfaValue::Null);
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
        &["nọmba", "random", "rand", "laarin", "range", "ida", "float", "boolean", "bool", "aruwo", "shuffle", "yan", "choice"]
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
