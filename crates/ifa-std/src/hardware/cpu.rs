//! CPU Domain (Domain 18) wrapper
//! Implements Zero-Copy CpuOponView memory abstraction.

use ifa_infra::cpu::CpuContext;
use ifa_types::value_union::IfaValue;
use ifa_types::{IfaError, IfaResult};
use std::sync::{Arc, RwLock};

/// A hardware-aware zero-copy buffer for CPU computing.
/// Registered in the VM's ResourceRegistry as `IfaValue::Resource(token)`.
pub struct CpuOponView {
    pub buffer: RwLock<Vec<f32>>,
    pub size: usize,
}

pub fn dispatch(
    method: &str,
    args: Vec<IfaValue>,
    ctx: &mut ifa_vm::native::VmContext,
) -> IfaResult<IfaValue> {
    match method {
        "configure" => handle_configure(args),
        "positive" => handle_iterator_method("positive", args),
        "negative" => handle_iterator_method("negative", args),
        "nonzero" => handle_iterator_method("nonzero", args),
        "even" => handle_iterator_method("even", args),
        "odd" => handle_iterator_method("odd", args),
        "sum" => handle_iterator_method("sum", args),
        "product" | "prod" => handle_iterator_method("product", args),
        "min" => handle_iterator_method("min", args),
        "max" => handle_iterator_method("max", args),
        "par_sum" => handle_iterator_method("par_sum", args),
        "par_filter" | "filter" => handle_iterator_method("par_filter", args),
        "par_sort" | "sort" => handle_iterator_method("par_sort", args),

        "threads" | "num_threads" => Ok(IfaValue::Int(CpuContext::num_threads() as i64)),
        "alloc_buffer" => handle_alloc_buffer(args, ctx),
        "read_buffer" => handle_read_buffer(args, ctx),
        "write_buffer" => handle_write_buffer(args, ctx),
        "par_map" | "map" => handle_par_map(args, ctx),
        "par_reduce" | "reduce" => handle_par_reduce(args, ctx),
        _ => Err(IfaError::Custom(format!(
            "Cpu: unknown method '{}'",
            method
        ))),
    }
}

fn handle_configure(args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    let threads = match args.first() {
        Some(IfaValue::Int(i)) if *i > 0 => *i as usize,
        Some(IfaValue::Float(f)) if *f > 0.0 => *f as usize,
        Some(other) => {
            return Err(IfaError::TypeError {
                expected: "positive Int or Float".into(),
                got: other.type_name().into(),
            });
        }
        None => {
            return Err(IfaError::ArgumentError(
                "Cpu.configure expects a thread count".into(),
            ));
        }
    };
    CpuContext::configure(threads).map_err(IfaError::Runtime)?;
    Ok(IfaValue::null())
}

fn handle_alloc_buffer(
    args: Vec<IfaValue>,
    ctx: &mut ifa_vm::native::VmContext,
) -> IfaResult<IfaValue> {
    let size = match args.first() {
        Some(IfaValue::Int(i)) if *i > 0 => *i as usize,
        _ => {
            return Err(IfaError::ArgumentError(
                "cpu.alloc_buffer: arg must be size (int)".into(),
            ));
        }
    };

    let view = CpuOponView {
        buffer: RwLock::new(vec![0.0; size]),
        size,
    };

    let token = ctx.resource_registry().register(view);
    Ok(IfaValue::Resource(Arc::new(token)))
}

fn handle_read_buffer(
    args: Vec<IfaValue>,
    ctx: &mut ifa_vm::native::VmContext,
) -> IfaResult<IfaValue> {
    let view_token = match args.first() {
        Some(IfaValue::Resource(r)) => **r,
        _ => {
            return Err(IfaError::ArgumentError(
                "cpu.read_buffer: first arg must be CpuOponView".into(),
            ));
        }
    };

    let registry = ctx.resource_registry();
    let view = registry
        .get::<CpuOponView>(view_token)
        .ok_or_else(|| IfaError::Runtime("CpuOponView handle not found".into()))?;

    let buffer = view
        .buffer
        .read()
        .map_err(|_| IfaError::Runtime("CpuOponView read lock poisoned".into()))?;
    let mut list = Vec::with_capacity(view.size);
    for &val in buffer.iter() {
        list.push(IfaValue::Float(val as f64));
    }

    Ok(IfaValue::List(ifa_types::gc::IfaGc::new(list)))
}

fn handle_write_buffer(
    args: Vec<IfaValue>,
    ctx: &mut ifa_vm::native::VmContext,
) -> IfaResult<IfaValue> {
    let view_token = match args.first() {
        Some(IfaValue::Resource(r)) => **r,
        _ => {
            return Err(IfaError::ArgumentError(
                "cpu.write_buffer: first arg must be CpuOponView".into(),
            ));
        }
    };
    let list_val = match args.get(1) {
        Some(IfaValue::List(l)) => l,
        _ => {
            return Err(IfaError::ArgumentError(
                "cpu.write_buffer: second arg must be a List of numbers".into(),
            ));
        }
    };

    let registry = ctx.resource_registry();
    let view = registry
        .get::<CpuOponView>(view_token)
        .ok_or_else(|| IfaError::Runtime("CpuOponView handle not found".into()))?;

    if list_val.len() != view.size {
        return Err(IfaError::Runtime(format!(
            "List size {} does not match CpuOponView size {}",
            list_val.len(),
            view.size
        )));
    }

    let mut buffer = view
        .buffer
        .write()
        .map_err(|_| IfaError::Runtime("CpuOponView write lock poisoned".into()))?;
    for (i, item) in list_val.iter().enumerate() {
        match item {
            IfaValue::Float(f) => buffer[i] = *f as f32,
            IfaValue::Int(n) => buffer[i] = *n as f32,
            _ => return Err(IfaError::Runtime("List must contain only numbers".into())),
        }
    }

    Ok(IfaValue::null())
}

fn handle_par_map(args: Vec<IfaValue>, ctx: &mut ifa_vm::native::VmContext) -> IfaResult<IfaValue> {
    let view_token = match args.first() {
        Some(IfaValue::Resource(r)) => **r,
        _ => {
            return Err(IfaError::ArgumentError(
                "cpu.par_map: first arg must be CpuOponView".into(),
            ));
        }
    };
    let op = match args.get(1) {
        Some(IfaValue::Str(s)) => s.to_string(),
        _ => {
            return Err(IfaError::ArgumentError(
                "cpu.par_map: second arg must be operation string".into(),
            ));
        }
    };

    let registry = ctx.resource_registry();
    let view = registry
        .get::<CpuOponView>(view_token)
        .ok_or_else(|| IfaError::Runtime("CpuOponView handle not found".into()))?;

    // Pre-validate operation so we don't panic inside Rayon loop
    map_numeric_op(0.0, &op)?;

    let buffer = view
        .buffer
        .read()
        .map_err(|_| IfaError::Runtime("CpuOponView read lock poisoned".into()))?;
    let mapped = CpuContext::par_map(&buffer, |x| map_numeric_op(*x, &op).unwrap_or(0.0));

    let new_view = CpuOponView {
        buffer: RwLock::new(mapped),
        size: view.size,
    };

    let token = registry.register(new_view);
    Ok(IfaValue::Resource(Arc::new(token)))
}

fn handle_par_reduce(
    args: Vec<IfaValue>,
    ctx: &mut ifa_vm::native::VmContext,
) -> IfaResult<IfaValue> {
    let view_token = match args.first() {
        Some(IfaValue::Resource(r)) => **r,
        _ => {
            return Err(IfaError::ArgumentError(
                "cpu.par_reduce: first arg must be CpuOponView".into(),
            ));
        }
    };
    let op = match args.get(1) {
        Some(IfaValue::Str(s)) => s.to_string(),
        _ => {
            return Err(IfaError::ArgumentError(
                "cpu.par_reduce: second arg must be operation string".into(),
            ));
        }
    };

    let registry = ctx.resource_registry();
    let view = registry
        .get::<CpuOponView>(view_token)
        .ok_or_else(|| IfaError::Runtime("CpuOponView handle not found".into()))?;

    // Pre-validate
    reduce_numeric_op(&[0.0], &op)?;

    let buffer = view
        .buffer
        .read()
        .map_err(|_| IfaError::Runtime("CpuOponView read lock poisoned".into()))?;
    let result = reduce_numeric_op(&buffer, &op)?;

    Ok(IfaValue::Float(result as f64))
}

fn map_numeric_op(value: f32, op: &str) -> IfaResult<f32> {
    match op {
        "square" => Ok(value * value),
        "cube" => Ok(value * value * value),
        "double" => Ok(value * 2.0),
        "increment" | "inc" => Ok(value + 1.0),
        "decrement" | "dec" => Ok(value - 1.0),
        "neg" | "negate" => Ok(-value),
        "abs" => Ok(value.abs()),
        "sqrt" => Ok(value.sqrt()),
        _ => Err(IfaError::ArgumentError(format!(
            "Cpu.par_map unknown operation '{}'",
            op
        ))),
    }
}

fn reduce_numeric_op(data: &[f32], op: &str) -> IfaResult<f32> {
    match op {
        "sum" => Ok(CpuContext::par_sum(data)),
        "product" | "prod" => Ok(CpuContext::par_reduce(data, 1.0, |x| *x, |a, b| a * b)),
        "min" => Ok(CpuContext::par_reduce(
            data,
            f32::INFINITY,
            |x| *x,
            |a: f32, b: f32| a.min(b),
        )),
        "max" => Ok(CpuContext::par_reduce(
            data,
            f32::NEG_INFINITY,
            |x| *x,
            |a: f32, b: f32| a.max(b),
        )),
        _ => Err(IfaError::ArgumentError(format!(
            "Cpu.par_reduce unknown operation '{}'",
            op
        ))),
    }
}

fn handle_iterator_method(method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    if args.is_empty() {
        return Err(IfaError::ArgumentError(format!(
            "Cpu.{} expects at least 1 argument",
            method
        )));
    }
    let list_val = &args[0];

    // Convert IfaValue to a vector of f64 for math operations if possible
    let mut vec_f64 = Vec::new();
    let mut vec_int = Vec::new();
    let mut is_float = false;

    if let IfaValue::List(list) = list_val {
        for v in list.iter() {
            match v {
                IfaValue::Float(f) => {
                    is_float = true;
                    vec_f64.push(*f);
                    vec_int.push(*f as i64);
                }
                IfaValue::Int(i) => {
                    vec_f64.push(*i as f64);
                    vec_int.push(*i);
                }
                _ => {
                    return Err(IfaError::TypeError {
                        expected: "List of numbers".into(),
                        got: v.type_name().into(),
                    });
                }
            }
        }
    } else {
        return Err(IfaError::TypeError {
            expected: "List".into(),
            got: list_val.type_name().into(),
        });
    }

    match method {
        "positive" => {
            let res: Vec<IfaValue> = if is_float {
                vec_f64
                    .into_iter()
                    .filter(|&x| x > 0.0)
                    .map(IfaValue::float)
                    .collect()
            } else {
                vec_int
                    .into_iter()
                    .filter(|&x| x > 0)
                    .map(IfaValue::int)
                    .collect()
            };
            Ok(IfaValue::list(res))
        }
        "negative" => {
            let res: Vec<IfaValue> = if is_float {
                vec_f64
                    .into_iter()
                    .filter(|&x| x < 0.0)
                    .map(IfaValue::float)
                    .collect()
            } else {
                vec_int
                    .into_iter()
                    .filter(|&x| x < 0)
                    .map(IfaValue::int)
                    .collect()
            };
            Ok(IfaValue::list(res))
        }
        "nonzero" => {
            let res: Vec<IfaValue> = if is_float {
                vec_f64
                    .into_iter()
                    .filter(|&x| x != 0.0)
                    .map(IfaValue::float)
                    .collect()
            } else {
                vec_int
                    .into_iter()
                    .filter(|&x| x != 0)
                    .map(IfaValue::int)
                    .collect()
            };
            Ok(IfaValue::list(res))
        }
        "even" => {
            let res: Vec<IfaValue> = vec_int
                .into_iter()
                .filter(|&x| x % 2 == 0)
                .map(IfaValue::int)
                .collect();
            Ok(IfaValue::list(res))
        }
        "odd" => {
            let res: Vec<IfaValue> = vec_int
                .into_iter()
                .filter(|&x| x % 2 != 0)
                .map(IfaValue::int)
                .collect();
            Ok(IfaValue::list(res))
        }
        "sum" | "par_sum" => {
            if is_float {
                let sum: f64 = vec_f64.into_iter().sum();
                Ok(IfaValue::float(sum))
            } else {
                let sum: i64 = vec_int.into_iter().sum();
                Ok(IfaValue::int(sum))
            }
        }
        "product" => {
            if is_float {
                let prod: f64 = vec_f64.into_iter().product();
                Ok(IfaValue::float(prod))
            } else {
                let prod: i64 = vec_int.into_iter().product();
                Ok(IfaValue::int(prod))
            }
        }
        "min" => {
            if is_float {
                let m = vec_f64.into_iter().fold(f64::INFINITY, |a, b| a.min(b));
                Ok(IfaValue::float(m))
            } else {
                let m = vec_int.into_iter().min().unwrap_or(0);
                Ok(IfaValue::int(m))
            }
        }
        "max" => {
            if is_float {
                let m = vec_f64.into_iter().fold(f64::NEG_INFINITY, |a, b| a.max(b));
                Ok(IfaValue::float(m))
            } else {
                let m = vec_int.into_iter().max().unwrap_or(0);
                Ok(IfaValue::int(m))
            }
        }
        "par_sort" => {
            if is_float {
                vec_f64.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                Ok(IfaValue::list(
                    vec_f64.into_iter().map(IfaValue::float).collect(),
                ))
            } else {
                vec_int.sort();
                Ok(IfaValue::list(
                    vec_int.into_iter().map(IfaValue::int).collect(),
                ))
            }
        }
        "par_filter" => {
            // Placeholder: par_filter should ideally take a closure
            // We just implement a basic copy to satisfy the method mapping if no closure given
            if is_float {
                Ok(IfaValue::list(
                    vec_f64.into_iter().map(IfaValue::float).collect(),
                ))
            } else {
                Ok(IfaValue::list(
                    vec_int.into_iter().map(IfaValue::int).collect(),
                ))
            }
        }
        _ => Err(IfaError::ArgumentError(format!(
            "Unknown Cpu iterator method {}",
            method
        ))),
    }
}
