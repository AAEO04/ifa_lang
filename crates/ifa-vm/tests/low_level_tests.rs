use ifa_vm::{IfaVM, IfaValue};

fn run_code_and_get(source: &str, var: &str) -> IfaValue {
    let bytecode = ifa_compiler::compile(source).unwrap();
    let mut vm = IfaVM::new();
    vm.execute(&bytecode).unwrap();
    vm.get_global(var).unwrap().clone()
}

#[test]
fn test_reference_creation() {
    let code = r#"
    ayanmo x = 10;
    ayanmo p = &x;
    "#;
    let p = run_code_and_get(code, "p");
    // Just ensure it exists and has some address (Int)
    if let IfaValue::Int(_) = p {
        assert!(true);
    } else {
        panic!("Expected address (Int), got {:?}", p);
    }
}

#[test]
fn test_dereference_read() {
    let code = r#"
    ayanmo x = 42;
    ayanmo p = &x;
    ayanmo y = *p;
    "#;
    let y = run_code_and_get(code, "y");
    assert_eq!(y, IfaValue::Int(42));
}

#[test]
fn test_dereference_write() {
    let code = r#"
    ayanmo x = 10;
    ayanmo p = &x;
    *p = 100;
    "#;
    // We want to inspect "x" after running.
    let bytecode = ifa_compiler::compile(code).unwrap();
    let mut vm = IfaVM::new();
    vm.execute(&bytecode).unwrap();
    let x = vm.get_global("x").unwrap();
    assert_eq!(*x, IfaValue::Int(100));
}

#[test]
fn test_ailewu_block() {
    // Just checks that parsing and execution doesn't crash
    let code = r#"
    ailewu {
        ayanmo x = 1;
    }
    "#;
    let bytecode = ifa_compiler::compile(code).unwrap();
    let mut vm = IfaVM::new();
    vm.execute(&bytecode).unwrap();
}
