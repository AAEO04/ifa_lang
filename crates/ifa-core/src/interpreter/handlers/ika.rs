//! # Ìká Handler - Strings
//!
//! Handles string operations.
//! Binary pattern: 0100

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{EnvRef, OduHandler};

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
        _env: &EnvRef,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        let arg0 = args.get(0);

        match method {
            // Concatenate strings
            "so" | "concat" => {
                let result: String = args.iter().map(|a| a.to_string()).collect();
                Ok(IfaValue::str(result))
            }

            // Join list with delimiter
            "dapo" | "join" => {
                if let (Some(list_val), Some(delim_val)) = (arg0, args.get(1)) {
                    if let (IfaValue::List(parts), IfaValue::Str(delim)) = (list_val, delim_val) {
                        let result: String = parts
                            .iter()
                            .map(|a| a.to_string())
                            .collect::<Vec<String>>()
                            .join(delim);
                        return Ok(IfaValue::str(result));
                    }
                }
                Err(IfaError::Runtime("join requires list and delimiter".into()))
            }

            // String length (Unicode-aware)
            "gigun" | "len" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Str(s) = val {
                        Ok(IfaValue::int(s.chars().count() as i64))
                    } else {
                        Err(IfaError::Runtime("len requires a string argument".into()))
                    }
                } else {
                    Err(IfaError::Runtime("len requires a string argument".into()))
                }
            }

            // Split string
            "pin" | "split" => {
                if let (Some(str_val), Some(delim_val)) = (arg0, args.get(1)) {
                    if let (IfaValue::Str(s), IfaValue::Str(delim)) = (str_val, delim_val) {
                        let parts: Vec<IfaValue> =
                            s.split(delim.as_ref()).map(IfaValue::str).collect();
                        return Ok(IfaValue::list(parts));
                    }
                }
                Err(IfaError::Runtime(
                    "split requires string and delimiter".into(),
                ))
            }

            // Trim whitespace
            "trim" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Str(s) = val {
                        Ok(IfaValue::str(s.trim()))
                    } else {
                        Err(IfaError::Runtime("trim requires a string argument".into()))
                    }
                } else {
                    Err(IfaError::Runtime("trim requires a string argument".into()))
                }
            }

            // Uppercase
            "nla" | "uppercase" | "upper" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Str(s) = val {
                        Ok(IfaValue::str(s.to_uppercase()))
                    } else {
                        Err(IfaError::Runtime("uppercase requires a string".into()))
                    }
                } else {
                    Err(IfaError::Runtime("uppercase requires a string".into()))
                }
            }

            // Lowercase
            "kekere" | "lowercase" | "lower" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Str(s) = val {
                        Ok(IfaValue::str(s.to_lowercase()))
                    } else {
                        Err(IfaError::Runtime("lowercase requires a string".into()))
                    }
                } else {
                    Err(IfaError::Runtime("lowercase requires a string".into()))
                }
            }

            // Contains check
            "ni" | "contains" | "has" => {
                if let (Some(haystack_val), Some(needle_val)) = (arg0, args.get(1)) {
                    if let (IfaValue::Str(haystack), IfaValue::Str(needle)) =
                        (haystack_val, needle_val)
                    {
                        return Ok(IfaValue::bool(haystack.contains(&**needle)));
                    }
                }
                Err(IfaError::Runtime("contains requires two strings".into()))
            }

            // Replace
            "ropo" | "replace" => {
                if let (Some(s_val), Some(from_val), Some(to_val)) =
                    (arg0, args.get(1), args.get(2))
                {
                    if let (IfaValue::Str(s), IfaValue::Str(from), IfaValue::Str(to)) =
                        (s_val, from_val, to_val)
                    {
                        return Ok(IfaValue::str(s.replace(&**from, &**to)));
                    }
                }
                Err(IfaError::Runtime(
                    "replace requires string, from, to".into(),
                ))
            }

            // Substring
            "sub" | "substring" | "slice" => {
                if let (Some(s_val), Some(start_val)) = (arg0, args.get(1)) {
                    if let (IfaValue::Str(s), IfaValue::Int(start)) = (s_val, start_val) {
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
                            .unwrap_or_else(|| s.len().saturating_sub(start));

                        // Safe substring via chars
                        let result: String = s.chars().skip(start).take(len).collect();
                        return Ok(IfaValue::str(result));
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
