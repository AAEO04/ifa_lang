//! # GPU Infrastructure (The Warrior's Forge)
//!
//! Hardware acceleration via WGPU. Includes production-grade memory pooling with slab allocator.

#[cfg(feature = "gpu")]
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(feature = "gpu")]
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
#[cfg(feature = "gpu")]
use wgpu::{Device, Instance, Queue};

/// Global GPU Context with pipeline caching
#[cfg(feature = "gpu")]
pub struct GpuContext {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    /// Cached compute pipelines by shader name
    pipeline_cache: Arc<RwLock<HashMap<String, Arc<wgpu::ComputePipeline>>>>,
}

#[cfg(feature = "gpu")]
impl Clone for GpuContext {
    fn clone(&self) -> Self {
        Self {
            device: self.device.clone(),
            queue: self.queue.clone(),
            pipeline_cache: self.pipeline_cache.clone(),
        }
    }
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
                None,
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            pipeline_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Blocking initialization (for synchronous CLI)
    pub fn new_blocking() -> Result<Self, String> {
        pollster::block_on(Self::new())
    }

    /// Create a Compute Pipeline from WGSL source (uncached)
    fn create_pipeline_uncached(
        &self,
        label: &str,
        shader_source: &str,
        entry_point: &str,
    ) -> wgpu::ComputePipeline {
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });

        self.device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(label),
                layout: None,
                module: &shader,
                entry_point,
            })
    }

    /// Get or create a cached compute pipeline
    pub fn get_or_create_pipeline(
        &self,
        name: &str,
        shader_source: &str,
        entry_point: &str,
    ) -> Arc<wgpu::ComputePipeline> {
        // Try read lock first
        {
            let cache = self.pipeline_cache.read().unwrap();
            if let Some(pipeline) = cache.get(name) {
                return pipeline.clone();
            }
        }

        // Create and cache
        let pipeline = Arc::new(self.create_pipeline_uncached(name, shader_source, entry_point));
        let mut cache = self.pipeline_cache.write().unwrap();
        cache.insert(name.to_string(), pipeline.clone());
        pipeline
    }

    /// Create a Compute Pipeline from WGSL source (for backwards compatibility)
    pub fn create_compute_pipeline(
        &self,
        label: &str,
        shader_source: &str,
        entry_point: &str,
    ) -> wgpu::ComputePipeline {
        self.create_pipeline_uncached(label, shader_source, entry_point)
    }

    /// Create a memory pool for this GPU context
    pub fn create_memory_pool(&self, total_size: u64) -> MemoryPool {
        MemoryPool::new(self.device.clone(), total_size)
    }

    /// Create a slab memory pool
    pub fn create_slab_pool(&self) -> SlabMemoryPool {
        SlabMemoryPool::new(self.device.clone())
    }

    // =========================================================================
    // HIGH-LEVEL COMPUTE API (uses shaders from shaders.rs)
    // =========================================================================

    /// Matrix multiplication: C = A * B
    /// A is M×K, B is K×N, C is M×N
    pub fn matmul(
        &self,
        a: &wgpu::Buffer,
        b: &wgpu::Buffer,
        m: u32,
        n: u32,
        k: u32,
    ) -> wgpu::Buffer {
        use super::shaders::TILED_MATMUL_SHADER;
        use wgpu::util::DeviceExt;

        // Create output buffer
        let c = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("matmul_output"),
            size: (m * n) as u64 * std::mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Create params uniform
        let params = [m, n, k, 0u32]; // M, N, K, pad
        let params_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("matmul_params"),
                contents: bytemuck::cast_slice(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let pipeline = self.create_compute_pipeline("tiled_matmul", TILED_MATMUL_SHADER, "main");
        let bind_group_layout = pipeline.get_bind_group_layout(0);
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("matmul_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: c.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("matmul_encoder"),
            });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("matmul_pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            // Dispatch workgroups (16x16 tiles)
            let wg_x = (n + 15) / 16;
            let wg_y = (m + 15) / 16;
            pass.dispatch_workgroups(wg_x, wg_y, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        c
    }

    /// Apply ReLU activation in-place
    pub fn relu(&self, data: &wgpu::Buffer, count: u32) {
        use super::shaders::RELU_SHADER;
        use wgpu::util::DeviceExt;

        let params = [count, 0u32, 0u32, 0u32];
        let params_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("relu_params"),
                contents: bytemuck::cast_slice(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let pipeline = self.create_compute_pipeline("relu", RELU_SHADER, "main");
        let bind_group_layout = pipeline.get_bind_group_layout(0);
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("relu_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: data.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("relu_encoder"),
            });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("relu_pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((count + 255) / 256, 1, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Elementwise map: output[i] = input[i] * scale + bias
    pub fn map_scale_bias(
        &self,
        input: &wgpu::Buffer,
        count: u32,
        scale: f32,
        bias: f32,
    ) -> wgpu::Buffer {
        use super::shaders::MAP_SCALE_BIAS_SHADER;
        use wgpu::util::DeviceExt;

        let output = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("map_output"),
            size: count as u64 * std::mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Pack params: count, scale, bias, pad
        let params: [u32; 4] = [count, scale.to_bits(), bias.to_bits(), 0];
        let params_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("map_params"),
                contents: bytemuck::cast_slice(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let pipeline =
            self.create_compute_pipeline("map_scale_bias", MAP_SCALE_BIAS_SHADER, "main");
        let bind_group_layout = pipeline.get_bind_group_layout(0);
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("map_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: input.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: output.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("map_encoder"),
            });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("map_pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((count + 255) / 256, 1, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output
    }

    /// Vector addition: c = a + b
    pub fn vec_add(&self, a: &wgpu::Buffer, b: &wgpu::Buffer, count: u32) -> wgpu::Buffer {
        use super::shaders::VEC_ADD_SHADER;
        use wgpu::util::DeviceExt;

        let c = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vec_add_output"),
            size: count as u64 * std::mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let params = [count, 0u32, 0u32, 0u32];
        let params_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vec_add_params"),
                contents: bytemuck::cast_slice(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let pipeline = self.create_compute_pipeline("vec_add", VEC_ADD_SHADER, "main");
        let bind_group_layout = pipeline.get_bind_group_layout(0);
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("vec_add_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: c.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("vec_add_encoder"),
            });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("vec_add_pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((count + 255) / 256, 1, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        c
    }

    /// Wait for all GPU operations to complete
    pub fn sync(&self) {
        self.device.poll(wgpu::Maintain::Wait);
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

/// A handle to a suballocated region within a MemoryPool
#[cfg(feature = "gpu")]
#[derive(Debug, Clone)]
pub struct PoolAllocation {
    /// Offset within the pool's buffer
    pub offset: u64,
    /// Size of this allocation
    pub size: u64,
    /// Slab index (for slab allocator)
    pub slab_index: Option<usize>,
}

/// Simple bump allocator MemoryPool (for frame-based allocation)
#[cfg(feature = "gpu")]
pub struct MemoryPool {
    _device: Arc<Device>,
    pub buffer: wgpu::Buffer,
    block_size: u64,
    next_offset: AtomicU64,
}

#[cfg(feature = "gpu")]
impl MemoryPool {
    pub const DEFAULT_SIZE: u64 = 64 * 1024 * 1024;

    pub fn new(device: Arc<Device>, block_size: u64) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("MemoryPool"),
            size: block_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Self {
            _device: device,
            buffer,
            block_size,
            next_offset: AtomicU64::new(0),
        }
    }

    pub fn with_default_size(device: Arc<Device>) -> Self {
        Self::new(device, Self::DEFAULT_SIZE)
    }

    pub fn allocate(&self, size: u64) -> Option<PoolAllocation> {
        const ALIGNMENT: u64 = 256;
        let aligned_size = (size + ALIGNMENT - 1) & !(ALIGNMENT - 1);

        loop {
            let current = self.next_offset.load(Ordering::Relaxed);
            let new_offset = current + aligned_size;

            if new_offset > self.block_size {
                return None;
            }

            match self.next_offset.compare_exchange_weak(
                current,
                new_offset,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    return Some(PoolAllocation {
                        offset: current,
                        size: aligned_size,
                        slab_index: None,
                    });
                }
                Err(_) => continue,
            }
        }
    }

    pub fn reset(&self) {
        self.next_offset.store(0, Ordering::SeqCst);
    }

    pub fn available(&self) -> u64 {
        self.block_size
            .saturating_sub(self.next_offset.load(Ordering::Relaxed))
    }

    pub fn slice(&self, allocation: &PoolAllocation) -> wgpu::BufferSlice<'_> {
        self.buffer
            .slice(allocation.offset..allocation.offset + allocation.size)
    }
}

/// Slab size classes for the slab allocator
#[cfg(feature = "gpu")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlabClass {
    Tiny = 0,   // 256 bytes
    Small = 1,  // 1 KB
    Medium = 2, // 4 KB
    Large = 3,  // 16 KB
    Huge = 4,   // 64 KB
    Giant = 5,  // 256 KB
    Mega = 6,   // 1 MB
}

#[cfg(feature = "gpu")]
impl SlabClass {
    pub const fn size(&self) -> u64 {
        match self {
            SlabClass::Tiny => 256,
            SlabClass::Small => 1024,
            SlabClass::Medium => 4 * 1024,
            SlabClass::Large => 16 * 1024,
            SlabClass::Huge => 64 * 1024,
            SlabClass::Giant => 256 * 1024,
            SlabClass::Mega => 1024 * 1024,
        }
    }

    pub fn from_size(size: u64) -> Option<Self> {
        if size <= 256 {
            Some(SlabClass::Tiny)
        } else if size <= 1024 {
            Some(SlabClass::Small)
        } else if size <= 4 * 1024 {
            Some(SlabClass::Medium)
        } else if size <= 16 * 1024 {
            Some(SlabClass::Large)
        } else if size <= 64 * 1024 {
            Some(SlabClass::Huge)
        } else if size <= 256 * 1024 {
            Some(SlabClass::Giant)
        } else if size <= 1024 * 1024 {
            Some(SlabClass::Mega)
        } else {
            None
        }
    }

    pub const COUNT: usize = 7;
}

/// Individual slab holding fixed-size allocations
#[cfg(feature = "gpu")]
#[allow(dead_code)] // slot_size stored for diagnostics/resize
struct Slab {
    buffer: wgpu::Buffer,
    slot_size: u64,
    slot_count: usize,
    free_bitmap: Vec<AtomicUsize>,
}

#[cfg(feature = "gpu")]
impl Slab {
    fn new(device: &Device, slot_size: u64, slot_count: usize) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Slab_{}", slot_size)),
            size: slot_size * slot_count as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Bitmap: 1 bit per slot, using usize for atomic ops (64 slots per word on 64-bit)
        let words = (slot_count + 63) / 64;
        let free_bitmap: Vec<AtomicUsize> = (0..words)
            .map(|_| AtomicUsize::new(!0)) // All 1s = all free
            .collect();

        Self {
            buffer,
            slot_size,
            slot_count,
            free_bitmap,
        }
    }

    fn allocate(&self) -> Option<usize> {
        for (word_idx, word) in self.free_bitmap.iter().enumerate() {
            loop {
                let current = word.load(Ordering::Relaxed);
                if current == 0 {
                    break; // No free slots in this word
                }

                // Find first set bit (free slot)
                let bit_idx = current.trailing_zeros() as usize;
                let slot_idx = word_idx * 64 + bit_idx;

                if slot_idx >= self.slot_count {
                    break; // Beyond actual slots
                }

                // Clear the bit (mark as allocated)
                let new_value = current & !(1 << bit_idx);

                match word.compare_exchange_weak(
                    current,
                    new_value,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return Some(slot_idx),
                    Err(_) => continue, // Retry
                }
            }
        }
        None
    }

    fn free(&self, slot_idx: usize) {
        let word_idx = slot_idx / 64;
        let bit_idx = slot_idx % 64;

        if word_idx < self.free_bitmap.len() {
            self.free_bitmap[word_idx].fetch_or(1 << bit_idx, Ordering::SeqCst);
        }
    }

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

/// Production Slab Memory Pool with O(1) alloc/free
#[cfg(feature = "gpu")]
pub struct SlabMemoryPool {
    device: Arc<Device>,
    slabs: [Vec<Slab>; SlabClass::COUNT],
    stats: PoolStats,
}

/// Pool statistics
#[cfg(feature = "gpu")]
#[derive(Debug, Default)]
pub struct PoolStats {
    pub allocations: AtomicU64,
    pub frees: AtomicU64,
    pub bytes_allocated: AtomicU64,
    pub failed_allocations: AtomicU64,
}

#[cfg(feature = "gpu")]
impl SlabMemoryPool {
    const SLOTS_PER_SLAB: usize = 64;

    pub fn new(device: Arc<Device>) -> Self {
        Self {
            device,
            slabs: Default::default(),
            stats: PoolStats::default(),
        }
    }

    /// Allocate memory of given size
    pub fn allocate(&mut self, size: u64) -> Option<SlabAllocation> {
        let class = SlabClass::from_size(size)?;
        let class_idx = class as usize;

        // Try existing slabs
        for (slab_idx, slab) in self.slabs[class_idx].iter().enumerate() {
            if let Some(slot_idx) = slab.allocate() {
                self.stats.allocations.fetch_add(1, Ordering::Relaxed);
                self.stats
                    .bytes_allocated
                    .fetch_add(class.size(), Ordering::Relaxed);

                return Some(SlabAllocation {
                    class,
                    slab_idx,
                    slot_idx,
                    offset: slot_idx as u64 * class.size(),
                    size: class.size(),
                });
            }
        }

        // Create new slab
        let new_slab = Slab::new(&self.device, class.size(), Self::SLOTS_PER_SLAB);
        let slot_idx = new_slab.allocate().unwrap(); // First alloc always succeeds
        let slab_idx = self.slabs[class_idx].len();
        self.slabs[class_idx].push(new_slab);

        self.stats.allocations.fetch_add(1, Ordering::Relaxed);
        self.stats
            .bytes_allocated
            .fetch_add(class.size(), Ordering::Relaxed);

        Some(SlabAllocation {
            class,
            slab_idx,
            slot_idx,
            offset: slot_idx as u64 * class.size(),
            size: class.size(),
        })
    }

    /// Free a previously allocated block
    pub fn free(&self, allocation: &SlabAllocation) {
        let class_idx = allocation.class as usize;
        if allocation.slab_idx < self.slabs[class_idx].len() {
            self.slabs[class_idx][allocation.slab_idx].free(allocation.slot_idx);
            self.stats.frees.fetch_add(1, Ordering::Relaxed);
            self.stats
                .bytes_allocated
                .fetch_sub(allocation.size, Ordering::Relaxed);
        }
    }

    /// Get the buffer for an allocation
    pub fn buffer(&self, allocation: &SlabAllocation) -> Option<&wgpu::Buffer> {
        let class_idx = allocation.class as usize;
        self.slabs[class_idx]
            .get(allocation.slab_idx)
            .map(|slab| slab.buffer())
    }

    /// Get pool statistics
    pub fn stats(&self) -> &PoolStats {
        &self.stats
    }
}

/// Handle for a slab allocation
#[cfg(feature = "gpu")]
#[derive(Debug, Clone)]
pub struct SlabAllocation {
    pub class: SlabClass,
    pub slab_idx: usize,
    pub slot_idx: usize,
    pub offset: u64,
    pub size: u64,
}
