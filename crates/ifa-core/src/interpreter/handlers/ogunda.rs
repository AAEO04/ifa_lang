//! # Ògúndá Handler - Arrays/Lists
//!
//! Handles array/list operations.
//! Binary pattern: 1110

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{EnvRef, OduHandler};

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
        _env: &EnvRef,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        let arg0 = args.first();

        match method {
            // Create new list
            "da" | "create" | "new" => Ok(IfaValue::list(args)),

            // List length
            "iwon" | "gigun" | "len" => {
                if let Some(IfaValue::List(list)) = arg0 {
                    Ok(IfaValue::int(list.len() as i64))
                } else {
                    Err(IfaError::Runtime("len requires a list argument".into()))
                }
            }

            // Push element
            "fi" | "fikun" | "push" | "append" => {
                if let (Some(list_val), Some(elem)) = (arg0, args.get(1)) {
                    if let IfaValue::List(l) = list_val {
                        let mut list = (**l).clone();
                        list.push(elem.clone());
                        Ok(IfaValue::list(list))
                    } else {
                        Err(IfaError::Runtime("push requires a list".into()))
                    }
                } else {
                    Err(IfaError::Runtime("push requires list and element".into()))
                }
            }

            // Pop element
            "mu" | "yọ" | "pop" => {
                if let Some(IfaValue::List(l)) = arg0 {
                    let mut list = (**l).clone();
                    let popped = list.pop().unwrap_or(IfaValue::null());
                    // NOTE: Ideally we modify in place, but here we return the popped value?
                    // The original code returned the popped value but didn't mutate the original list effectively (pass by value).
                    // This is a known issue in the interpreter design (it passes copies).
                    // We will replicate legacy behavior: return popped value.
                    Ok(popped)
                } else {
                    Err(IfaError::Runtime("pop requires a list".into()))
                }
            }

            // Reverse list
            "pada" | "yipada" | "reverse" => {
                if let Some(IfaValue::List(l)) = arg0 {
                    let mut list = (**l).clone();
                    list.reverse();
                    Ok(IfaValue::list(list))
                } else {
                    Err(IfaError::Runtime("reverse requires a list".into()))
                }
            }

            // First element
            "akọkọ" | "first" => {
                if let Some(IfaValue::List(list)) = arg0 {
                    Ok(list.first().cloned().unwrap_or(IfaValue::null()))
                } else {
                    Err(IfaError::Runtime("first requires a list".into()))
                }
            }

            // Last element
            "ikẹhin" | "last" => {
                if let Some(IfaValue::List(list)) = arg0 {
                    Ok(list.last().cloned().unwrap_or(IfaValue::null()))
                } else {
                    Err(IfaError::Runtime("last requires a list".into()))
                }
            }

            // Get element at index
            "gba" | "get" => {
                if let (Some(list_val), Some(idx_val)) = (arg0, args.get(1)) {
                    if let (IfaValue::List(list), IfaValue::Int(idx)) = (list_val, idx_val) {
                        Ok(list.get(*idx as usize).cloned().unwrap_or(IfaValue::null()))
                    } else {
                        Err(IfaError::Runtime("get requires list and index".into()))
                    }
                } else {
                    Err(IfaError::Runtime("get requires list and index".into()))
                }
            }

            // Contains element
            "ni" | "contains" => {
                if let (Some(list_val), Some(elem)) = (arg0, args.get(1)) {
                    if let IfaValue::List(list) = list_val {
                        Ok(IfaValue::bool(list.contains(elem)))
                    } else {
                        Err(IfaError::Runtime("contains requires a list".into()))
                    }
                } else {
                    Err(IfaError::Runtime(
                        "contains requires list and element".into(),
                    ))
                }
            }

            // Slice list
            "ge" | "slice" => {
                if let (Some(list_val), Some(start_val)) = (arg0, args.get(1)) {
                    if let (IfaValue::List(list), IfaValue::Int(start)) = (list_val, start_val) {
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

                        let sliced: Vec<IfaValue> = list
                            .iter()
                            .skip(start)
                            .take(end.saturating_sub(start))
                            .cloned()
                            .collect();
                        Ok(IfaValue::list(sliced))
                    } else {
                        Err(IfaError::Runtime(
                            "slice requires list and start index".into(),
                        ))
                    }
                } else {
                    Err(IfaError::Runtime(
                        "slice requires list and start index".into(),
                    ))
                }
            }

            // Map function over list (simplified)
            "maapu" | "map" => {
                if let Some(IfaValue::List(list)) = arg0 {
                    Ok(IfaValue::list(list.as_ref().clone()))
                } else {
                    Err(IfaError::Runtime("map requires a list".into()))
                }
            }

            // Filter list (simplified)
            "ṣàjọ" | "filter" => {
                if let Some(IfaValue::List(list)) = arg0 {
                    Ok(IfaValue::list(list.as_ref().clone()))
                } else {
                    Err(IfaError::Runtime("filter requires a list".into()))
                }
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
