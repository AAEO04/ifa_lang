//! # GPU Infrastructure (The Warrior's Forge)
//! 
//! Hardware acceleration via WGPU. Wraps Device, Queue, and Pipeline creation.

#[cfg(feature = "gpu")]
use wgpu::{Instance, Device, Queue};
use std::sync::Arc;

/// Global GPU Context
#[derive(Clone)]
pub struct GpuContext {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    // Surface is optional (headless compute vs graphics)
}

#[cfg(feature = "gpu")]
impl GpuContext {
    /// Initialize GPU (Headless)
    pub async fn new() -> Result<Self, String> {
        let instance = Instance::default();
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or("No suitable GPU adapter found")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Ifa-Lang Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                },
                None, // Trace path
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
        })
    }
    

    /// Blocking initialization (for synchronous CLI)
    pub fn new_blocking() -> Result<Self, String> {
        pollster::block_on(Self::new())
    }

    /// Create a Compute Pipeline from WGSL source
    pub fn create_compute_pipeline(
        &self,
        label: &str,
        shader_source: &str,
        entry_point: &str,
    ) -> wgpu::ComputePipeline {
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(label),
            layout: None,
            module: &shader,
            entry_point,
        })
    }
}


/// GPU Buffer Wrapper (GpuVec)
#[cfg(feature = "gpu")]
pub struct GpuVec<T> {
    pub buffer: wgpu::Buffer,
    pub len: usize,
    _marker: std::marker::PhantomData<T>,
}

#[cfg(feature = "gpu")]
impl<T> GpuVec<T> {
    pub fn new(buffer: wgpu::Buffer, len: usize) -> Self {
        Self {
            buffer,
            len,
            _marker: std::marker::PhantomData,
        }
    }
}


/// Simple VRAM Memory Pool (Skeleton)
#[cfg(feature = "gpu")]
pub struct MemoryPool {
    device: Arc<Device>,
    allocated_buffers: Vec<wgpu::Buffer>,
    // In a real allocator, we would track free blocks here
}

#[cfg(feature = "gpu")]
impl MemoryPool {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            device,
            allocated_buffers: Vec::new(),
        }
    }

    pub fn allocate(&mut self, size: u64, label: &str) -> wgpu::Buffer {
        // TODO: Implement actual pooling/reuse logic
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        // We don't store it in 'allocated_buffers' yet because we are returning ownership.
        // A real pool would return a 'Handle' or 'Arc<Buffer>'.
        buffer
    }
}
