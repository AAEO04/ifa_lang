//! # Òtúúrúpọ̀n Handler - Math (Sub/Div)
//!
//! Handles subtraction, division, modulo, and related operations.
//! Binary pattern: 0010

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

/// Handler for Òtúúrúpọ̀n (Math Sub/Div) domain.
pub struct OturuponHandler;

impl OduHandler for OturuponHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Oturupon
    }

    fn call(
        &self,
        method: &str,
        args: Vec<IfaValue>,
        _env: &mut Environment,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        let arg0 = args.first();
        let arg1 = args.get(1);

        match method {
            // Subtraction
            "din" | "yọkuro" | "sub" | "subtract" => {
                if let (Some(left), Some(right)) = (arg0, arg1) {
                    match (left, right) {
                        (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::int(a - b)),
                        (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::float(a - b)),
                        (IfaValue::Int(a), IfaValue::Float(b)) => Ok(IfaValue::float(*a as f64 - b)),
                        (IfaValue::Float(a), IfaValue::Int(b)) => Ok(IfaValue::float(a - *b as f64)),
                        _ => Ok(IfaValue::int(0)),
                    }
                } else {
                    Ok(IfaValue::int(0))
                }
            },

            // Division
            "pin" | "div" | "divide" => {
                if let (Some(left), Some(right)) = (arg0, arg1) {
                    match (left, right) {
                        (IfaValue::Int(a), IfaValue::Int(b)) => {
                            if *b == 0 { return Err(IfaError::Runtime("Division by zero".into())); }
                            Ok(IfaValue::int(a / b))
                        },
                        (IfaValue::Float(a), IfaValue::Float(b)) => {
                            if *b == 0.0 { return Err(IfaError::Runtime("Division by zero".into())); }
                            Ok(IfaValue::float(a / b))
                        },
                        (IfaValue::Int(a), IfaValue::Float(b)) => {
                            if *b == 0.0 { return Err(IfaError::Runtime("Division by zero".into())); }
                            Ok(IfaValue::float(*a as f64 / b))
                        },
                        (IfaValue::Float(a), IfaValue::Int(b)) => {
                            if *b == 0 { return Err(IfaError::Runtime("Division by zero".into())); }
                            Ok(IfaValue::float(a / *b as f64))
                        },
                         // Handle errors or missing args in default
                        _ => Ok(IfaValue::int(0)),
                    }
                } else {
                     Ok(IfaValue::int(0))
                }
            },

            // Modulo
            "iyoku" | "mod" | "modulo" => {
                 if let (Some(left), Some(right)) = (arg0, arg1) {
                    match (left, right) {
                        (IfaValue::Int(a), IfaValue::Int(b)) => {
                             if *b == 0 { return Err(IfaError::Runtime("Division by zero".into())); }
                             Ok(IfaValue::int(a % b))
                        },
                        (IfaValue::Float(a), IfaValue::Float(b)) => {
                             if *b == 0.0 { return Err(IfaError::Runtime("Division by zero".into())); }
                             Ok(IfaValue::float(a % b))
                        },
                        _ => Ok(IfaValue::int(0)),
                    }
                 } else {
                     Ok(IfaValue::int(0))
                 }
            },

            // Floor division
            "floor_div" => {
                 if let (Some(left), Some(right)) = (arg0, arg1) {
                    match (left, right) {
                        (IfaValue::Int(a), IfaValue::Int(b)) => {
                             if *b == 0 { return Err(IfaError::Runtime("Division by zero".into())); }
                             Ok(IfaValue::int(a.div_euclid(*b)))
                        },
                        (IfaValue::Float(a), IfaValue::Float(b)) => {
                             if *b == 0.0 { return Err(IfaError::Runtime("Division by zero".into())); }
                             Ok(IfaValue::float((a / b).floor()))
                        },
                        _ => Ok(IfaValue::int(0)),
                    }
                 } else {
                     Ok(IfaValue::int(0))
                 }
            },

            // Negate
            "neg" | "negate" => {
                if let Some(val) = arg0 {
                    match val {
                        IfaValue::Int(n) => Ok(IfaValue::int(-n)),
                        IfaValue::Float(f) => Ok(IfaValue::float(-f)),
                         _ => Ok(IfaValue::int(0)),
                    }
                } else {
                     Ok(IfaValue::int(0))
                }
            },

            // Square root
            "sqrt" => {
                 if let Some(val) = arg0 {
                    match val {
                        IfaValue::Int(n) => Ok(IfaValue::float((*n as f64).sqrt())),
                        IfaValue::Float(f) => Ok(IfaValue::float(f.sqrt())),
                        _ => Ok(IfaValue::float(0.0)),
                    }
                 } else {
                      Ok(IfaValue::float(0.0))
                 }
            },

            _ => Err(IfaError::Runtime(format!(
                "Unknown Òtúúrúpọ̀n method: {}",
                method
            ))),
        }
    }

    fn methods(&self) -> &'static [&'static str] {
        &[
            "yọkuro",
            "sub",
            "subtract",
            "pin",
            "div",
            "divide",
            "iyoku",
            "mod",
            "modulo",
            "floor_div",
            "neg",
            "negate",
            "sqrt",
        ]
    }
}
