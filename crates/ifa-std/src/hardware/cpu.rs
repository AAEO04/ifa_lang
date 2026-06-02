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

    let buffer = view.buffer.read().map_err(|_| IfaError::Runtime("CpuOponView read lock poisoned".into()))?;
    let mut list = Vec::with_capacity(view.size);
    for &val in buffer.iter() {
        list.push(IfaValue::Float(val as f64));
    }

    Ok(IfaValue::List(Arc::new(list)))
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

    let mut buffer = view.buffer.write().map_err(|_| IfaError::Runtime("CpuOponView write lock poisoned".into()))?;
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

    let buffer = view.buffer.read().map_err(|_| IfaError::Runtime("CpuOponView read lock poisoned".into()))?;
    let mapped = CpuContext::par_map(&*buffer, |x| map_numeric_op(*x, &op).unwrap());

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

    let buffer = view.buffer.read().map_err(|_| IfaError::Runtime("CpuOponView read lock poisoned".into()))?;
    let result = reduce_numeric_op(&*buffer, &op)?;

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
            |a, b| a.min(b),
        )),
        "max" => Ok(CpuContext::par_reduce(
            data,
            f32::NEG_INFINITY,
            |x| *x,
            |a, b| a.max(b),
        )),
        _ => Err(IfaError::ArgumentError(format!(
            "Cpu.par_reduce unknown operation '{}'",
            op
        ))),
    }
}
