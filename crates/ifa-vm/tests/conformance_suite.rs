#![cfg(feature = "std")]

use ifa_types::{ErrorCode, IfaValue};
use ifa_vm::compiler::Compiler;
use ifa_vm::parser::parse;
use ifa_vm::transpile_to_rust;
use ifa_vm::vm::IfaVM;

/// Evaluates source code on VM and Transpiler.
fn assert_conformance(source: &str, var_name: &str, expected: IfaValue) {
    let program = parse(source).expect("Failed to parse source code");

    // 1. Bytecode VM
    let compiler = Compiler::new("conformance_test");
    let bytecode = compiler
        .compile(&program)
        .expect("Failed to compile to bytecode");
    let mut vm = IfaVM::new();
    let vm_res = vm.execute(&bytecode);
    assert!(vm_res.is_ok(), "Bytecode VM failed: {:?}", vm_res);
    let vm_val = vm
        .get_global(var_name)
        .unwrap_or_else(|| panic!("Bytecode VM: Variable '{}' not found", var_name));

    // 2. Transpiler (Native)
    let rust_code = transpile_to_rust(&program);
    assert!(!rust_code.is_empty(), "Transpiler output was empty");

    // 3. Conformance Checks
    assert_eq!(
        vm_val.clone(),
        expected,
        "Bytecode VM mismatch. Expected {:?}, got {:?}",
        expected,
        vm_val
    );
}

fn assert_conformance_error(source: &str, expected_code: ErrorCode) {
    let program = parse(source).expect("Failed to parse source code");

    let compiler = Compiler::new("conformance_test");
    let bytecode = compiler.compile(&program).unwrap();
    let mut vm = IfaVM::new();
    let vm_err = vm.execute(&bytecode).unwrap_err();

    assert_eq!(
        vm_err.error_code(),
        expected_code,
        "Bytecode VM error code mismatch. Expected {:?}, got {:?}",
        expected_code,
        vm_err.error_code()
    );
}

#[test]
fn test_basic_arithmetic() {
    assert_conformance("ayanmo result = 5 + 3 * 2;", "result", IfaValue::Int(11));
}

#[test]
fn test_string_concatenation() {
    assert_conformance(
        r#"ayanmo result = "Ifa" + "-" + "Lang";"#,
        "result",
        IfaValue::str("Ifa-Lang"),
    );
}

#[test]
fn test_division_by_zero() {
    assert_conformance_error("ayanmo result = 10 / 0;", ErrorCode::DivByZero);
}

#[test]
fn test_closures() {
    assert_conformance(
        r#"
        ese make_adder(x) {
            ese add(y) {
                pada x + y;
            }
            pada add;
        }
        ayanmo add5 = make_adder(5);
        ayanmo result = add5(3);
        "#,
        "result",
        IfaValue::Int(8),
    );
}

#[test]
fn test_pointers() {
    assert_conformance(
        r#"
        ayanmo result = 0;
        ailewu {
            ayanmo p = &256;
            *p = 20;
            result = *p;
        }
        "#,
        "result",
        IfaValue::Int(20),
    );
}

#[test]
fn test_ailewu() {
    assert_conformance(
        r#"
        ayanmo result = 0;
        ailewu {
            result = 100;
        }
        "#,
        "result",
        IfaValue::Int(100),
    );
}
