//! # Ọ̀bàrà Handler - Math (Add/Mul)
//!
//! Handles addition, multiplication, and related math operations.
//! Binary pattern: 1000

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{EnvRef, OduHandler};

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
        _env: &EnvRef,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        let arg0 = args.get(0);
        let arg1 = args.get(1);

        match method {
            // Addition
            "fikun" | "add" => {
                let mut sum = 0i64;
                for arg in &args {
                    match arg {
                        IfaValue::Int(n) => sum += *n,
                        IfaValue::Float(_) => return Ok(self.float_sum(&args)),
                        _ => {}
                    }
                }
                Ok(IfaValue::int(sum))
            }

            // Multiplication
            "isodipupo" | "mul" | "multiply" => {
                let mut product = 1i64;
                for arg in &args {
                    match arg {
                        IfaValue::Int(n) => product *= *n,
                        IfaValue::Float(_) => return Ok(self.float_product(&args)),
                        _ => {}
                    }
                }
                Ok(IfaValue::int(product))
            }

            // Power
            "agbara" | "pow" | "power" => {
                if let (Some(base_val), Some(exp_val)) = (arg0, arg1) {
                    match (base_val, exp_val) {
                        (IfaValue::Int(base), IfaValue::Int(exp)) => {
                            Ok(IfaValue::int(base.pow(*exp as u32)))
                        }
                        (IfaValue::Float(base), IfaValue::Int(exp)) => {
                            Ok(IfaValue::float(base.powi(*exp as i32)))
                        }
                        (IfaValue::Float(base), IfaValue::Float(exp)) => {
                            Ok(IfaValue::float(base.powf(*exp)))
                        }
                        _ => Ok(IfaValue::int(0)),
                    }
                } else {
                    Ok(IfaValue::int(0))
                }
            }

            // Absolute value
            "abs" => {
                if let Some(val) = arg0 {
                    match val {
                        IfaValue::Int(n) => Ok(IfaValue::int(n.abs())),
                        IfaValue::Float(f) => Ok(IfaValue::float(f.abs())),
                        _ => Ok(IfaValue::int(0)),
                    }
                } else {
                    Ok(IfaValue::int(0))
                }
            }

            // Maximum
            "max" => {
                let first = args.first().cloned().unwrap_or(IfaValue::int(0));
                let max = args.iter().fold(first, |acc, v| match (&acc, v) {
                    (IfaValue::Int(a), IfaValue::Int(b)) if *b > *a => v.clone(),
                    (IfaValue::Float(a), IfaValue::Float(b)) if *b > *a => v.clone(),
                    _ => acc,
                });
                Ok(max)
            }

            // Minimum
            "min" => {
                let first = args.first().cloned().unwrap_or(IfaValue::int(0));
                let min = args.iter().fold(first, |acc, v| match (&acc, v) {
                    (IfaValue::Int(a), IfaValue::Int(b)) if b < a => v.clone(),
                    (IfaValue::Float(a), IfaValue::Float(b)) if b < a => v.clone(),
                    _ => acc,
                });
                Ok(min)
            }

            _ => Err(IfaError::Runtime(format!(
                "Unknown Ọ̀bàrà method: {}",
                method
            ))),
        }
    }

    // ... skipping methods ...

    fn methods(&self) -> &'static [&'static str] {
        &[
            "fikun",
            "add",
            "isodipupo",
            "mul",
            "multiply",
            "agbara",
            "pow",
            "power",
            "abs",
            "max",
            "min",
        ]
    }
}

impl ObaraHandler {
    fn float_sum(&self, args: &[IfaValue]) -> IfaValue {
        let mut sum = 0.0f64;
        for arg in args {
            match arg {
                IfaValue::Int(n) => sum += *n as f64,
                IfaValue::Float(f) => sum += *f,
                _ => {}
            }
        }
        IfaValue::float(sum)
    }

    fn float_product(&self, args: &[IfaValue]) -> IfaValue {
        let mut product = 1.0f64;
        for arg in args {
            match arg {
                IfaValue::Int(n) => product *= *n as f64,
                IfaValue::Float(f) => product *= *f,
                _ => {}
            }
        }
        IfaValue::float(product)
    }
}
