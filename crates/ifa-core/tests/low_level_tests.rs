#![cfg(feature = "std")]

use ifa_core::interpreter::Interpreter;
use ifa_core::opon::OponSize;
use ifa_core::parser::parse;
use ifa_core::value::IfaValue;

fn run_code(source: &str) -> Interpreter {
    let program = parse(source).unwrap();
    let mut interp = Interpreter::new();
    interp.execute(&program).unwrap();
    interp
}

#[test]
fn test_reference_creation() {
    let code = r#"
    ayanmo x = 10;
    ayanmo p = &x;
    "#;
    let interp = run_code(code);

    let _p = ifa_core::interpreter::Environment::get(&interp.env, "p").unwrap();
    // Just ensure it exists
    assert!(true);
}

#[test]
fn test_dereference_read() {
    let code = r#"
    ayanmo x = 42;
    ayanmo p = &x;
    ayanmo y = *p;
    "#;
    let interp = run_code(code);

    let y = ifa_core::interpreter::Environment::get(&interp.env, "y").unwrap();
    assert_eq!(y, IfaValue::Int(42));
}

#[test]
fn test_dereference_write() {
    let code = r#"
    ayanmo x = 10;
    ayanmo p = &x;
    *p = 100;
    "#;
    let interp = run_code(code);

    let x = ifa_core::interpreter::Environment::get(&interp.env, "x").unwrap();
    assert_eq!(x, IfaValue::Int(100));
}

#[test]
fn test_ailewu_block() {
    // Just checks that parsing and execution doesn't crash
    let code = r#"
    ailewu {
        ayanmo x = 1;
    }
    "#;
    run_code(code);
}
