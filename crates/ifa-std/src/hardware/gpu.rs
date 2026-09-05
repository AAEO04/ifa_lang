//! GPU Domain (Domain 19) wrapper

#[cfg(feature = "gpu")]
use ifa_infra::gpu::GpuContext;
use ifa_types::value_union::{IfaValue, NativeFutureCell, NativeFutureState};
use ifa_types::{IfaError, IfaResult};
use std::sync::Arc;
#[cfg(feature = "gpu")]
use wgpu;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

/// A hardware-aware zero-copy buffer.
/// Registered in the VM's ResourceRegistry as `IfaValue::Resource(token)`.
///
/// The name deliberately avoids `OponView`, which is reserved for the planned
/// borrow-checking design in `docs/design/AMBITIOUS_FEATURES.md`.
#[cfg(feature = "gpu")]
pub struct GpuBufferView {
    pub buffer: wgpu::Buffer,
    pub size_in_floats: usize,
}

// SAFETY: wasm32 is single-threaded. `wgpu::Buffer` does not implement `Send`/`Sync`
// on wasm32 because it contains wasm_bindgen types. Since the wasm32 target has no
// true thread concurrency, implementing these traits is sound for our usage pattern.
#[cfg(all(feature = "gpu", target_arch = "wasm32"))]
unsafe impl Send for GpuBufferView {}
#[cfg(all(feature = "gpu", target_arch = "wasm32"))]
unsafe impl Sync for GpuBufferView {}

fn pending_cell() -> NativeFutureCell {
    Arc::new(std::sync::RwLock::new(NativeFutureState::Pending))
}

fn resolve_cell(cell: &NativeFutureCell, value: IfaValue) {
    // If the lock is poisoned (the holding thread panicked), recover the inner value
    // so the future resolves rather than hanging as Pending forever.
    *cell.write().unwrap_or_else(|e| e.into_inner()) =
        NativeFutureState::Ready(bincode::serialize(&value).unwrap());
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_native_task<F>(name: &str, cell: NativeFutureCell, task: F) -> IfaResult<IfaValue>
where
    F: FnOnce() -> IfaValue + Send + 'static,
{
    let future_cell = cell.clone();
    std::thread::Builder::new()
        .name(name.into())
        .spawn(move || resolve_cell(&future_cell, task()))
        .map_err(|e| IfaError::Runtime(format!("{name} thread spawn failed: {e}")))?;

    Ok(IfaValue::NativeFuture(cell))
}

#[cfg(target_arch = "wasm32")]
fn spawn_wasm_task<F>(cell: NativeFutureCell, task: F) -> IfaValue
where
    F: std::future::Future<Output = IfaValue> + 'static,
{
    let cell_clone = cell.clone();
    spawn_local(async move {
        let value = task.await;
        resolve_cell(&cell_clone, value);
    });
    IfaValue::NativeFuture(cell)
}

#[cfg(feature = "gpu")]
pub fn handle_init(
    _args: Vec<IfaValue>,
    ctx: &mut ifa_vm::native::VmContext,
) -> IfaResult<IfaValue> {
    let registry = ctx.resource_registry();
    let cell = pending_cell();

    #[cfg(target_arch = "wasm32")]
    {
        Ok(spawn_wasm_task(cell, async move {
            match GpuContext::new().await {
                Ok(gpu) => {
                    let token = registry.register(gpu);
                    IfaValue::Resource(Arc::new(token))
                }
                Err(e) => IfaValue::str(format!("GpuError: {e}")),
            }
        }))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        spawn_native_task("ifa-gpu-init", cell, move || {
            match pollster::block_on(GpuContext::new()) {
                Ok(gpu) => {
                    let token = registry.register(gpu);
                    IfaValue::Resource(Arc::new(token))
                }
                Err(e) => IfaValue::str(format!("GpuError: {e}")),
            }
        })
    }
}

#[cfg(feature = "gpu")]
pub fn handle_dispatch_pipeline(
    args: Vec<IfaValue>,
    ctx: &mut ifa_vm::native::VmContext,
) -> IfaResult<IfaValue> {
    let token = match args.first() {
        Some(IfaValue::Resource(r)) => **r,
        _ => {
            return Err(IfaError::ArgumentError(
                "gpu.dispatch: first arg must be gpu handle".into(),
            ));
        }
    };
    let name = args.get(1).map(|v| v.to_string()).unwrap_or_default();
    let x = args
        .get(2)
        .and_then(|v| {
            if let IfaValue::Int(i) = v {
                Some(*i as u32)
            } else {
                None
            }
        })
        .unwrap_or(1);
    let y = args
        .get(3)
        .and_then(|v| {
            if let IfaValue::Int(i) = v {
                Some(*i as u32)
            } else {
                None
            }
        })
        .unwrap_or(1);
    let z = args
        .get(4)
        .and_then(|v| {
            if let IfaValue::Int(i) = v {
                Some(*i as u32)
            } else {
                None
            }
        })
        .unwrap_or(1);

    let registry = ctx.resource_registry();
    if let Some(gpu) = registry.get::<GpuContext>(token) {
        gpu.dispatch_pipeline(&name, x, y, z)
            .map_err(IfaError::Runtime)?;
        Ok(IfaValue::null())
    } else {
        Err(IfaError::Runtime("GPU handle not found in registry".into()))
    }
}

#[cfg(feature = "gpu")]
pub fn handle_sync(
    args: Vec<IfaValue>,
    ctx: &mut ifa_vm::native::VmContext,
) -> IfaResult<IfaValue> {
    let token = match args.first() {
        Some(IfaValue::Resource(r)) => **r,
        _ => {
            return Err(IfaError::ArgumentError(
                "gpu.sync: first arg must be gpu handle".into(),
            ));
        }
    };
    let registry = ctx.resource_registry();
    if let Some(gpu) = registry.get::<GpuContext>(token) {
        gpu.sync();
        Ok(IfaValue::null())
    } else {
        Err(IfaError::Runtime("GPU handle not found in registry".into()))
    }
}

#[cfg(feature = "gpu")]
pub fn handle_alloc_buffer(
    args: Vec<IfaValue>,
    ctx: &mut ifa_vm::native::VmContext,
) -> IfaResult<IfaValue> {
    let gpu_token = match args.first() {
        Some(IfaValue::Resource(r)) => **r,
        _ => {
            return Err(IfaError::ArgumentError(
                "gpu.alloc_buffer: first arg must be gpu handle".into(),
            ));
        }
    };
    let size_in_floats = match args.get(1) {
        Some(IfaValue::Int(i)) => *i as usize,
        _ => {
            return Err(IfaError::ArgumentError(
                "gpu.alloc_buffer: second arg must be size (int)".into(),
            ));
        }
    };

    let registry = ctx.resource_registry();
    if let Some(gpu) = registry.get::<GpuContext>(gpu_token) {
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GpuBufferView_Buffer"),
            size: (size_in_floats * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let view = GpuBufferView {
            buffer,
            size_in_floats,
        };

        let view_token = registry.register(view);
        Ok(IfaValue::Resource(Arc::new(view_token)))
    } else {
        Err(IfaError::Runtime("GPU handle not found in registry".into()))
    }
}

#[cfg(feature = "gpu")]
pub fn handle_read_buffer(
    args: Vec<IfaValue>,
    ctx: &mut ifa_vm::native::VmContext,
) -> IfaResult<IfaValue> {
    let gpu_token = match args.first() {
        Some(IfaValue::Resource(r)) => **r,
        _ => {
            return Err(IfaError::ArgumentError(
                "gpu.read_buffer: first arg must be gpu handle".into(),
            ));
        }
    };
    let view_token = match args.get(1) {
        Some(IfaValue::Resource(r)) => **r,
        _ => {
            return Err(IfaError::ArgumentError(
                "gpu.read_buffer: second arg must be GpuBufferView resource".into(),
            ));
        }
    };

    let registry = ctx.resource_registry();
    let gpu = registry
        .get::<GpuContext>(gpu_token)
        .ok_or_else(|| IfaError::Runtime("GPU handle not found".into()))?;
    let view = registry
        .get::<GpuBufferView>(view_token)
        .ok_or_else(|| IfaError::Runtime("GpuBufferView handle not found".into()))?;
    let cell = pending_cell();

    #[cfg(target_arch = "wasm32")]
    {
        let gpu = gpu.clone();
        let view = view.clone();
        Ok(spawn_wasm_task(cell, async move {
            match gpu.read_buffer_async(&view.buffer).await {
                Ok(bytes) => bytes_to_ifa_list(&bytes, view.size_in_floats),
                Err(e) => IfaValue::str(format!("GpuError: {e}")),
            }
        }))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let gpu = gpu.clone();
        let view = view.clone();
        spawn_native_task("ifa-gpu-readback", cell, move || {
            match pollster::block_on(gpu.read_buffer_async(&view.buffer)) {
                Ok(bytes) => bytes_to_ifa_list(&bytes, view.size_in_floats),
                Err(e) => IfaValue::str(format!("GpuError: {e}")),
            }
        })
    }
}

#[cfg(feature = "gpu")]
pub fn handle_write_buffer(
    args: Vec<IfaValue>,
    ctx: &mut ifa_vm::native::VmContext,
) -> IfaResult<IfaValue> {
    let gpu_token = match args.first() {
        Some(IfaValue::Resource(r)) => **r,
        _ => {
            return Err(IfaError::ArgumentError(
                "gpu.write_buffer: first arg must be gpu handle".into(),
            ));
        }
    };
    let view_token = match args.get(1) {
        Some(IfaValue::Resource(r)) => **r,
        _ => {
            return Err(IfaError::ArgumentError(
                "gpu.write_buffer: second arg must be GpuBufferView resource".into(),
            ));
        }
    };
    let list_val = match args.get(2) {
        Some(IfaValue::List(l)) => l,
        _ => {
            return Err(IfaError::ArgumentError(
                "gpu.write_buffer: third arg must be a List of numbers".into(),
            ));
        }
    };

    let registry = ctx.resource_registry();
    let gpu = registry
        .get::<GpuContext>(gpu_token)
        .ok_or_else(|| IfaError::Runtime("GPU handle not found".into()))?;
    let view = registry
        .get::<GpuBufferView>(view_token)
        .ok_or_else(|| IfaError::Runtime("GpuBufferView handle not found".into()))?;

    if list_val.len() != view.size_in_floats {
        return Err(IfaError::Runtime(format!(
            "List size {} does not match GpuBufferView size {}",
            list_val.len(),
            view.size_in_floats
        )));
    }

    let mut float_vec = Vec::with_capacity(view.size_in_floats);
    for item in list_val.iter() {
        match item {
            IfaValue::Float(f) => float_vec.push(*f as f32),
            IfaValue::Int(i) => float_vec.push(*i as f32),
            _ => return Err(IfaError::Runtime("List must contain only numbers".into())),
        }
    }

    let bytes: &[u8] = bytemuck::cast_slice(&float_vec);
    gpu.queue.write_buffer(&view.buffer, 0, bytes);

    Ok(IfaValue::null())
}

#[cfg(feature = "gpu")]
fn bytes_to_ifa_list(bytes: &[u8], size_in_floats: usize) -> IfaValue {
    let floats: &[f32] = bytemuck::cast_slice(bytes);
    let mut list = Vec::with_capacity(size_in_floats);
    for &f in floats.iter().take(size_in_floats) {
        list.push(IfaValue::Float(f as f64));
    }
    IfaValue::List(ifa_types::gc::IfaGc::new(list))
}

#[cfg(feature = "gpu")]
pub fn dispatch(
    method: &str,
    args: Vec<IfaValue>,
    ctx: &mut ifa_vm::native::VmContext,
) -> IfaResult<IfaValue> {
    match method {
        "init" => handle_init(args, ctx),
        "dispatch" => handle_dispatch_pipeline(args, ctx),
        "sync" => handle_sync(args, ctx),
        "alloc_buffer" => handle_alloc_buffer(args, ctx),
        "read_buffer" => handle_read_buffer(args, ctx),
        "write_buffer" => handle_write_buffer(args, ctx),
        _ => Err(IfaError::Custom(format!("Gpu: unknown method '{}'", method))),
    }
}

