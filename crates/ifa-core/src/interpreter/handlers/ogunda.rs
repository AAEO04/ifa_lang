//! # Ògúndá Handler - Arrays/Lists
//!
//! Handles array/list operations.
//! Binary pattern: 1110

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

/// Handler for Ògúndá (Arrays/Lists) domain.
pub struct OgundaHandler;

impl OduHandler for OgundaHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Ogunda
    }

    fn call(
        &self,
        method: &str,
        args: Vec<IfaValue>,
        _env: &mut Environment,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        match method {
            // Create new list
            "da" | "create" | "new" => Ok(IfaValue::List(args)),

            // List length
            "iwon" | "gigun" | "len" => {
                if let Some(IfaValue::List(list)) = args.first() {
                    Ok(IfaValue::Int(list.len() as i64))
                } else {
                    Err(IfaError::Runtime("len requires a list argument".into()))
                }
            }

            // Push element
            "fi" | "fikun" | "push" | "append" => {
                if args.len() >= 2 {
                    if let IfaValue::List(mut list) = args[0].clone() {
                        list.push(args[1].clone());
                        return Ok(IfaValue::List(list));
                    }
                }
                Err(IfaError::Runtime("push requires list and element".into()))
            }

            // Pop element
            "mu" | "yọ" | "pop" => {
                if let Some(IfaValue::List(mut list)) = args.first().cloned() {
                    let val = list.pop().unwrap_or(IfaValue::Null);
                    return Ok(val);
                }
                Err(IfaError::Runtime("pop requires a list".into()))
            }

            // Reverse list
            "pada" | "yipada" | "reverse" => {
                if let Some(IfaValue::List(mut list)) = args.first().cloned() {
                    list.reverse();
                    return Ok(IfaValue::List(list));
                }
                Err(IfaError::Runtime("reverse requires a list".into()))
            }

            // First element
            "akọkọ" | "first" => {
                if let Some(IfaValue::List(list)) = args.first() {
                    return Ok(list.first().cloned().unwrap_or(IfaValue::Null));
                }
                Err(IfaError::Runtime("first requires a list".into()))
            }

            // Last element
            "ikẹhin" | "last" => {
                if let Some(IfaValue::List(list)) = args.first() {
                    return Ok(list.last().cloned().unwrap_or(IfaValue::Null));
                }
                Err(IfaError::Runtime("last requires a list".into()))
            }

            // Get element at index
            "gba" | "get" => {
                if args.len() >= 2 {
                    if let (IfaValue::List(list), IfaValue::Int(idx)) = (&args[0], &args[1]) {
                        let idx = *idx as usize;
                        return Ok(list.get(idx).cloned().unwrap_or(IfaValue::Null));
                    }
                }
                Err(IfaError::Runtime("get requires list and index".into()))
            }

            // Contains element
            "ni" | "contains" => {
                if args.len() >= 2 {
                    if let IfaValue::List(list) = &args[0] {
                        return Ok(IfaValue::Bool(list.contains(&args[1])));
                    }
                }
                Err(IfaError::Runtime(
                    "contains requires list and element".into(),
                ))
            }

            // Slice list
            "ge" | "slice" => {
                if args.len() >= 2 {
                    if let (IfaValue::List(list), IfaValue::Int(start)) = (&args[0], &args[1]) {
                        let start = *start as usize;
                        let end = args
                            .get(2)
                            .and_then(|v| {
                                if let IfaValue::Int(n) = v {
                                    Some(*n as usize)
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(list.len());
                        let sliced: Vec<IfaValue> =
                            list.iter().skip(start).take(end - start).cloned().collect();
                        return Ok(IfaValue::List(sliced));
                    }
                }
                Err(IfaError::Runtime(
                    "slice requires list and start index".into(),
                ))
            }

            // Map function over list (simplified)
            "maapu" | "map" => {
                // Note: Full map implementation requires closure support
                if let Some(IfaValue::List(list)) = args.first() {
                    return Ok(IfaValue::List(list.clone()));
                }
                Err(IfaError::Runtime("map requires a list".into()))
            }

            // Filter list (simplified)
            "ṣàjọ" | "filter" => {
                // Note: Full filter implementation requires closure support
                if let Some(IfaValue::List(list)) = args.first() {
                    return Ok(IfaValue::List(list.clone()));
                }
                Err(IfaError::Runtime("filter requires a list".into()))
            }

            _ => Err(IfaError::Runtime(format!(
                "Unknown Ògúndá method: {}",
                method
            ))),
        }
    }

    fn methods(&self) -> &'static [&'static str] {
        &[
            "da",
            "create",
            "new",
            "gigun",
            "len",
            "fikun",
            "push",
            "append",
            "yọ",
            "pop",
            "yipada",
            "reverse",
            "akọkọ",
            "first",
            "ikẹhin",
            "last",
            "gba",
            "get",
            "ni",
            "contains",
            "ge",
            "slice",
            "maapu",
            "map",
            "ṣàjọ",
            "filter",
        ]
    }
}
