#![cfg(feature = "std")]

use ifa_core::interpreter::Interpreter;
use ifa_core::opon::{OponSize, create_opon_with_panic_handler};
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

    let p = interp.env.get("p").unwrap();
    if let IfaValue::Ref(name) = p {
        assert_eq!(name, "x");
    } else {
        panic!("Expected Ref, got {:?}", p);
    }
}

#[test]
fn test_dereference_read() {
    let code = r#"
    ayanmo x = 42;
    ayanmo p = &x;
    ayanmo y = *p;
    "#;
    let interp = run_code(code);

    let y = interp.env.get("y").unwrap();
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

    let x = interp.env.get("x").unwrap();
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
