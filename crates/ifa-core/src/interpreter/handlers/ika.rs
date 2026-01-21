//! # Ìká Handler - Strings
//!
//! Handles string operations.
//! Binary pattern: 0100

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

/// Handler for Ìká (Strings) domain.
pub struct IkaHandler;

impl OduHandler for IkaHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Ika
    }

    fn call(
        &self,
        method: &str,
        args: Vec<IfaValue>,
        _env: &mut Environment,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        match method {
            // Concatenate strings
            "so" | "concat" => {
                let result: String = args.iter().map(|a| a.to_string()).collect();
                Ok(IfaValue::Str(result))
            }

            // Join list with delimiter
            "dapo" | "join" => {
                if args.len() >= 2 {
                    if let (IfaValue::List(parts), IfaValue::Str(delim)) = (&args[0], &args[1]) {
                        let result: String = parts
                            .iter()
                            .map(|a| a.to_string())
                            .collect::<Vec<String>>()
                            .join(delim.as_str());
                        return Ok(IfaValue::Str(result));
                    }
                }
                Err(IfaError::Runtime("join requires list and delimiter".into()))
            }

            // String length (Unicode-aware)
            "gigun" | "len" => {
                if let Some(IfaValue::Str(s)) = args.first() {
                    Ok(IfaValue::Int(s.chars().count() as i64))
                } else {
                    Err(IfaError::Runtime("len requires a string argument".into()))
                }
            }

            // Split string
            "pin" | "split" => {
                if args.len() >= 2 {
                    if let (IfaValue::Str(s), IfaValue::Str(delim)) = (&args[0], &args[1]) {
                        let parts: Vec<IfaValue> = s
                            .split(delim.as_str())
                            .map(|p| IfaValue::Str(p.to_string()))
                            .collect();
                        return Ok(IfaValue::List(parts));
                    }
                }
                Err(IfaError::Runtime(
                    "split requires string and delimiter".into(),
                ))
            }

            // Trim whitespace
            "trim" => {
                if let Some(IfaValue::Str(s)) = args.first() {
                    Ok(IfaValue::Str(s.trim().to_string()))
                } else {
                    Err(IfaError::Runtime("trim requires a string argument".into()))
                }
            }

            // Uppercase
            "nla" | "uppercase" | "upper" => {
                if let Some(IfaValue::Str(s)) = args.first() {
                    Ok(IfaValue::Str(s.to_uppercase()))
                } else {
                    Err(IfaError::Runtime("uppercase requires a string".into()))
                }
            }

            // Lowercase
            "kekere" | "lowercase" | "lower" => {
                if let Some(IfaValue::Str(s)) = args.first() {
                    Ok(IfaValue::Str(s.to_lowercase()))
                } else {
                    Err(IfaError::Runtime("lowercase requires a string".into()))
                }
            }

            // Contains check
            "ni" | "contains" | "has" => {
                if args.len() >= 2 {
                    if let (IfaValue::Str(haystack), IfaValue::Str(needle)) = (&args[0], &args[1]) {
                        return Ok(IfaValue::Bool(haystack.contains(needle.as_str())));
                    }
                }
                Err(IfaError::Runtime("contains requires two strings".into()))
            }

            // Replace
            "ropo" | "replace" => {
                if args.len() >= 3 {
                    if let (IfaValue::Str(s), IfaValue::Str(from), IfaValue::Str(to)) =
                        (&args[0], &args[1], &args[2])
                    {
                        return Ok(IfaValue::Str(s.replace(from.as_str(), to.as_str())));
                    }
                }
                Err(IfaError::Runtime(
                    "replace requires string, from, to".into(),
                ))
            }

            // Substring
            "sub" | "substring" | "slice" => {
                if args.len() >= 2 {
                    if let (IfaValue::Str(s), IfaValue::Int(start)) = (&args[0], &args[1]) {
                        let start = *start as usize;
                        let len = args
                            .get(2)
                            .and_then(|v| {
                                if let IfaValue::Int(n) = v {
                                    Some(*n as usize)
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(s.len() - start);
                        let result: String = s.chars().skip(start).take(len).collect();
                        return Ok(IfaValue::Str(result));
                    }
                }
                Err(IfaError::Runtime(
                    "substring requires string and start index".into(),
                ))
            }

            _ => Err(IfaError::Runtime(format!("Unknown Ìká method: {}", method))),
        }
    }

    fn methods(&self) -> &'static [&'static str] {
        &[
            "so",
            "concat",
            "dapo",
            "join",
            "gigun",
            "len",
            "pin",
            "split",
            "trim",
            "nla",
            "uppercase",
            "upper",
            "kekere",
            "lowercase",
            "lower",
            "ni",
            "contains",
            "has",
            "ropo",
            "replace",
            "sub",
            "substring",
            "slice",
        ]
    }
}
