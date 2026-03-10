use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ifa_core::{Bytecode, IfaVM, IfaValue, OpCode};

/// Benchmark arithmetic operations (Push + Add)
fn bench_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("arithmetic");

    // Setup a simple addition loop bytecode
    // We can't easily loop inside the VM without jump overhead,
    // so we'll benchmark a single run of a block of additions.
    let mut bc = Bytecode::new("bench");

    // Fill with 1000 pairs of Push + Add
    // To keep stack balanced: Push 1, Push 1, Add, Pop
    for _ in 0..1000 {
        bc.code.push(OpCode::PushInt as u8);
        bc.code.extend_from_slice(&1i64.to_le_bytes());

        bc.code.push(OpCode::PushInt as u8);
        bc.code.extend_from_slice(&2i64.to_le_bytes());

        bc.code.push(OpCode::Add as u8);

        bc.code.push(OpCode::Pop as u8);
    }

    bc.code.push(OpCode::Halt as u8);

    group.bench_function("add_1000_ops", |b| {
        b.iter(|| {
            let mut vm = IfaVM::new();
            // We're measuring VM setup + execution here.
            // Ideally we'd reuse VM but reset it.
            let _ = vm.execute(black_box(&bc));
        })
    });

    group.finish();
}

/// Benchmark dispatch overhead (Nops/Pops)
fn bench_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch");

    let mut bc = Bytecode::new("bench");

    // 1000 simple operations (PushNull + Pop)
    for _ in 0..1000 {
        bc.code.push(OpCode::PushNull as u8);
        bc.code.push(OpCode::Pop as u8);
    }
    bc.code.push(OpCode::Halt as u8);

    group.bench_function("dispatch_1000_ops", |b| {
        b.iter(|| {
            let mut vm = IfaVM::new();
            let _ = vm.execute(black_box(&bc));
        })
    });

    group.finish();
}

criterion_group!(benches, bench_arithmetic, bench_dispatch);
criterion_main!(benches);
