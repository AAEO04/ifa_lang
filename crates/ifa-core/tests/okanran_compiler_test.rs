use ifa_core::compiler::Compiler;
use ifa_core::parser::parse;
use ifa_core::vm::IfaVM;
use ifa_types::IfaValue;

#[test]
fn test_gbiyanju_syntax_success() {
    let source = r#"
    gbiyanju {
        ayanmo x = 10;
        ayanmo y = 2;
        ayanmo z = x / y; // 5
        pada z;
    } gba (e) {
        // Should not reach here
        pada 999;
    }
    "#;

    let program = parse(source).expect("Failed to parse");
    let compiler = Compiler::new("test");
    let bytecode = compiler.compile(&program).expect("Failed to compile");

    let mut vm = IfaVM::new();
    let result = vm.execute(&bytecode).expect("VM failed");

    assert_eq!(result, IfaValue::Int(5));
}

#[test]
fn test_gbiyanju_syntax_failure() {
    let source = r#"
    gbiyanju {
        ayanmo x = 10;
        ayanmo y = 0;
        ayanmo z = x / y; // Error!
        pada z;
    } gba (e) {
        // e contains error info
        pada -1;
    }
    "#;

    let program = parse(source).expect("Failed to parse");
    let compiler = Compiler::new("test");
    let bytecode = compiler.compile(&program).expect("Failed to compile");

    let mut vm = IfaVM::new();
    let result = vm.execute(&bytecode).expect("VM failed");

    // Should catch error and return -1
    assert_eq!(result, IfaValue::Int(-1));
}
