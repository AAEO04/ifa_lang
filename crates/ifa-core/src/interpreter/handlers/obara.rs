//! # Ọ̀bàrà Handler - Math (Add/Mul)
//!
//! Handles addition, multiplication, and related math operations.
//! Binary pattern: 1000

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

/// Handler for Ọ̀bàrà (Math Add/Mul) domain.
pub struct ObaraHandler;

impl OduHandler for ObaraHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Obara
    }
    
    fn call(
        &self, 
        method: &str, 
        args: Vec<IfaValue>, 
        _env: &mut Environment
    ) -> IfaResult<IfaValue> {
        match method {
            // Addition
            "fikun" | "add" => {
                let mut sum = 0i64;
                for arg in &args {
                    match arg {
                        IfaValue::Int(n) => sum += n,
                        IfaValue::Float(_) => return Ok(self.float_sum(&args)),
                        _ => {}
                    }
                }
                Ok(IfaValue::Int(sum))
            }
            
            // Multiplication
            "isodipupo" | "mul" | "multiply" => {
                let mut product = 1i64;
                for arg in &args {
                    match arg {
                        IfaValue::Int(n) => product *= n,
                        IfaValue::Float(_) => return Ok(self.float_product(&args)),
                        _ => {}
                    }
                }
                Ok(IfaValue::Int(product))
            }
            
            // Power
            "agbara" | "pow" | "power" => {
                match (args.first(), args.get(1)) {
                    (Some(IfaValue::Int(base)), Some(IfaValue::Int(exp))) => {
                        Ok(IfaValue::Int(base.pow(*exp as u32)))
                    }
                    (Some(IfaValue::Float(base)), Some(IfaValue::Int(exp))) => {
                        Ok(IfaValue::Float(base.powi(*exp as i32)))
                    }
                    (Some(IfaValue::Float(base)), Some(IfaValue::Float(exp))) => {
                        Ok(IfaValue::Float(base.powf(*exp)))
                    }
                    _ => Ok(IfaValue::Int(0)),
                }
            }
            
            // Absolute value
            "abs" => {
                match args.first() {
                    Some(IfaValue::Int(n)) => Ok(IfaValue::Int(n.abs())),
                    Some(IfaValue::Float(f)) => Ok(IfaValue::Float(f.abs())),
                    _ => Ok(IfaValue::Int(0)),
                }
            }
            
            // Maximum
            "max" => {
                let first = args.first().cloned().unwrap_or(IfaValue::Int(0));
                let max = args.iter().fold(first, |acc, v| {
                    if v > &acc { v.clone() } else { acc }
                });
                Ok(max)
            }
            
            // Minimum  
            "min" => {
                let first = args.first().cloned().unwrap_or(IfaValue::Int(0));
                let min = args.iter().fold(first, |acc, v| {
                    if v < &acc { v.clone() } else { acc }
                });
                Ok(min)
            }
            
            _ => Err(IfaError::Runtime(format!(
                "Unknown Ọ̀bàrà method: {}",
                method
            ))),
        }
    }
    
    fn methods(&self) -> &'static [&'static str] {
        &["fikun", "add", "isodipupo", "mul", "multiply", "agbara", "pow", "power", "abs", "max", "min"]
    }
}

impl ObaraHandler {
    fn float_sum(&self, args: &[IfaValue]) -> IfaValue {
        let mut sum = 0.0f64;
        for arg in args {
            match arg {
                IfaValue::Int(n) => sum += *n as f64,
                IfaValue::Float(f) => sum += f,
                _ => {}
            }
        }
        IfaValue::Float(sum)
    }
    
    fn float_product(&self, args: &[IfaValue]) -> IfaValue {
        let mut product = 1.0f64;
        for arg in args {
            match arg {
                IfaValue::Int(n) => product *= *n as f64,
                IfaValue::Float(f) => product *= f,
                _ => {}
            }
        }
        IfaValue::Float(product)
    }
}
