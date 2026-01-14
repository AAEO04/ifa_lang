//! GPU Compute Benchmark
//!
//! Demonstrates massive parallel speedups using GPU compute shaders.

#[cfg(feature = "gpu")]
use ifa_std::infra::gpu::GpuContext;

#[cfg(feature = "gpu")]
fn main() {
    use std::time::Instant;
    use wgpu::util::DeviceExt;

    println!("=== Ifá-Lang GPU Compute Benchmark ===\n");

    // Initialize GPU
    let ctx = GpuContext::new_blocking().expect("Failed to initialize GPU");
    println!("✓ GPU initialized: {:?}", ctx.device.limits().max_compute_workgroups_per_dimension);

    // Test data
    let data_size: u32 = 10_000_000;
    let input_data: Vec<f32> = (0..data_size).map(|i| i as f32).collect();
    
    // --- CPU Baseline ---
    let start = Instant::now();
    let _cpu_result: Vec<f32> = input_data.iter().map(|x| x * 2.0 + 1.0).collect();
    let cpu_duration = start.elapsed();
    println!("CPU sequential ({} elements): {:?}", data_size, cpu_duration);

    // --- GPU Compute ---
    // Create input buffer
    let input_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Input Buffer"),
        contents: bytemuck::cast_slice(&input_data),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    // Create output buffer
    let output_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: (data_size as u64) * std::mem::size_of::<f32>() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // Staging buffer for reading results
    let staging_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: (data_size as u64) * std::mem::size_of::<f32>() as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Compute shader (WGSL)
    let shader_source = r#"
        @group(0) @binding(0) var<storage, read> input: array<f32>;
        @group(0) @binding(1) var<storage, read_write> output: array<f32>;

        @compute @workgroup_size(256)
        fn main(@builtin(global_invocation_id) id: vec3<u32>) {
            let i = id.x;
            if (i < arrayLength(&input)) {
                output[i] = input[i] * 2.0 + 1.0;
            }
        }
    "#;

    let pipeline = ctx.create_compute_pipeline("map_shader", shader_source, "main");

    // Create bind group layout and bind group
    let bind_group_layout = pipeline.get_bind_group_layout(0);
    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Compute Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buffer.as_entire_binding(),
            },
        ],
    });

    // Execute compute pass
    let start = Instant::now();
    
    let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Compute Encoder"),
    });
    
    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        // Dispatch enough workgroups to cover all elements
        let workgroups = (data_size + 255) / 256;
        compute_pass.dispatch_workgroups(workgroups, 1, 1);
    }
    
    // Copy output to staging buffer
    encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, 
        (data_size as u64) * std::mem::size_of::<f32>() as u64);
    
    ctx.queue.submit(std::iter::once(encoder.finish()));
    
    // Wait for GPU to complete
    ctx.device.poll(wgpu::Maintain::Wait);
    
    let gpu_duration = start.elapsed();
    println!("GPU compute    ({} elements): {:?}", data_size, gpu_duration);

    // Calculate speedup
    let speedup = cpu_duration.as_secs_f64() / gpu_duration.as_secs_f64();
    println!("\n🚀 GPU Speedup: {:.2}x", speedup);

    println!("\n=== Benchmark Complete ===");
}

#[cfg(not(feature = "gpu"))]
fn main() {
    println!("⚠️  GPU feature not enabled.");
    println!("Run with: cargo run --example gpu_bench --features gpu --release");
}
