//! GPU Matrix Multiply Benchmark
//!
//! Demonstrates massive GPU speedups for compute-intensive operations.

#[cfg(feature = "gpu")]
fn main() {
    use ifa_std::infra::gpu::GpuContext;
    use std::time::Instant;
    use wgpu::util::DeviceExt;

    println!("=== Ifá-Lang GPU Matrix Multiply Benchmark ===\n");

    // Matrix size (N x N)
    let n: u32 = 1024;
    let matrix_size = (n * n) as usize;

    // Initialize matrices with random values
    let a: Vec<f32> = (0..matrix_size).map(|i| (i % 17) as f32 * 0.1).collect();
    let b: Vec<f32> = (0..matrix_size).map(|i| (i % 19) as f32 * 0.1).collect();

    println!(
        "Matrix size: {}x{} ({:.1}M elements)",
        n,
        n,
        matrix_size as f64 / 1_000_000.0
    );
    println!(
        "Total FLOPs: {:.1}B",
        (2.0 * (n as f64).powi(3)) / 1_000_000_000.0
    );

    // --- CPU Baseline (naive triple loop) ---
    let start = Instant::now();
    let mut cpu_result = vec![0.0f32; matrix_size];
    for i in 0..n as usize {
        for j in 0..n as usize {
            let mut sum = 0.0f32;
            for k in 0..n as usize {
                sum += a[i * n as usize + k] * b[k * n as usize + j];
            }
            cpu_result[i * n as usize + j] = sum;
        }
    }
    let cpu_duration = start.elapsed();
    println!("\nCPU naive matmul: {:?}", cpu_duration);
    let cpu_gflops = (2.0 * (n as f64).powi(3)) / cpu_duration.as_secs_f64() / 1e9;
    println!("CPU throughput: {:.2} GFLOPS", cpu_gflops);

    // --- GPU Compute ---
    let ctx = GpuContext::new_blocking().expect("Failed to initialize GPU");

    // Create buffers
    let buffer_a = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Matrix A"),
            contents: bytemuck::cast_slice(&a),
            usage: wgpu::BufferUsages::STORAGE,
        });

    let buffer_b = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Matrix B"),
            contents: bytemuck::cast_slice(&b),
            usage: wgpu::BufferUsages::STORAGE,
        });

    let buffer_c = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Matrix C"),
        size: (matrix_size * std::mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // Compute shader for matrix multiplication
    let shader_source = format!(
        r#"
        @group(0) @binding(0) var<storage, read> a: array<f32>;
        @group(0) @binding(1) var<storage, read> b: array<f32>;
        @group(0) @binding(2) var<storage, read_write> c: array<f32>;

        const N: u32 = {n}u;

        @compute @workgroup_size(16, 16)
        fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
            let row = id.x;
            let col = id.y;
            
            if (row >= N || col >= N) {{
                return;
            }}
            
            var sum: f32 = 0.0;
            for (var k: u32 = 0u; k < N; k = k + 1u) {{
                sum = sum + a[row * N + k] * b[k * N + col];
            }}
            c[row * N + col] = sum;
        }}
    "#
    );

    let pipeline = ctx.create_compute_pipeline("matmul", &shader_source, "main");

    let bind_group_layout = pipeline.get_bind_group_layout(0);
    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Matmul Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer_a.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buffer_b.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: buffer_c.as_entire_binding(),
            },
        ],
    });

    // Execute and time
    let start = Instant::now();

    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Matmul Encoder"),
        });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Matmul Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        // Dispatch workgroups (ceil(N/16) x ceil(N/16))
        let workgroups = (n + 15) / 16;
        pass.dispatch_workgroups(workgroups, workgroups, 1);
    }

    ctx.queue.submit(std::iter::once(encoder.finish()));
    ctx.device.poll(wgpu::Maintain::Wait);

    let gpu_duration = start.elapsed();
    println!("\nGPU matmul: {:?}", gpu_duration);
    let gpu_gflops = (2.0 * (n as f64).powi(3)) / gpu_duration.as_secs_f64() / 1e9;
    println!("GPU throughput: {:.2} GFLOPS", gpu_gflops);

    // Calculate speedup
    let speedup = cpu_duration.as_secs_f64() / gpu_duration.as_secs_f64();
    println!("\n🚀 GPU Speedup: {:.1}x", speedup);
    println!(
        "   GPU is {:.1}x more efficient (GFLOPS)",
        gpu_gflops / cpu_gflops
    );

    println!("\n=== Benchmark Complete ===");
}

#[cfg(not(feature = "gpu"))]
fn main() {
    println!("⚠️  GPU feature not enabled.");
    println!("Run with: cargo run --example gpu_matmul --features gpu --release");
}
