//! Performance benchmarks for ifa-core

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ifa_core::*;

fn bench_value_arithmetic(c: &mut Criterion) {
    c.bench_function("value_addition", |b| {
        b.iter(|| {
            let a = IfaValue::Int(42);
            let b = IfaValue::Int(24);
            black_box(a + b)
        })
    });

    c.bench_function("value_multiplication", |b| {
        b.iter(|| {
            let a = IfaValue::Float(3.14);
            let b = IfaValue::Float(2.71);
            black_box(a * b)
        })
    });

    c.bench_function("value_string_concat", |b| {
        b.iter(|| {
            let a = IfaValue::Str("Hello, ".to_string());
            let b = IfaValue::Str("World!".to_string());
            black_box(a + b)
        })
    });
}

fn bench_vm_execution(c: &mut Criterion) {
    let mut bytecode = Bytecode::new();

    // Create a complex arithmetic expression
    for i in 0..100 {
        bytecode.write_op(OpCode::PushInt(i));
        bytecode.write_op(OpCode::PushInt(i + 1));
        bytecode.write_op(OpCode::Add);
        bytecode.write_op(OpCode::Pop);
    }
    bytecode.write_op(OpCode::PushInt(42));
    bytecode.write_op(OpCode::Return);

    c.bench_function("vm_simple_arithmetic", |b| {
        b.iter(|| {
            let mut vm = IfaVM::new();
            black_box(vm.execute(&bytecode).unwrap())
        })
    });

    // Benchmark with function calls
    let mut func_bytecode = Bytecode::new();

    // Function definition
    func_bytecode.write_op(OpCode::Jump(20));
    func_bytecode.write_op(OpCode::Add);
    func_bytecode.write_op(OpCode::Return);

    // Main code with many function calls
    for _ in 0..100 {
        func_bytecode.write_op(OpCode::PushInt(10));
        func_bytecode.write_op(OpCode::PushInt(20));
        func_bytecode.write_op(OpCode::Call { ip: 10 });
        func_bytecode.write_op(OpCode::Pop);
    }
    func_bytecode.write_op(OpCode::Return);

    c.bench_function("vm_function_calls", |b| {
        b.iter(|| {
            let mut vm = IfaVM::new();
            black_box(vm.execute(&func_bytecode).unwrap())
        })
    });
}

fn bench_compilation(c: &mut Criterion) {
    let program = Program {
        statements: vec![
            Statement::VariableDeclaration {
                name: "x".to_string(),
                value: Expression::Literal(IfaValue::Int(10)),
            },
            Statement::VariableDeclaration {
                name: "y".to_string(),
                value: Expression::Literal(IfaValue::Int(20)),
            },
            Statement::Expression(Expression::BinaryOp {
                left: Box::new(Expression::Variable("x".to_string())),
                op: crate::ast::BinaryOperator::Add,
                right: Box::new(Expression::Variable("y".to_string())),
            }),
        ],
    };

    c.bench_function("compile_simple_program", |b| {
        b.iter(|| black_box(compile(program.clone()).unwrap()))
    });

    // Large program compilation
    let mut statements = Vec::new();
    for i in 0..1000 {
        statements.push(Statement::VariableDeclaration {
            name: format!("var{}", i),
            value: Expression::Literal(IfaValue::Int(i)),
        });
    }
    let large_program = Program { statements };

    c.bench_function("compile_large_program", |b| {
        b.iter(|| black_box(compile(large_program.clone()).unwrap()))
    });
}

fn bench_parsing(c: &mut Criterion) {
    let simple_code = "1 + 2 * 3 - 4 / 5";

    c.bench_function("parse_simple_expression", |b| {
        b.iter(|| {
            let tokens = tokenize(simple_code).unwrap();
            black_box(parse(tokens).unwrap())
        })
    });

    let complex_code = r#"
        let x = 10
        let y = 20
        let z = x + y * 2
        fn add(a, b) {
            a + b
        }
        add(z, 5)
    "#;

    c.bench_function("parse_complex_program", |b| {
        b.iter(|| {
            let tokens = tokenize(complex_code).unwrap();
            black_box(parse(tokens).unwrap())
        })
    });
}

fn bench_tokenization(c: &mut Criterion) {
    let simple_code = "1 + 2 * 3";

    c.bench_function("tokenize_simple", |b| {
        b.iter(|| black_box(tokenize(simple_code).unwrap()))
    });

    let large_code = "let x = 1\n".repeat(1000);

    c.bench_function("tokenize_large", |b| {
        b.iter(|| black_box(tokenize(&large_code).unwrap()))
    });
}

fn bench_memory_operations(c: &mut Criterion) {
    c.bench_function("opon_allocate", |b| {
        b.iter(|| {
            let opon = Opon::with_capacity(1024);
            black_box(opon)
        })
    });

    c.bench_function("value_list_operations", |b| {
        b.iter(|| {
            let mut list = IfaValue::List(Vec::new());
            for i in 0..100 {
                list.push(IfaValue::Int(i)).unwrap();
            }
            black_box(list)
        })
    });

    c.bench_function("value_map_operations", |b| {
        b.iter(|| {
            let mut map = std::collections::HashMap::new();
            for i in 0..100 {
                map.insert(format!("key{}", i), IfaValue::Int(i));
            }
            black_box(IfaValue::Map(map))
        })
    });
}

fn bench_string_operations(c: &mut Criterion) {
    let base_string = "Hello, World! ".repeat(100);

    c.bench_function("string_concatenation", |b| {
        b.iter(|| {
            let a = IfaValue::Str(base_string.clone());
            let b = IfaValue::Str(base_string.clone());
            black_box(a + b)
        })
    });

    c.bench_function("string_indexing", |b| {
        let s = IfaValue::Str(base_string.clone());
        b.iter(|| {
            for i in 0..100 {
                black_box(s.get(&IfaValue::Int(i)).unwrap());
            }
        })
    });

    c.bench_function("string_slicing", |b| {
        let s = IfaValue::Str(base_string.clone());
        b.iter(|| black_box(s.slice(0, 100).unwrap()))
    });
}

criterion_group!(
    benches,
    bench_value_arithmetic,
    bench_vm_execution,
    bench_compilation,
    bench_parsing,
    bench_tokenization,
    bench_memory_operations,
    bench_string_operations
);

criterion_main!(benches);
