#![cfg(feature = "std")]

use ifa_core::compiler::Compiler;
use ifa_core::parser::parse;

#[test]
fn async_function_returns_future_and_await_unwraps() {
    let source = r#"
    daro ese fetch(x) { pada x + 1; }
    pada reti fetch(41);
    "#;
    let program = parse(source).expect("parse failed");
    let compiler = Compiler::new("async_function_returns_future_and_await_unwraps");
    let bytecode = compiler.compile(&program).expect("compile failed");
    let mut vm = ifa_core::vm::IfaVM::new();
    let got = vm.execute(&bytecode).expect("vm failed");
    assert_eq!(got, ifa_types::IfaValue::Int(42));
}

#[test]
fn osa_spawn_returns_future() {
    let source = r#"
    ese work() { pada 7; }
    ayanmo f = Osa.ise(work);
    pada reti f;
    "#;
    let program = parse(source).expect("parse failed");
    let compiler = Compiler::new("osa_spawn_returns_future");
    let bytecode = compiler.compile(&program).expect("compile failed");
    let mut vm = ifa_core::vm::IfaVM::new();
    let got = vm.execute(&bytecode).expect("vm failed");
    assert_eq!(got, ifa_types::IfaValue::Int(7));
}
