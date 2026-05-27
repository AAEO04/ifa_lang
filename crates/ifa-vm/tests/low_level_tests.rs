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
    ayanmo p = &1234;
    "#;
    let p = run_code_and_get(code, "p");
    assert_eq!(p, IfaValue::Int(1234));
}

#[test]
fn test_dereference_read() {
    let code = r#"
    ayanmo p = &1234;
    *p = 42;
    ayanmo y = *p;
    "#;
    let y = run_code_and_get(code, "y");
    assert_eq!(y, IfaValue::Int(42));
}

#[test]
fn test_dereference_write() {
    let code = r#"
    ayanmo p = &1234;
    *p = 100;
    "#;
    let bytecode = ifa_compiler::compile(code).unwrap();
    let mut vm = IfaVM::new();
    vm.execute(&bytecode).unwrap();
    let x = vm.opon.get(1234).unwrap();
    assert_eq!(*x, IfaValue::Int(100));
}

#[test]
fn test_ailewu_block() {
    let code = r#"
    ailewu {
        ayanmo x = 1;
    }
    "#;
    let bytecode = ifa_compiler::compile(code).unwrap();
    let mut vm = IfaVM::new();
    vm.execute(&bytecode).unwrap();
}
