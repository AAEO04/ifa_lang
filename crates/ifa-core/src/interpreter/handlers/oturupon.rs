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
        match method {
            // Subtraction
            "din" | "yọkuro" | "sub" | "subtract" => match (args.first(), args.get(1)) {
                (Some(IfaValue::Int(a)), Some(IfaValue::Int(b))) => Ok(IfaValue::Int(a - b)),
                (Some(IfaValue::Float(a)), Some(IfaValue::Float(b))) => Ok(IfaValue::Float(a - b)),
                (Some(IfaValue::Int(a)), Some(IfaValue::Float(b))) => {
                    Ok(IfaValue::Float(*a as f64 - b))
                }
                (Some(IfaValue::Float(a)), Some(IfaValue::Int(b))) => {
                    Ok(IfaValue::Float(a - *b as f64))
                }
                _ => Ok(IfaValue::Int(0)),
            },

            // Division
            "pin" | "div" | "divide" => match (args.first(), args.get(1)) {
                (Some(IfaValue::Int(a)), Some(IfaValue::Int(b))) if *b != 0 => {
                    Ok(IfaValue::Int(a / b))
                }
                (Some(IfaValue::Float(a)), Some(IfaValue::Float(b))) if *b != 0.0 => {
                    Ok(IfaValue::Float(a / b))
                }
                (Some(IfaValue::Int(a)), Some(IfaValue::Float(b))) if *b != 0.0 => {
                    Ok(IfaValue::Float(*a as f64 / b))
                }
                (Some(IfaValue::Float(a)), Some(IfaValue::Int(b))) if *b != 0 => {
                    Ok(IfaValue::Float(a / *b as f64))
                }
                (_, Some(IfaValue::Int(0))) | (_, Some(IfaValue::Float(0.0))) => {
                    Err(IfaError::Runtime("Division by zero".into()))
                }
                _ => Ok(IfaValue::Int(0)),
            },

            // Modulo
            "iyoku" | "mod" | "modulo" => match (args.first(), args.get(1)) {
                (Some(IfaValue::Int(a)), Some(IfaValue::Int(b))) if *b != 0 => {
                    Ok(IfaValue::Int(a % b))
                }
                (Some(IfaValue::Float(a)), Some(IfaValue::Float(b))) if *b != 0.0 => {
                    Ok(IfaValue::Float(a % b))
                }
                _ => Ok(IfaValue::Int(0)),
            },

            // Floor division
            "floor_div" => match (args.first(), args.get(1)) {
                (Some(IfaValue::Int(a)), Some(IfaValue::Int(b))) if *b != 0 => {
                    Ok(IfaValue::Int(a.div_euclid(*b)))
                }
                (Some(IfaValue::Float(a)), Some(IfaValue::Float(b))) if *b != 0.0 => {
                    Ok(IfaValue::Float((a / b).floor()))
                }
                _ => Ok(IfaValue::Int(0)),
            },

            // Negate
            "neg" | "negate" => match args.first() {
                Some(IfaValue::Int(n)) => Ok(IfaValue::Int(-n)),
                Some(IfaValue::Float(f)) => Ok(IfaValue::Float(-f)),
                _ => Ok(IfaValue::Int(0)),
            },

            // Square root
            "sqrt" => match args.first() {
                Some(IfaValue::Int(n)) => Ok(IfaValue::Float((*n as f64).sqrt())),
                Some(IfaValue::Float(f)) => Ok(IfaValue::Float(f.sqrt())),
                _ => Ok(IfaValue::Float(0.0)),
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
