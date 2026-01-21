//! # Ọ̀sá Handler - Concurrency
//!
//! Handles parallel and concurrent operations.
//! Binary pattern: 0111
//!
//! Real parallelism is achieved by converting IfaValue to primitives internally,
//! using rayon on those, then converting back. This avoids Arc<Mutex> overhead.

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};
use rayon::prelude::*;

/// Handler for Ọ̀sá (Concurrency) domain.
pub struct OsaHandler;

impl OduHandler for OsaHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Osa
    }

    fn call(
        &self,
        method: &str,
        args: Vec<IfaValue>,
        _env: &mut Environment,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        match method {
            // Get number of available threads
            "awọn_okun" | "threads" | "num_threads" => {
                let threads = rayon::current_num_threads();
                Ok(IfaValue::Int(threads as i64))
            }

            // Parallel sum - converts to i64, uses par_iter, real parallelism
            "afikun_afiwe" | "parallel_sum" | "sum" => {
                if let Some(IfaValue::List(list)) = args.first() {
                    // Step 1: Convert IfaValue -> i64 (sequential, fast)
                    let nums: Vec<i64> = list
                        .iter()
                        .filter_map(|v| match v {
                            IfaValue::Int(n) => Some(*n),
                            IfaValue::Float(f) => Some(*f as i64),
                            _ => None,
                        })
                        .collect();

                    // Step 2: Real parallel sum on primitives
                    let sum: i64 = nums.par_iter().sum();
                    return Ok(IfaValue::Int(sum));
                }
                Err(IfaError::Runtime("sum requires a list of numbers".into()))
            }

            // Parallel product
            "isoro_afiwe" | "parallel_product" | "product" => {
                if let Some(IfaValue::List(list)) = args.first() {
                    let nums: Vec<i64> = list
                        .iter()
                        .filter_map(|v| match v {
                            IfaValue::Int(n) => Some(*n),
                            IfaValue::Float(f) => Some(*f as i64),
                            _ => None,
                        })
                        .collect();

                    let product: i64 = nums.par_iter().product();
                    return Ok(IfaValue::Int(product));
                }
                Err(IfaError::Runtime(
                    "product requires a list of numbers".into(),
                ))
            }

            // Parallel min
            "kekere_afiwe" | "parallel_min" | "min" => {
                if let Some(IfaValue::List(list)) = args.first() {
                    let nums: Vec<i64> = list
                        .iter()
                        .filter_map(|v| match v {
                            IfaValue::Int(n) => Some(*n),
                            IfaValue::Float(f) => Some(*f as i64),
                            _ => None,
                        })
                        .collect();

                    let min = nums.par_iter().min().copied();
                    return Ok(min.map(IfaValue::Int).unwrap_or(IfaValue::Null));
                }
                Err(IfaError::Runtime("min requires a list of numbers".into()))
            }

            // Parallel max
            "tobi_afiwe" | "parallel_max" | "max" => {
                if let Some(IfaValue::List(list)) = args.first() {
                    let nums: Vec<i64> = list
                        .iter()
                        .filter_map(|v| match v {
                            IfaValue::Int(n) => Some(*n),
                            IfaValue::Float(f) => Some(*f as i64),
                            _ => None,
                        })
                        .collect();

                    let max = nums.par_iter().max().copied();
                    return Ok(max.map(IfaValue::Int).unwrap_or(IfaValue::Null));
                }
                Err(IfaError::Runtime("max requires a list of numbers".into()))
            }

            // Parallel sort - real parallelism on f64 primitives
            "tọ_afiwe" | "parallel_sort" | "sort" => {
                if let Some(IfaValue::List(list)) = args.first() {
                    // Extract as (index, f64) for parallel sort
                    let mut indexed: Vec<(usize, f64)> = list
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let val = match v {
                                IfaValue::Int(n) => *n as f64,
                                IfaValue::Float(f) => *f,
                                _ => 0.0,
                            };
                            (i, val)
                        })
                        .collect();

                    // Parallel sort by value
                    indexed.par_sort_by(|a, b| {
                        a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
                    });

                    // Build result using sorted order
                    let sorted: Vec<IfaValue> =
                        indexed.iter().map(|(i, _)| list[*i].clone()).collect();

                    return Ok(IfaValue::List(sorted));
                }
                Err(IfaError::Runtime("sort requires a list".into()))
            }

            // Sequential any/all (no need for parallelism on bools)
            "eyikeyi_afiwe" | "parallel_any" | "any" => {
                if let Some(IfaValue::List(list)) = args.first() {
                    let any = list.iter().any(|v| match v {
                        IfaValue::Bool(b) => *b,
                        IfaValue::Int(n) => *n != 0,
                        IfaValue::Str(s) => !s.is_empty(),
                        IfaValue::List(l) => !l.is_empty(),
                        IfaValue::Null => false,
                        _ => true,
                    });
                    return Ok(IfaValue::Bool(any));
                }
                Err(IfaError::Runtime("any requires a list".into()))
            }

            "gbogbo_afiwe" | "parallel_all" | "all" => {
                if let Some(IfaValue::List(list)) = args.first() {
                    let all = list.iter().all(|v| match v {
                        IfaValue::Bool(b) => *b,
                        IfaValue::Int(n) => *n != 0,
                        IfaValue::Null => false,
                        _ => true,
                    });
                    return Ok(IfaValue::Bool(all));
                }
                Err(IfaError::Runtime("all requires a list".into()))
            }

            // Sleep (blocking)
            "sun" | "sleep" => {
                if let Some(IfaValue::Int(ms)) = args.first() {
                    std::thread::sleep(std::time::Duration::from_millis(*ms as u64));
                    return Ok(IfaValue::Null);
                }
                Err(IfaError::Runtime("sleep requires milliseconds".into()))
            }

            // Async operations - explain limitation
            "bẹrẹ" | "spawn" | "duro" | "await" | "fi" | "send" | "gba" | "recv" => {
                Err(IfaError::Runtime(
                    "Async operations require tokio runtime. Use ifa-std with tokio feature."
                        .into(),
                ))
            }

            _ => Err(IfaError::Runtime(format!("Unknown Ọ̀sá method: {}", method))),
        }
    }

    fn methods(&self) -> &'static [&'static str] {
        &[
            "awọn_okun",
            "threads",
            "num_threads",
            "afikun_afiwe",
            "parallel_sum",
            "sum",
            "isoro_afiwe",
            "parallel_product",
            "product",
            "kekere_afiwe",
            "parallel_min",
            "min",
            "tobi_afiwe",
            "parallel_max",
            "max",
            "tọ_afiwe",
            "parallel_sort",
            "sort",
            "eyikeyi_afiwe",
            "parallel_any",
            "any",
            "gbogbo_afiwe",
            "parallel_all",
            "all",
            "sun",
            "sleep",
            "bẹrẹ",
            "spawn",
            "duro",
            "await",
            "fi",
            "send",
            "gba",
            "recv",
        ]
    }
}
