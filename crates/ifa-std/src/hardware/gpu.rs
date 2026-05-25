//! GPU Domain (Domain 19) wrapper

#[cfg(feature = "gpu")]
use ifa_infra::gpu::{GpuContext, GpuVec};
use ifa_types::{IfaError, IfaResult};
use ifa_types::value_union::{IfaValue, FutureState};
use wgpu;
use std::sync::{Arc, Mutex};

/// A hardware-aware zero-copy buffer.
/// Registered in the VM's ResourceRegistry as `IfaValue::Resource(token)`.
#[cfg(feature = "gpu")]
pub struct OponView {
    pub buffer: wgpu::Buffer,
    pub size_in_floats: usize,
}

#[cfg(feature = "gpu")]
pub fn handle_init(args: Vec<IfaValue>, ctx: &mut ifa_vm::native::VmContext) -> IfaResult<IfaValue> {
    let registry = ctx.resource_registry();
    let cell = Arc::new(Mutex::new(FutureState::Pending));
    let cell_clone = cell.clone();

    std::thread::Builder::new()
        .name("ifa-gpu-init".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("ifa-gpu runtime");

            let val = match rt.block_on(GpuContext::new()) {
                Ok(gpu) => {
                    let token = registry.register(gpu);
                    IfaValue::Resource(Arc::new(token))
                }
                Err(e) => IfaValue::str(format!("GpuError: {e}")),
            };
            *cell_clone.lock().unwrap() = FutureState::Ready(val);
        })
        .map_err(|e| IfaError::Runtime(format!("GPU thread spawn failed: {e}")))?;

    Ok(IfaValue::Future(cell))
}

#[cfg(feature = "gpu")]
pub fn handle_dispatch_pipeline(args: Vec<IfaValue>, ctx: &mut ifa_vm::native::VmContext) -> IfaResult<IfaValue> {
    let token = match args.first() {
        Some(IfaValue::Resource(r)) => **r,
        _ => return Err(IfaError::ArgumentError("gpu.dispatch: first arg must be gpu handle".into())),
    };
    let name = args.get(1).map(|v| v.to_string()).unwrap_or_default();
    let x = args.get(2).and_then(|v| if let IfaValue::Int(i) = v { Some(*i as u32) } else { None }).unwrap_or(1);
    let y = args.get(3).and_then(|v| if let IfaValue::Int(i) = v { Some(*i as u32) } else { None }).unwrap_or(1);
    let z = args.get(4).and_then(|v| if let IfaValue::Int(i) = v { Some(*i as u32) } else { None }).unwrap_or(1);

    let registry = ctx.resource_registry();
    if let Some(gpu) = registry.get::<GpuContext>(token) {
        gpu.dispatch_pipeline(&name, x, y, z).map_err(IfaError::Runtime)?;
        Ok(IfaValue::null())
    } else {
        Err(IfaError::Runtime("GPU handle not found in registry".into()))
    }
}

#[cfg(feature = "gpu")]
pub fn handle_sync(args: Vec<IfaValue>, ctx: &mut ifa_vm::native::VmContext) -> IfaResult<IfaValue> {
    let token = match args.first() {
        Some(IfaValue::Resource(r)) => **r,
        _ => return Err(IfaError::ArgumentError("gpu.sync: first arg must be gpu handle".into())),
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
pub fn handle_alloc_buffer(args: Vec<IfaValue>, ctx: &mut ifa_vm::native::VmContext) -> IfaResult<IfaValue> {
    let gpu_token = match args.first() {
        Some(IfaValue::Resource(r)) => **r,
        _ => return Err(IfaError::ArgumentError("gpu.alloc_buffer: first arg must be gpu handle".into())),
    };
    let size_in_floats = match args.get(1) {
        Some(IfaValue::Int(i)) => *i as usize,
        _ => return Err(IfaError::ArgumentError("gpu.alloc_buffer: second arg must be size (int)".into())),
    };

    let registry = ctx.resource_registry();
    if let Some(gpu) = registry.get::<GpuContext>(gpu_token) {
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("OponView_Buffer"),
            size: (size_in_floats * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let opon_view = OponView {
            buffer,
            size_in_floats,
        };

        let view_token = registry.register(opon_view);
        Ok(IfaValue::Resource(Arc::new(view_token)))
    } else {
        Err(IfaError::Runtime("GPU handle not found in registry".into()))
    }
}

#[cfg(feature = "gpu")]
pub fn handle_read_buffer(args: Vec<IfaValue>, ctx: &mut ifa_vm::native::VmContext) -> IfaResult<IfaValue> {
    let gpu_token = match args.first() {
        Some(IfaValue::Resource(r)) => **r,
        _ => return Err(IfaError::ArgumentError("gpu.read_buffer: first arg must be gpu handle".into())),
    };
    let view_token = match args.get(1) {
        Some(IfaValue::Resource(r)) => **r,
        _ => return Err(IfaError::ArgumentError("gpu.read_buffer: second arg must be OponView resource".into())),
    };

    let registry = ctx.resource_registry();
    let gpu = registry.get::<GpuContext>(gpu_token).ok_or_else(|| IfaError::Runtime("GPU handle not found".into()))?;
    let view = registry.get::<OponView>(view_token).ok_or_else(|| IfaError::Runtime("OponView handle not found".into()))?;

    let bytes = gpu.read_buffer(&view.buffer).map_err(IfaError::Runtime)?;
    
    // Cast bytes back to floats
    let floats: &[f32] = bytemuck::cast_slice(&bytes);
    let mut list = Vec::with_capacity(view.size_in_floats);
    for i in 0..view.size_in_floats {
        list.push(IfaValue::Float(floats[i] as f64));
    }

    Ok(IfaValue::List(Arc::new(list)))
}

#[cfg(feature = "gpu")]
pub fn handle_write_buffer(args: Vec<IfaValue>, ctx: &mut ifa_vm::native::VmContext) -> IfaResult<IfaValue> {
    let gpu_token = match args.first() {
        Some(IfaValue::Resource(r)) => **r,
        _ => return Err(IfaError::ArgumentError("gpu.write_buffer: first arg must be gpu handle".into())),
    };
    let view_token = match args.get(1) {
        Some(IfaValue::Resource(r)) => **r,
        _ => return Err(IfaError::ArgumentError("gpu.write_buffer: second arg must be OponView resource".into())),
    };
    let list_val = match args.get(2) {
        Some(IfaValue::List(l)) => l,
        _ => return Err(IfaError::ArgumentError("gpu.write_buffer: third arg must be a List of numbers".into())),
    };

    let registry = ctx.resource_registry();
    let gpu = registry.get::<GpuContext>(gpu_token).ok_or_else(|| IfaError::Runtime("GPU handle not found".into()))?;
    let view = registry.get::<OponView>(view_token).ok_or_else(|| IfaError::Runtime("OponView handle not found".into()))?;

    if list_val.len() != view.size_in_floats {
        return Err(IfaError::Runtime(format!("List size {} does not match OponView size {}", list_val.len(), view.size_in_floats)));
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
