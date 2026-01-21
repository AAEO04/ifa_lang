//! HPC Infrastructure Benchmarks
//!
//! Demonstrates performance gains from the infra layer.

use std::time::Instant;

/// Benchmark parallel vs sequential operations
fn main() {
    println!("=== Ifá-Lang HPC Benchmark ===\n");

    // Test data size
    let data_size = 10_000_000;
    let data: Vec<f64> = (0..data_size).map(|i| i as f64).collect();

    // --- Sequential Map ---
    let start = Instant::now();
    let _result: Vec<f64> = data.iter().map(|x| x * 2.0 + 1.0).collect();
    let seq_duration = start.elapsed();
    println!(
        "Sequential map ({} elements): {:?}",
        data_size, seq_duration
    );

    // --- Parallel Map (via CpuContext) ---
    #[cfg(feature = "parallel")]
    {
        use ifa_std::infra::cpu::CpuContext;

        let start = Instant::now();
        let _result = CpuContext::par_map(&data, |x| x * 2.0 + 1.0);
        let par_duration = start.elapsed();
        println!(
            "Parallel map   ({} elements): {:?}",
            data_size, par_duration
        );

        let speedup = seq_duration.as_secs_f64() / par_duration.as_secs_f64();
        println!("\n🚀 Speedup: {:.2}x", speedup);
    }

    #[cfg(not(feature = "parallel"))]
    {
        println!(
            "\n⚠️  Parallel feature not enabled. Run with: cargo run --example hpc_bench --features parallel"
        );
    }

    println!("\n=== Benchmark Complete ===");
}
